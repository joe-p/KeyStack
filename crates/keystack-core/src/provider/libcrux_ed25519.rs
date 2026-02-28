use async_trait::async_trait;
use libcrux_ed25519::{secret_to_public, sign};

use crate::provider::{Provider, ProviderError};

pub struct LibCruxEd25519Provider;

pub enum LibCruxEd25519Action {
    Generate,
    Sign,
}

impl From<&str> for LibCruxEd25519Action {
    fn from(s: &str) -> Self {
        match s {
            "generate" => Self::Generate,
            "sign" => Self::Sign,
            _ => panic!("Unknown action: {}", s),
        }
    }
}

#[async_trait]
impl Provider for LibCruxEd25519Provider {
    fn name(&self) -> String {
        "builtin-libcrux-ed25519".to_string()
    }

    fn version(&self) -> String {
        "0.1.0".to_string()
    }

    async fn do_action(
        &self,
        request: &crate::provider::ActionRequest,
    ) -> Result<Vec<u8>, ProviderError> {
        let action = LibCruxEd25519Action::from(request.action_id.as_str());
        match action {
            LibCruxEd25519Action::Generate => {
                let mut random_bytes = [0u8; 32];
                getrandom::fill(&mut random_bytes).unwrap();

                request
                    .scoped_backend
                    .create(&"".into(), &random_bytes)
                    .await
                    .map_err(|e| ProviderError::BackendError { source: e })?;

                let mut pk = [0u8; 32];
                secret_to_public(&mut pk, &random_bytes);

                Ok(pk.to_vec())
            }
            LibCruxEd25519Action::Sign => {
                let mut key_bytes = [0u8; 32];
                request
                    .scoped_backend
                    .read(&"".into(), &mut key_bytes)
                    .await
                    .unwrap();

                Ok(sign(&request.payload, &key_bytes)
                    .map_err(|e| format!("Signing failed: {:?}", e))?
                    .to_vec())
            }
        }
    }
}
