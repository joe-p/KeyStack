use async_trait::async_trait;

use crate::id_provider::{IdentityProvider, IdentityProviderError};

pub struct DisabledIdentityProvider;

#[async_trait]
impl IdentityProvider for DisabledIdentityProvider {
    async fn authenticate_user(
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
