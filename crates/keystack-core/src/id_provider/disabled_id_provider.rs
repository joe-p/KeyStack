use std::sync::Arc;

use async_trait::async_trait;

use crate::id_provider::{AuthenticatedUser, IdentityProvider, IdentityProviderError};

pub struct EmptyUser {}

#[async_trait]
impl AuthenticatedUser for EmptyUser {
    fn id(&self) -> &str {
        ""
    }

    async fn has_role(&self, _role_id: &str) -> Result<bool, IdentityProviderError> {
        Ok(false)
    }
}

pub struct DisabledIdentityProvider;

#[async_trait]
impl IdentityProvider for DisabledIdentityProvider {
    async fn authenticate_user(
        &self,
        user_id: &str,
        _extra_data: Option<&[u8]>,
    ) -> Result<Arc<dyn AuthenticatedUser>, IdentityProviderError> {
        if user_id.is_empty() {
            return Ok(Arc::new(EmptyUser {}));
        }
        Err(IdentityProviderError::AuthenticationFailed)
    }
}
