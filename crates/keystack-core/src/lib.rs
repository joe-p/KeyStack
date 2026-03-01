use std::{collections::HashMap, hash::Hash, path::PathBuf, sync::Arc};

use snafu::Snafu;

use crate::{
    backend::Backend,
    id_manager::IdentityManagerError,
    processor::PreProcessorError,
    provider::{Provider, ProviderError},
};

pub mod backend;
pub mod id_manager;
pub mod processor;
pub mod provider;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyPath(PathBuf);

impl From<&str> for KeyPath {
    fn from(value: &str) -> Self {
        Self(PathBuf::from(value))
    }
}

#[derive(Debug, Snafu)]
pub enum KeyStackError {
    #[snafu(display("Pre-processing failed: {}", source))]
    PreProcessorError { source: PreProcessorError },
    #[snafu(display("Provider action failed: {}", source))]
    ProviderError { source: ProviderError },
    #[snafu(display("Pre-processor not found: {}", id))]
    PreProcessorNotFound { id: String },
    #[snafu(display("Provider not found: {}", id))]
    ProviderNotFound { id: String },
    #[snafu(display("Identity manager error: {}", source))]
    IdentityManagerError { source: IdentityManagerError },
}

impl From<IdentityManagerError> for KeyStackError {
    fn from(source: IdentityManagerError) -> Self {
        KeyStackError::IdentityManagerError { source }
    }
}

pub enum KeyStackRequest {
    Action {
        key_path: KeyPath,
        pre_processor_ids: Vec<String>,
        action_id: String,
        payload: Vec<u8>,
        provider_id: String,
        auth_data: Option<Vec<u8>>,
        user_id: Option<String>,
    },
}

pub enum KeyStackResponse {
    Action {
        action_id: String,
        pre_processor_context: HashMap<String, Vec<u8>>,
        provider_response: Vec<u8>,
    },
}

pub struct KeyStack {
    required_pre_processors: Vec<String>,
    backend: Arc<dyn Backend>,
    pre_processors: HashMap<String, Arc<dyn processor::PreProcessor>>,
    providers: HashMap<String, Arc<dyn provider::Provider>>,
    identity_manager: Arc<dyn id_manager::IdentityManager>,
}

impl Default for KeyStack {
    fn default() -> Self {
        let ed25519_provider = provider::libcrux_ed25519::LibCruxEd25519Provider;

        let providers = HashMap::from([(
            ed25519_provider.name(),
            Arc::new(ed25519_provider) as Arc<dyn provider::Provider>,
        )]);

        Self {
            identity_manager: Arc::new(id_manager::disabled_id_manager::DisabledIdentityManager),
            required_pre_processors: Vec::new(),
            backend: Arc::new(backend::hashmap_backend::HashMapBackend {
                store: std::sync::Mutex::new(HashMap::new()),
            }),
            pre_processors: HashMap::new(),
            providers,
        }
    }
}

impl KeyStack {
    pub async fn handle_request(
        &self,
        request: KeyStackRequest,
    ) -> Result<KeyStackResponse, KeyStackError> {
        match &request {
            KeyStackRequest::Action {
                key_path,
                pre_processor_ids,
                action_id,
                payload,
                provider_id,
                auth_data,
                user_id,
            } => {
                self.identity_manager
                    .user_authenticate(&user_id.clone().unwrap_or_default(), auth_data.as_deref())
                    .await?;

                let user = id_manager::User::new(
                    "default-user".to_string(),
                    self.identity_manager.clone(),
                );
                let all_pre_processor_ids = self
                    .required_pre_processors
                    .iter()
                    .chain(pre_processor_ids.iter())
                    .cloned()
                    .collect::<Vec<_>>();

                let context = processor::PreProcessorContext {
                    user,
                    key_path: key_path.clone(),
                    action_id: action_id.clone(),
                    payload: payload.clone(),
                };

                let mut pre_processor_results = HashMap::new();
                for pre_processor_str in all_pre_processor_ids {
                    let pre_processor =
                        self.pre_processors.get(&pre_processor_str).ok_or_else(|| {
                            KeyStackError::PreProcessorNotFound {
                                id: pre_processor_str.clone(),
                            }
                        })?;

                    let result = pre_processor
                        .pre_process(&context)
                        .map_err(|e| KeyStackError::PreProcessorError { source: e })?;

                    pre_processor_results.insert(pre_processor_str, result);
                }

                let provider = self.providers.get(provider_id).ok_or_else(|| {
                    KeyStackError::ProviderNotFound {
                        id: provider_id.clone(),
                    }
                })?;

                let scoped_backend =
                    backend::ScopedBackend::new(self.backend.clone(), key_path.clone());

                let action_request = provider::ActionRequest {
                    action_id: action_id.clone(),
                    scoped_backend,
                    payload: payload.clone(),
                };
                let provider_response = provider
                    .do_action(&action_request)
                    .await
                    .map_err(|e| KeyStackError::ProviderError { source: e })?;

                Ok(KeyStackResponse::Action {
                    action_id: action_id.clone(),
                    pre_processor_context: pre_processor_results,
                    provider_response,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyPath, KeyStack, KeyStackRequest, KeyStackResponse};
    use libcrux_ed25519::verify;

    #[tokio::test(flavor = "current_thread")]
    async fn default_keystack_generates_key_and_signs_payload() {
        let keystack = KeyStack::default();
        let key_path: KeyPath = "test-key".into();

        let generate_response = keystack
            .handle_request(KeyStackRequest::Action {
                auth_data: None,
                user_id: None,
                key_path: key_path.clone(),
                pre_processor_ids: Vec::new(),
                action_id: "generate".to_string(),
                payload: Vec::new(),
                provider_id: "builtin-libcrux-ed25519".to_string(),
            })
            .await
            .expect("generate action should succeed");

        let generated_public_key = match generate_response {
            KeyStackResponse::Action {
                provider_response, ..
            } => provider_response,
        };

        assert_eq!(generated_public_key.len(), 32);

        let public_key: [u8; 32] = generated_public_key
            .try_into()
            .expect("generated public key should be 32 bytes");

        let payload = b"payload-to-sign".to_vec();

        let sign_response = keystack
            .handle_request(KeyStackRequest::Action {
                auth_data: None,
                user_id: None,
                key_path,
                pre_processor_ids: Vec::new(),
                action_id: "sign".to_string(),
                payload: payload.clone(),
                provider_id: "builtin-libcrux-ed25519".to_string(),
            })
            .await
            .expect("sign action should succeed");

        let signature = match sign_response {
            KeyStackResponse::Action {
                provider_response, ..
            } => provider_response,
        };

        assert_eq!(signature.len(), 64);

        let signature: [u8; 64] = signature.try_into().expect("signature should be 64 bytes");

        verify(&payload, &public_key, &signature).expect("signature should verify");
    }
}
