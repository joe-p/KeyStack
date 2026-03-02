use async_trait::async_trait;

pub struct UserError {}

#[async_trait]
pub trait User {
    fn id(&self) -> &str;
    async fn has_role(&self, role_id: &str) -> Result<bool, UserError>;
}
