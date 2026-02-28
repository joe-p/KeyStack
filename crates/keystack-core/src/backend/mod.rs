use std::sync::Arc;

use super::KeyPath;
use async_trait::async_trait;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum BackendError {
    #[snafu(display("Key not found: {}", path.0.to_string_lossy()))]
    KeyNotFound { path: KeyPath },
    #[snafu(display("Key already exists: {}", path.0.to_string_lossy()))]
    AlreadyExists { path: KeyPath },
}

pub mod hashmap_backend;

#[async_trait]
pub trait Backend: Send + Sync {
    async fn read(&self, path: &KeyPath, destination: &mut [u8]) -> Result<usize, BackendError>;
    async fn create(&self, path: &KeyPath, data: &[u8]) -> Result<(), BackendError>;
    async fn update(&self, path: &KeyPath, data: &[u8]) -> Result<(), BackendError>;
    async fn delete(&self, path: &KeyPath) -> Result<(), BackendError>;
}

pub struct ScopedBackend {
    backend: Arc<dyn Backend>,
    pub scope: KeyPath,
}

impl ScopedBackend {
    pub fn new(backend: Arc<dyn Backend>, scope: KeyPath) -> Self {
        Self { backend, scope }
    }

    pub async fn read(
        &self,
        child_path: &KeyPath,
        destination: &mut [u8],
    ) -> Result<usize, BackendError> {
        let full_path = self.scope.0.join(&child_path.0);
        self.backend.read(&KeyPath(full_path), destination).await
    }

    pub async fn create(&self, child_path: &KeyPath, data: &[u8]) -> Result<(), BackendError> {
        let full_path = self.scope.0.join(&child_path.0);
        self.backend.create(&KeyPath(full_path), data).await
    }

    pub async fn update(&self, child_path: &KeyPath, data: &[u8]) -> Result<(), BackendError> {
        let full_path = self.scope.0.join(&child_path.0);
        self.backend.update(&KeyPath(full_path), data).await
    }

    pub async fn delete(&self, child_path: &KeyPath) -> Result<(), BackendError> {
        let full_path = self.scope.0.join(&child_path.0);
        self.backend.delete(&KeyPath(full_path)).await
    }
}
