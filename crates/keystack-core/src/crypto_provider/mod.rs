use async_trait::async_trait;
use snafu::Snafu;

use crate::secret_provider::ScopedSecretProvider;

pub mod libcrux_ed25519;

pub struct ActionRequest {
    pub action_id: String,
    pub scoped_secret_provider: ScopedSecretProvider,
    pub payload: Vec<u8>,
}

#[derive(Debug, Snafu)]
pub enum CryptoProviderError {
    #[snafu(display("Provider action failed due to secret_provider error: {}", source))]
    SecretProviderError {
        source: crate::secret_provider::SecretProviderError,
    },
    #[snafu(display("Provider error: {}", message))]
    CryptoProviderError { message: String },
}

impl From<String> for CryptoProviderError {
    fn from(s: String) -> Self {
        CryptoProviderError::CryptoProviderError { message: s }
    }
}

#[async_trait]
pub trait CryptoProvider {
    async fn do_action(&self, request: &ActionRequest) -> Result<Vec<u8>, CryptoProviderError>;

    fn name(&self) -> String;

    fn version(&self) -> String;
}
