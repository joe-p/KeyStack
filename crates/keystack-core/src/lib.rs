use std::{collections::HashMap, hash::Hash, path::PathBuf, sync::Arc};

use snafu::Snafu;

use crate::{backend::Backend, processor::PreProcessorError, provider::ProviderError};

pub mod backend;
pub mod processor;
pub mod provider;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyPath(PathBuf);

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
}

pub enum KeyStackRequest {
    Action {
        key_path: KeyPath,
        pre_processor_ids: Vec<String>,
        action_id: String,
        payload: Vec<u8>,
        provider_id: String,
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
            } => {
                let all_pre_processor_ids = self
                    .required_pre_processors
                    .iter()
                    .chain(pre_processor_ids.iter())
                    .cloned()
                    .collect::<Vec<_>>();

                let context = processor::PreProcessContext {
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
