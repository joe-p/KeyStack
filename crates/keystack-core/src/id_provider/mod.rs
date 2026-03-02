use std::sync::Arc;

use async_trait::async_trait;
use snafu::Snafu;

pub mod disabled_id_provider;

#[derive(Debug, Snafu)]
pub enum IdentityProviderError {
    AuthenticationFailed,
    NotImplemented,
}

pub struct User {
    user_id: String,
    identity_provider: Arc<dyn IdentityProvider>,
}

impl User {
    pub fn new(user_id: String, identity_provider: Arc<dyn IdentityProvider>) -> Self {
        Self {
            user_id,
            identity_provider,
        }
    }

    pub async fn has_role(&self, role_id: &str) -> Result<bool, IdentityProviderError> {
        self.identity_provider
            .user_has_role(&self.user_id, role_id)
            .await
    }

    pub fn id(&self) -> &str {
        &self.user_id
    }
}

#[async_trait]
pub trait IdentityProvider {
    /// Used to authenticate that a user is who they say they are. The extra_data field can be used to pass additional information to the authentication process, such as a token.
    async fn authenticate_user(
        &self,
        user_id: &str,
        extra_data: Option<&[u8]>,
    ) -> Result<bool, IdentityProviderError>;

    async fn user_has_role(
        &self,
        user_id: &str,
        role_id: &str,
    ) -> Result<bool, IdentityProviderError>;
}
