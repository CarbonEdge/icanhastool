//! icanhastool - Voice interface for Claude Code
//!
//! A cross-platform desktop application that provides voice input
//! to Claude Code using local speech recognition (Vosk).

pub mod audio;
pub mod claude;
pub mod commands;
pub mod vosk_stt;

use commands::AppState;
use std::path::PathBuf;
use tauri::Manager;

/// Get the app data directory for storing models and settings
fn get_app_data_dir(app: &tauri::App) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Failed to get app data directory")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_data_dir = get_app_data_dir(app);

            // Ensure models directory exists
            let models_dir = app_data_dir.join("models");
            std::fs::create_dir_all(&models_dir).ok();

            // Initialize app state
            let state = AppState::new(app_data_dir);
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_audio_devices,
            commands::start_recording,
            commands::stop_recording,
            commands::is_recording,
            commands::list_models,
            commands::list_installed_models,
            commands::load_model,
            commands::is_model_loaded,
            commands::start_claude,
            commands::stop_claude,
            commands::send_to_claude,
            commands::resize_claude,
            commands::claude_status,
            commands::get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modules_compile() {
        // This test ensures all modules compile correctly
        // The actual functionality is tested in each module's tests
        assert!(true);
    }
}
