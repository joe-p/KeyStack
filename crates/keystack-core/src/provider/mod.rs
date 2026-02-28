use async_trait::async_trait;
use snafu::Snafu;

use crate::backend::ScopedBackend;

pub mod libcrux_ed25519;

pub struct ActionRequest {
    pub action_id: String,
    pub scoped_backend: ScopedBackend,
    pub payload: Vec<u8>,
}

#[derive(Debug, Snafu)]
pub enum ProviderError {
    #[snafu(display("Provider action failed due to backend error: {}", source))]
    BackendError {
        source: crate::backend::BackendError,
    },
    #[snafu(display("Provider error: {}", message))]
    ProviderError { message: String },
}

impl From<String> for ProviderError {
    fn from(s: String) -> Self {
        ProviderError::ProviderError { message: s }
    }
}

#[async_trait]
pub trait Provider {
    async fn do_action(&self, request: &ActionRequest) -> Result<Vec<u8>, ProviderError>;

    fn name(&self) -> String;

    fn version(&self) -> String;
}
