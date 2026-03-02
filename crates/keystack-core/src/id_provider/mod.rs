use std::sync::Arc;

use async_trait::async_trait;
use snafu::Snafu;

pub mod disabled_id_provider;

#[derive(Debug, Snafu)]
pub enum IdentityProviderError {
    AuthenticationFailed,
    NotImplemented,
}

#[async_trait]
pub trait AuthenticatedUser {
    fn id(&self) -> &str;
    async fn has_role(&self, role_id: &str) -> Result<bool, IdentityProviderError>;
}

#[async_trait]
pub trait IdentityProvider {
    /// Used to authenticate that a user is who they say they are. The extra_data field can be used to pass additional information to the authentication process, such as a token.
    async fn authenticate_user(
        &self,
        user_id: &str,
        extra_data: Option<&[u8]>,
    ) -> Result<Arc<dyn AuthenticatedUser>, IdentityProviderError>;
}
