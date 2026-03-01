use async_trait::async_trait;

use crate::id_provider::{IdentityProvider, IdentityProviderError};

pub struct DisabledIdentityProvider;

#[async_trait]
impl IdentityProvider for DisabledIdentityProvider {
    // Role management

    async fn role_create(&self, _role_id: &str) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }
    async fn role_delete(&self, _role_id: &str) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }

    // Group management

    async fn group_create(&self, _group_id: &str) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }

    async fn group_delete(&self, _group_id: &str) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }
    async fn group_add_role(
        &self,
        _group_id: &str,
        _role_id: &str,
    ) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }
    async fn group_remove_role(
        &self,
        _group_id: &str,
        _role_id: &str,
    ) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }

    // User management

    async fn user_create(&self, _user_id: &str) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }
    async fn user_delete(&self, _user_id: &str) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }
    async fn user_add_role(
        &self,
        _user_id: &str,
        _role_id: &str,
    ) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }
    async fn user_remove_role(
        &self,
        _user_id: &str,
        _role_id: &str,
    ) -> Result<(), IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }

    async fn user_authenticate(
        &self,
        user_id: &str,
        _extra_data: Option<&[u8]>,
    ) -> Result<bool, IdentityProviderError> {
        if user_id.is_empty() {
            return Ok(true);
        }
        return Ok(false);
    }

    async fn user_has_role(
        &self,
        _user_id: &str,
        _role_id: &str,
    ) -> Result<bool, IdentityProviderError> {
        return Err(IdentityProviderError::NotImplemented);
    }
}
