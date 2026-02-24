mod audio;
mod core;
mod hotkey;
mod injection;
mod processors;
mod settings;

use std::sync::Arc;

use core::orchestrator::DictationOrchestrator;
use hotkey::win32_alt_hook::start_alt_double_tap_listener;
use processors::deepgram::DeepgramProcessor;
use processors::gemini::GeminiCleaner;
use settings::commands::{
    clear_deepgram_api_key, clear_gemini_api_key, get_launch_on_startup_enabled,
    get_processing_enabled, has_deepgram_api_key, has_gemini_api_key, save_deepgram_api_key,
    save_gemini_api_key, set_launch_on_startup_enabled, set_processing_enabled, SettingsState,
};
use settings::stronghold_store::StrongholdStore;
use tauri::menu::MenuBuilder;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

const TRAY_ID: &str = "air_keys_tray";
const MENU_SETTINGS: &str = "settings";
const MENU_QUIT: &str = "quit";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None::<Vec<&str>>,
        ))
        .setup(|app| {
            let app_handle = app.handle().clone();

            if cfg!(debug_assertions) {
                app_handle.plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let key_store = Arc::new(StrongholdStore::new(&app_handle)?);
            let processor = Arc::new(DeepgramProcessor::new(key_store.clone()));
            let cleaner = Arc::new(GeminiCleaner::new(key_store.clone()));
            let orchestrator = Arc::new(DictationOrchestrator::new(
                app_handle.clone(),
                processor,
                cleaner,
                key_store.clone(),
            )?);
            app.manage(SettingsState::new(key_store.clone()));
            app.manage(orchestrator.clone());

            let menu = MenuBuilder::new(app)
                .text(MENU_SETTINGS, "Settings")
                .separator()
                .text(MENU_QUIT, "Quit")
                .build()?;

            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .menu(&menu)
                .tooltip("Air Keys - idle")
                .on_menu_event(move |app_handle, event| match event.id.as_ref() {
                    MENU_SETTINGS => {
                        if let Some(window) = app_handle.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    MENU_QUIT => {
                        app_handle.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.hide();
            }
            if let Some(window) = app.get_webview_window("recording") {
                let _ = window.hide();
            }

            start_alt_double_tap_listener(orchestrator.clone(), 400)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_deepgram_api_key,
            clear_deepgram_api_key,
            has_deepgram_api_key,
            save_gemini_api_key,
            clear_gemini_api_key,
            has_gemini_api_key,
            get_processing_enabled,
            set_processing_enabled,
            get_launch_on_startup_enabled,
            set_launch_on_startup_enabled
        ])
        .run(tauri::generate_context!())
        .expect("error while running air keys application");
}
