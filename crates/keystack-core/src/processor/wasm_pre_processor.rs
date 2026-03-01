use keystack_wasm_guest::PreProcessorGuestContext;
use snafu::Snafu;
use wasmtime::{Caller, Engine, Linker, Memory, Module, Store, TypedFunc};

use crate::processor::{PreProcessor, PreProcessorContext, PreProcessorError};

impl From<PreProcessorContext> for PreProcessorGuestContext {
    fn from(context: PreProcessorContext) -> Self {
        Self {
            user: context.user.id().to_string(),
            key_path: context.key_path.0,
            action_id: context.action_id,
            payload: context.payload,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum WasmPreProcessorError {
    ModuleFailed,
    LinkerFailed,
    InstantiateFailed,
    GetFuncFailed,
    GetMemoryFailed,
    AllocFailed,
    MemoryWriteFailed,
    MemoryReadFailed,
    CallFailed,
}

pub struct WasmPreProcessor {
    engine: Engine,
    module: Module,
}

impl WasmPreProcessor {
    pub fn from_module(
        engine: &Engine,
        wasm_bytes: impl AsRef<[u8]>,
    ) -> Result<Self, WasmPreProcessorError> {
        let module =
            Module::new(engine, wasm_bytes).map_err(|_| WasmPreProcessorError::ModuleFailed)?;

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
    ) -> Result<(i32, i32), WasmPreProcessorError> {
        let len = data.len() as i32;

        let ptr = alloc_func
            .call(&mut *store, len)
            .map_err(|_| WasmPreProcessorError::AllocFailed)?;

        if ptr == 0 {
            return Err(WasmPreProcessorError::AllocFailed);
        }

        memory
            .write(store, ptr as usize, data)
            .map_err(|_| WasmPreProcessorError::MemoryWriteFailed)?;

        Ok((ptr, len))
    }
}

impl PreProcessor for WasmPreProcessor {
    fn pre_process(&self, context: &PreProcessorContext) -> Result<Vec<u8>, PreProcessorError> {
        let start = std::time::Instant::now();

        // Host functionality can be arbitrary Rust functions and is provided
        // to guests through a `Linker`.
        let mut linker = Linker::new(&self.engine);

        linker
            .func_wrap(
                "host",
                "host_func",
                move |_caller: Caller<_>, param: i32| {
                    println!("Got {} from WebAssembly", param);
                },
            )
            .map_err(|_| WasmPreProcessorError::LinkerFailed)?;

        let mut store = Store::new(&self.engine, ());

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|_| WasmPreProcessorError::InstantiateFailed)?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(WasmPreProcessorError::GetMemoryFailed)?;

        let alloc_func = instance
            .get_typed_func::<i32, i32>(&mut store, "alloc")
            .map_err(|_| WasmPreProcessorError::GetFuncFailed)?;

        let process_func = instance
            .get_typed_func::<(i32, i32, i32, i32, i32, i32, i32, i32), (i32, i32)>(
                &mut store, "process",
            )
            .map_err(|_| WasmPreProcessorError::GetFuncFailed)?;

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

        let (result_ptr, result_len) = process_func
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
            .map_err(|_| WasmPreProcessorError::CallFailed)?;

        let result_len_usize = result_len as usize;
        let mut result_bytes = vec![0u8; result_len_usize];
        memory
            .read(&mut store, result_ptr as usize, &mut result_bytes)
            .map_err(|_| WasmPreProcessorError::MemoryReadFailed)?;

        println!("WASM pre-processing completed in {:?}", start.elapsed());

        Ok(result_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyPath;
    use crate::id_manager::User;
    use crate::id_manager::disabled_id_manager::DisabledIdentityManager;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_wasm_pre_processor_with_context() {
        let engine = Engine::default();
        let wat = r#"
        (module
            (import "host" "host_func" (func $host_hello (param i32)))
            
            ;; Memory export for host to write context data
            (memory (export "memory") 2)
            
            ;; Heap starts after data/stack section
            (global $heap_offset (mut i32) (i32.const 1024))
            
            ;; alloc: Simple bump allocator
            ;; Returns pointer to allocated memory or 0 on failure
            (func $alloc (export "alloc") (param $size i32) (result i32)
                (local $ptr i32)
                ;; Get current heap pointer
                (local.set $ptr (global.get $heap_offset))
                ;; Check bounds (memory size is 2 pages = 128KB = 131072 bytes)
                (if (i32.gt_u (i32.add (local.get $ptr) (local.get $size)) (i32.const 131072))
                    (then (return (i32.const 0)))
                )
                ;; Advance heap pointer
                (global.set $heap_offset (i32.add (local.get $ptr) (local.get $size)))
                ;; Return allocated pointer
                (local.get $ptr)
            )
             
            ;; process: Accepts 8 parameters (4 pointers + 4 lengths) for context members
            ;; user_ptr, user_len, key_path_ptr, key_path_len,
            ;; action_id_ptr, action_id_len, payload_ptr, payload_len
            ;; Returns: (ptr, len) tuple pointing to result data
            (func (export "process") 
                (param $user_ptr i32) (param $user_len i32)
                (param $key_path_ptr i32) (param $key_path_len i32)
                (param $action_id_ptr i32) (param $action_id_len i32)
                (param $payload_ptr i32) (param $payload_len i32)
                (result i32 i32)
                (local $result_ptr i32)
                (local $result_len i32)
                ;; Allocate memory for result data (4 bytes for test data)
                (local.set $result_len (i32.const 4))
                (local.set $result_ptr (call $alloc (local.get $result_len)))
                ;; Write test result data: [0x01, 0x02, 0x03, 0x04]
                (i32.store8 (local.get $result_ptr) (i32.const 1))
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 1)) (i32.const 2))
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 2)) (i32.const 3))
                (i32.store8 (i32.add (local.get $result_ptr) (i32.const 3)) (i32.const 4))
                ;; Return (ptr, len) tuple
                (return (local.get $result_ptr) (local.get $result_len))
            )
        )
    "#;

        let pre_processor = WasmPreProcessor::from_module(&engine, wat).unwrap();

        // Create test context with sample data
        let identity_manager = Arc::new(DisabledIdentityManager);
        let user = User::new("test-user-123".to_string(), identity_manager);
        let key_path = KeyPath(PathBuf::from("test/key/path"));
        let action_id = "test-action".to_string();
        let payload = vec![1, 2, 3, 4, 5];

        let context = PreProcessorContext {
            user,
            key_path,
            action_id,
            payload,
        };

        let result = pre_processor.pre_process(&context).unwrap();

        // The process function returns test data: [0x01, 0x02, 0x03, 0x04]
        assert_eq!(result, vec![0x01, 0x02, 0x03, 0x04]);
    }
}
