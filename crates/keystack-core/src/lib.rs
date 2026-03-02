use std::{collections::HashMap, hash::Hash, path::PathBuf, sync::Arc};

use snafu::Snafu;

use crate::{
    context_provider::ContextProviderError,
    crypto_provider::{CryptoProvider, CryptoProviderError},
    id_provider::IdentityProviderError,
    secret_provider::SecretProvider,
};

pub mod context_provider;
pub mod crypto_provider;
pub mod id_provider;
pub mod secret_provider;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyPath(PathBuf);

impl From<&str> for KeyPath {
    fn from(value: &str) -> Self {
        Self(PathBuf::from(value))
    }
}

#[derive(Debug, Snafu)]
pub enum KeyStackError {
    #[snafu(display("ContextProvider failed: {}", source))]
    ContextProviderError { source: ContextProviderError },
    #[snafu(display("Provider action failed: {}", source))]
    CryptoProviderError { source: CryptoProviderError },
    #[snafu(display("ContextProvider not found: {}", id))]
    ContextProviderNotFound { id: String },
    #[snafu(display("Provider not found: {}", id))]
    ProviderNotFound { id: String },
    #[snafu(display("Identity provider error: {}", source))]
    IdentityProvider { source: IdentityProviderError },
}

impl From<IdentityProviderError> for KeyStackError {
    fn from(source: IdentityProviderError) -> Self {
        KeyStackError::IdentityProvider { source }
    }
}

pub enum KeyStackRequest {
    Action {
        key_path: KeyPath,
        context_provider_ids: Vec<String>,
        action_id: String,
        payload: Vec<u8>,
        crypto_provider_id: String,
        auth_data: Option<Vec<u8>>,
        user_id: Option<String>,
    },
}

pub enum KeyStackResponse {
    Action {
        action_id: String,
        pre_action_context: HashMap<String, Vec<u8>>,
        provider_response: Vec<u8>,
    },
}

pub struct KeyStack {
    required_context_providers: Vec<String>,
    secret_provider: Arc<dyn SecretProvider>,
    context_providers: HashMap<String, Arc<dyn context_provider::ContextProvider>>,
    crypto_providers: HashMap<String, Arc<dyn crypto_provider::CryptoProvider>>,
    identity_manager: Arc<dyn id_provider::IdentityProvider>,
}

impl Default for KeyStack {
    fn default() -> Self {
        let ed25519_provider = crypto_provider::libcrux_ed25519::LibCruxEd25519Provider;

        let crypto_providers = HashMap::from([(
            ed25519_provider.name(),
            Arc::new(ed25519_provider) as Arc<dyn crypto_provider::CryptoProvider>,
        )]);

        Self {
            identity_manager: Arc::new(id_provider::disabled_id_provider::DisabledIdentityProvider),
            required_context_providers: Vec::new(),
            secret_provider: Arc::new(
                secret_provider::hashmap_secret_provider::HashMapSecretProvider {
                    store: std::sync::Mutex::new(HashMap::new()),
                },
            ),
            context_providers: HashMap::new(),
            crypto_providers,
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
                context_provider_ids,
                action_id,
                payload,
                crypto_provider_id,
                auth_data,
                user_id,
            } => {
                self.identity_manager
                    .user_authenticate(&user_id.clone().unwrap_or_default(), auth_data.as_deref())
                    .await?;

                let user = id_provider::User::new(
                    "default-user".to_string(),
                    self.identity_manager.clone(),
                );
                let all_context_provider_ids = self
                    .required_context_providers
                    .iter()
                    .chain(context_provider_ids.iter())
                    .cloned()
                    .collect::<Vec<_>>();

                let context = context_provider::ContextProviderContext {
                    user,
                    key_path: key_path.clone(),
                    action_id: action_id.clone(),
                    payload: payload.clone(),
                };

                let mut plugin_results = HashMap::new();
                for plugin_id in all_context_provider_ids {
                    let plugin = self.context_providers.get(&plugin_id).ok_or_else(|| {
                        KeyStackError::ContextProviderNotFound {
                            id: plugin_id.clone(),
                        }
                    })?;

                    let result = plugin
                        .pre_action_hook(&context)
                        .map_err(|e| KeyStackError::ContextProviderError { source: e })?;

                    plugin_results.insert(plugin_id, result);
                }

                let provider = self
                    .crypto_providers
                    .get(crypto_provider_id)
                    .ok_or_else(|| KeyStackError::ProviderNotFound {
                        id: crypto_provider_id.clone(),
                    })?;

                let scoped_secret_provider = secret_provider::ScopedSecretProvider::new(
                    self.secret_provider.clone(),
                    key_path.clone(),
                );

                let action_request = crypto_provider::ActionRequest {
                    action_id: action_id.clone(),
                    scoped_secret_provider,
                    payload: payload.clone(),
                };
                let provider_response = provider
                    .do_action(&action_request)
                    .await
                    .map_err(|e| KeyStackError::CryptoProviderError { source: e })?;

                Ok(KeyStackResponse::Action {
                    action_id: action_id.clone(),
                    pre_action_context: plugin_results,
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
                context_provider_ids: Vec::new(),
                action_id: "generate".to_string(),
                payload: Vec::new(),
                crypto_provider_id: "builtin-libcrux-ed25519".to_string(),
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
                context_provider_ids: Vec::new(),
                action_id: "sign".to_string(),
                payload: payload.clone(),
                crypto_provider_id: "builtin-libcrux-ed25519".to_string(),
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
