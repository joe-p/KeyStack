use keystack_wasm_guest::ContextProviderGuestContext;
use snafu::Snafu;
use wasmtime::{Engine, Linker, Memory, Module, Store, TypedFunc};

use crate::context_provider::{ContextProvider, ContextProviderContext, ContextProviderError};

impl From<ContextProviderContext> for ContextProviderGuestContext {
    fn from(context: ContextProviderContext) -> Self {
        Self {
            user: context.user.id().to_string(),
            key_path: context.key_path.0,
            action_id: context.action_id,
            payload: context.payload,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum WasmContextProviderError {
    ModuleFailed,
    InstantiateFailed,
    GetFuncFailed,
    GetMemoryFailed,
    AllocFailed,
    MemoryWriteFailed,
    MemoryReadFailed,
    CallFailed,
}

pub struct WasmContextProvider {
    engine: Engine,
    module: Module,
}

impl WasmContextProvider {
    pub fn from_module(
        engine: &Engine,
        wasm_bytes: impl AsRef<[u8]>,
    ) -> Result<Self, WasmContextProviderError> {
        let module =
            Module::new(engine, wasm_bytes).map_err(|_| WasmContextProviderError::ModuleFailed)?;

        Ok(Self {
            engine: engine.clone(),
            module,
        })
    }

    fn alloc_and_write(
        &self,
        store: &mut Store<()>,
        alloc_func: &TypedFunc<i32, i32>,
        memory: &Memory,
        data: &[u8],
    ) -> Result<(i32, i32), WasmContextProviderError> {
        let len = data.len() as i32;

        let ptr = alloc_func
            .call(&mut *store, len)
            .map_err(|_| WasmContextProviderError::AllocFailed)?;

        if ptr == 0 {
            return Err(WasmContextProviderError::AllocFailed);
        }

        memory
            .write(store, ptr as usize, data)
            .map_err(|_| WasmContextProviderError::MemoryWriteFailed)?;

        Ok((ptr, len))
    }
}

impl ContextProvider for WasmContextProvider {
    fn pre_action_hook(
        &self,
        context: &ContextProviderContext,
    ) -> Result<Vec<u8>, ContextProviderError> {
        let start = std::time::Instant::now();

        // Host functionality can be arbitrary Rust functions and is provided
        // to guests through a `Linker`.
        let linker = Linker::new(&self.engine);

        let mut store = Store::new(&self.engine, ());

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|_| WasmContextProviderError::InstantiateFailed)?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(WasmContextProviderError::GetMemoryFailed)?;

        let alloc_func = instance
            .get_typed_func::<i32, i32>(&mut store, "alloc")
            .map_err(|_| WasmContextProviderError::GetFuncFailed)?;

        let hook = instance
            .get_typed_func::<(i32, i32, i32, i32, i32, i32, i32, i32), i32>(
                &mut store,
                "pre_action_hook",
            )
            .map_err(|_| WasmContextProviderError::GetFuncFailed)?;

        let user_bytes = context.user.id().to_string().into_bytes();
        let key_path_bytes = context
            .key_path
            .0
            .to_string_lossy()
            .into_owned()
            .into_bytes();
        let action_id_bytes = context.action_id.clone().into_bytes();

        let (user_ptr, user_len) =
            self.alloc_and_write(&mut store, &alloc_func, &memory, &user_bytes)?;
        let (key_path_ptr, key_path_len) =
            self.alloc_and_write(&mut store, &alloc_func, &memory, &key_path_bytes)?;
        let (action_id_ptr, action_id_len) =
            self.alloc_and_write(&mut store, &alloc_func, &memory, &action_id_bytes)?;
        let (payload_ptr, payload_len) =
            self.alloc_and_write(&mut store, &alloc_func, &memory, &context.payload)?;

        let result_ptr = hook
            .call(
                &mut store,
                (
                    user_ptr,
                    user_len,
                    key_path_ptr,
                    key_path_len,
                    action_id_ptr,
                    action_id_len,
                    payload_ptr,
                    payload_len,
                ),
            )
            .map_err(|_| WasmContextProviderError::CallFailed)?;

        if result_ptr == 0 {
            return Err(WasmContextProviderError::CallFailed.into());
        }

        // Read result length from first 4 bytes (little-endian i32)
        let mut result_len_bytes = [0u8; 4];
        memory
            .read(&mut store, result_ptr as usize, &mut result_len_bytes)
            .map_err(|_| WasmContextProviderError::MemoryReadFailed)?;
        let result_len = i32::from_le_bytes(result_len_bytes) as usize;

        // Read actual result data (starts after the 4-byte length prefix)
        let mut result_bytes = vec![0u8; result_len];
        memory
            .read(&mut store, (result_ptr as usize) + 4, &mut result_bytes)
            .map_err(|_| WasmContextProviderError::MemoryReadFailed)?;

        println!("WASM pre-action plugin completed in {:?}", start.elapsed());

        Ok(result_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyPath;
    use crate::tests::TestUser;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_wasm_context_provider_with_context() {
        let engine = Engine::default();

        std::process::Command::new("cargo")
            .args([
                "build",
                "--package",
                "simple-context-provider",
                "--target",
                "wasm32-unknown-unknown",
            ])
            .status()
            .expect("Failed to build WASM guest module");

        let cargo_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let wasm_path = cargo_dir
            .join("../../target/wasm32-unknown-unknown/debug/simple_context_provider.wasm");
        let wasm_bytes = std::fs::read(wasm_path).unwrap();

        let plugin = WasmContextProvider::from_module(&engine, wasm_bytes).unwrap();

        let user = TestUser {};
        let key_path = KeyPath(PathBuf::from("test/key/path"));
        let action_id = "test-action".to_string();
        let payload = vec![1, 2, 3, 4, 5];

        let context = ContextProviderContext {
            user: Arc::new(user),
            key_path,
            action_id,
            payload,
        };

        let result = plugin.pre_action_hook(&context).unwrap();

        // The pre_action_hook function returns test data: [0x01, 0x02, 0x03, 0x04]
        assert_eq!(result, vec![0x01, 0x02, 0x03, 0x04]);
    }
}
