use std::sync::Arc;

use tauri::State;

use super::stronghold_store::SecureKeyStore;

pub struct SettingsState {
    store: Arc<dyn SecureKeyStore>,
}

impl SettingsState {
    pub fn new(store: Arc<dyn SecureKeyStore>) -> Self {
        Self { store }
    }
}

#[tauri::command]
pub async fn save_deepgram_api_key(
    state: State<'_, SettingsState>,
    api_key: String,
) -> Result<(), String> {
    state
        .store
        .save_deepgram_key(api_key.trim().to_string())
        .await
        .map_err(|err| format!("failed to save key: {err}"))
}

#[tauri::command]
pub async fn clear_deepgram_api_key(state: State<'_, SettingsState>) -> Result<(), String> {
    state
        .store
        .clear_deepgram_key()
        .await
        .map_err(|err| format!("failed to clear key: {err}"))
}

#[tauri::command]
pub async fn has_deepgram_api_key(state: State<'_, SettingsState>) -> Result<bool, String> {
    state
        .store
        .read_deepgram_key()
        .await
        .map(|value| value.is_some())
        .map_err(|err| format!("failed to read key status: {err}"))
}
