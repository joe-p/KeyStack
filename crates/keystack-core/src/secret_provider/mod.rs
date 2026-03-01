use std::sync::Arc;

use super::KeyPath;
use async_trait::async_trait;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum SecretProviderError {
    #[snafu(display("Key not found: {}", path.0.to_string_lossy()))]
    KeyNotFound { path: KeyPath },
    #[snafu(display("Key already exists: {}", path.0.to_string_lossy()))]
    AlreadyExists { path: KeyPath },
}

pub mod hashmap_secret_provider;

#[async_trait]
pub trait SecretProvider: Send + Sync {
    async fn read(
        &self,
        path: &KeyPath,
        destination: &mut [u8],
    ) -> Result<usize, SecretProviderError>;
    async fn create(&self, path: &KeyPath, data: &[u8]) -> Result<(), SecretProviderError>;
    async fn update(&self, path: &KeyPath, data: &[u8]) -> Result<(), SecretProviderError>;
    async fn delete(&self, path: &KeyPath) -> Result<(), SecretProviderError>;
}

pub struct ScopedSecretProvider {
    secret_provider: Arc<dyn SecretProvider>,
    pub scope: KeyPath,
}

impl ScopedSecretProvider {
    pub fn new(secret_provider: Arc<dyn SecretProvider>, scope: KeyPath) -> Self {
        Self {
            secret_provider,
            scope,
        }
    }

    pub async fn read(
        &self,
        child_path: &KeyPath,
        destination: &mut [u8],
    ) -> Result<usize, SecretProviderError> {
        let full_path = self.scope.0.join(&child_path.0);
        self.secret_provider
            .read(&KeyPath(full_path), destination)
            .await
    }

    pub async fn create(
        &self,
        child_path: &KeyPath,
        data: &[u8],
    ) -> Result<(), SecretProviderError> {
        let full_path = self.scope.0.join(&child_path.0);
        self.secret_provider.create(&KeyPath(full_path), data).await
    }

    pub async fn update(
        &self,
        child_path: &KeyPath,
        data: &[u8],
    ) -> Result<(), SecretProviderError> {
        let full_path = self.scope.0.join(&child_path.0);
        self.secret_provider.update(&KeyPath(full_path), data).await
    }

    pub async fn delete(&self, child_path: &KeyPath) -> Result<(), SecretProviderError> {
        let full_path = self.scope.0.join(&child_path.0);
        self.secret_provider.delete(&KeyPath(full_path)).await
    }
}
