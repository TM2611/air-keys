use std::sync::Arc;

use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

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

#[tauri::command]
pub fn get_launch_on_startup_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|err| format!("failed to read launch on startup status: {err}"))
}

#[tauri::command]
pub fn set_launch_on_startup_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch
            .enable()
            .map_err(|err| format!("failed to enable launch on startup: {err}"))
    } else {
        autolaunch
            .disable()
            .map_err(|err| format!("failed to disable launch on startup: {err}"))
    }
}
