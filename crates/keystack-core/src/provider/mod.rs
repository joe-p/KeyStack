use async_trait::async_trait;
use snafu::Snafu;

use crate::backend::ScopedBackend;

pub struct ActionRequest {
    pub action_id: String,
    pub scoped_backend: ScopedBackend,
    pub payload: Vec<u8>,
}

#[derive(Debug, Snafu)]
pub enum ProviderError {}

#[async_trait]
pub trait Provider {
    async fn do_action(&self, request: &ActionRequest) -> Result<Vec<u8>, ProviderError>;
}
