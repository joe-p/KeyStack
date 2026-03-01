use std::sync::Arc;

use async_trait::async_trait;
use snafu::Snafu;

pub mod disabled_id_manager;

#[derive(Debug, Snafu)]
pub enum IdentityManagerError {
    AuthenticationFailed,
    NotImplemented,
}

pub struct User {
    user_id: String,
    identity_manager: Arc<dyn IdentityManager>,
}

impl User {
    pub fn new(user_id: String, identity_manager: Arc<dyn IdentityManager>) -> Self {
        Self {
            user_id,
            identity_manager,
        }
    }

    pub async fn user_has_role(&self, role_id: &str) -> Result<bool, IdentityManagerError> {
        self.identity_manager
            .user_has_role(&self.user_id, role_id)
            .await
    }

    pub fn id(&self) -> &str {
        &self.user_id
    }
}

#[async_trait]
pub trait IdentityManager {
    // Role management

    async fn role_create(&self, role_id: &str) -> Result<(), IdentityManagerError>;
    async fn role_delete(&self, role_id: &str) -> Result<(), IdentityManagerError>;

    // Group management

    async fn group_create(&self, group_id: &str) -> Result<(), IdentityManagerError>;
    async fn group_delete(&self, group_id: &str) -> Result<(), IdentityManagerError>;
    async fn group_add_role(
        &self,
        group_id: &str,
        role_id: &str,
    ) -> Result<(), IdentityManagerError>;
    async fn group_remove_role(
        &self,
        group_id: &str,
        role_id: &str,
    ) -> Result<(), IdentityManagerError>;

    // User management

    async fn user_create(&self, id: &str) -> Result<(), IdentityManagerError>;
    async fn user_delete(&self, id: &str) -> Result<(), IdentityManagerError>;
    async fn user_add_role(&self, user_id: &str, role_id: &str)
    -> Result<(), IdentityManagerError>;
    async fn user_remove_role(
        &self,
        user_id: &str,
        role_id: &str,
    ) -> Result<(), IdentityManagerError>;

    /// Used to authenticate that a user is who they say they are. The extra_data field can be used to pass additional information to the authentication process, such as a token.
    async fn user_authenticate(
        &self,
        user_id: &str,
        extra_data: Option<&[u8]>,
    ) -> Result<bool, IdentityManagerError>;

    async fn user_has_role(
        &self,
        user_id: &str,
        role_id: &str,
    ) -> Result<bool, IdentityManagerError>;
}
