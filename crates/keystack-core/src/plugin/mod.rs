use snafu::Snafu;

use crate::{KeyPath, id_provider::User, plugin::wasm_pre_action_plugin::WasmPreActionPluginError};

pub mod wasm_pre_action_plugin;

#[derive(Debug, Snafu)]
pub enum PreActionPluginError {
    WasmPreActionPluginError { source: WasmPreActionPluginError },
}

impl From<WasmPreActionPluginError> for PreActionPluginError {
    fn from(source: WasmPreActionPluginError) -> Self {
        PreActionPluginError::WasmPreActionPluginError { source }
    }
}

pub struct PreActionPluginContext {
    pub user: User,
    pub key_path: KeyPath,
    pub action_id: String,
    pub payload: Vec<u8>,
}

pub trait PreActionPlugin {
    /// Called before an action is executed and returns additional context that will be added to
    /// the response.
    fn pre_action_hook(
        &self,
        context: &PreActionPluginContext,
    ) -> Result<Vec<u8>, PreActionPluginError>;
}
