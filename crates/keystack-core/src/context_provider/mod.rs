use std::sync::Arc;

use snafu::Snafu;

use crate::{
    KeyPath, context_provider::wasm_context_provider::WasmContextProviderError, user::User,
};

pub mod wasm_context_provider;

#[derive(Debug, Snafu)]
pub enum ContextProviderError {
    WasmContextProviderError { source: WasmContextProviderError },
}

impl From<WasmContextProviderError> for ContextProviderError {
    fn from(source: WasmContextProviderError) -> Self {
        ContextProviderError::WasmContextProviderError { source }
    }
}

pub struct ContextProviderContext {
    pub user: Arc<dyn User>,
    pub key_path: KeyPath,
    pub action_id: String,
    pub payload: Vec<u8>,
}

pub trait ContextProvider {
    /// Called before an action is executed and returns additional context that will be added to
    /// the response.
    fn pre_action_hook(
        &self,
        context: &ContextProviderContext,
    ) -> Result<Vec<u8>, ContextProviderError>;
}
