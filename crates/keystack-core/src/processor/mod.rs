use snafu::Snafu;

use crate::{KeyPath, id_manager::User, processor::wasm_pre_processor::WasmPreProcessorError};

pub mod wasm_pre_processor;

#[derive(Debug, Snafu)]
pub enum PreProcessorError {
    WasmPreProcessorError { source: WasmPreProcessorError },
}

impl From<WasmPreProcessorError> for PreProcessorError {
    fn from(source: WasmPreProcessorError) -> Self {
        PreProcessorError::WasmPreProcessorError { source }
    }
}

pub struct PreProcessorContext {
    pub user: User,
    pub key_path: KeyPath,
    pub action_id: String,
    pub payload: Vec<u8>,
}

pub trait PreProcessor {
    /// Pre-process processes the given context and returns JSON bytes that will be returned
    /// alongside other pre-processor results.
    fn pre_process(&self, context: &PreProcessorContext) -> Result<Vec<u8>, PreProcessorError>;
}
