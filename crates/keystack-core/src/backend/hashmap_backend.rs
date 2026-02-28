use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;

use crate::{
    KeyPath,
    backend::{Backend, BackendError},
};

pub struct HashMapBackend {
    pub store: Mutex<HashMap<String, Vec<u8>>>,
}

#[async_trait]
impl Backend for HashMapBackend {
    async fn read(&self, path: &KeyPath, destination: &mut [u8]) -> Result<usize, BackendError> {
        let store = self.store.lock().unwrap();
        if let Some(data) = store.get(&path.0.to_string_lossy().to_string()) {
            let len = data.len().min(destination.len());
            destination[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            Err(BackendError::KeyNotFound { path: path.clone() })
        }
    }

    async fn create(&self, path: &KeyPath, data: &[u8]) -> Result<(), BackendError> {
        let mut store = self.store.lock().unwrap();

        if store.contains_key(&path.0.to_string_lossy().to_string()) {
            return Err(BackendError::AlreadyExists { path: path.clone() });
        }

        store.insert(path.0.to_string_lossy().to_string(), data.to_vec());
        Ok(())
    }

    async fn update(&self, path: &KeyPath, data: &[u8]) -> Result<(), BackendError> {
        let mut store = self.store.lock().unwrap();
        if let std::collections::hash_map::Entry::Occupied(mut e) =
            store.entry(path.0.to_string_lossy().to_string())
        {
            e.insert(data.to_vec());
            Ok(())
        } else {
            Err(BackendError::KeyNotFound { path: path.clone() })
        }
    }

    async fn delete(&self, path: &KeyPath) -> Result<(), BackendError> {
        let mut store = self.store.lock().unwrap();
        if store
            .remove(&path.0.to_string_lossy().to_string())
            .is_some()
        {
            Ok(())
        } else {
            Err(BackendError::KeyNotFound { path: path.clone() })
        }
    }
}
