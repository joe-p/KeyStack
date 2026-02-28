use snafu::Snafu;

use crate::{KeyPath, id_manager::User};

#[derive(Debug, Snafu)]
pub enum PreProcessorError {}

pub struct PreProcessContext {
    pub user: User,
    pub key_path: KeyPath,
    pub action_id: String,
    pub payload: Vec<u8>,
}

pub trait PreProcessor {
    /// Pre-process processes the given context and returns JSON bytes that will be returned
    /// alongside other pre-processor results.
    fn pre_process(&self, context: &PreProcessContext) -> Result<Vec<u8>, PreProcessorError>;
}
