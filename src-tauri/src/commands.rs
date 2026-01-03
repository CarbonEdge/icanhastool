//! Tauri IPC commands module.
//!
//! Exposes Rust functionality to the Svelte frontend via Tauri commands.

use crate::audio::{AudioCapture, AudioDeviceInfo, CpalAudioCapture};
use crate::claude::{ClaudeCodeProcess, ClaudeProcess, ProcessStatus};
use crate::vosk_stt::{ModelInfo, ModelManager, RecognitionResult, SpeechRecognizer, VoskRecognizer};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Application state shared across commands
pub struct AppState {
    pub audio: Arc<dyn AudioCapture>,
    pub recognizer: Arc<dyn SpeechRecognizer>,
    pub claude: Arc<dyn ClaudeProcess>,
    pub model_manager: ModelManager,
    audio_callback: Mutex<Option<Arc<dyn Fn(Vec<i16>) + Send + Sync>>>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let models_dir = app_data_dir.join("models");

        Self {
            audio: Arc::new(CpalAudioCapture::new()),
            recognizer: Arc::new(VoskRecognizer::new()),
            claude: Arc::new(ClaudeCodeProcess::new()),
            model_manager: ModelManager::new(models_dir),
            audio_callback: Mutex::new(None),
        }
    }

    #[cfg(test)]
    pub fn with_mocks(
        audio: Arc<dyn AudioCapture>,
        recognizer: Arc<dyn SpeechRecognizer>,
        claude: Arc<dyn ClaudeProcess>,
    ) -> Self {
        Self {
            audio,
            recognizer,
            claude,
            model_manager: ModelManager::new(PathBuf::from("/test/models")),
            audio_callback: Mutex::new(None),
        }
    }
}

// ============================================================================
// Audio Commands
// ============================================================================

#[tauri::command]
pub fn list_audio_devices(state: State<AppState>) -> Result<Vec<AudioDeviceInfo>, String> {
    state.audio.list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_recording(
    app: AppHandle,
    state: State<AppState>,
    device_name: Option<String>,
) -> Result<(), String> {
    let recognizer = state.recognizer.clone();

    // Create callback that processes audio through Vosk and emits events
    let callback: Arc<dyn Fn(Vec<i16>) + Send + Sync> = Arc::new(move |samples| {
        match recognizer.process_audio(&samples) {
            Ok(Some(result)) => {
                let _ = app.emit("transcription", &result);
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Recognition error: {}", e);
            }
        }
    });

    *state.audio_callback.lock() = Some(callback.clone());

    state
        .audio
        .start_recording(device_name.as_deref(), callback)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_recording(app: AppHandle, state: State<AppState>) -> Result<RecognitionResult, String> {
    state.audio.stop_recording();
    *state.audio_callback.lock() = None;

    // Get final transcription
    let result = state.recognizer.get_final_result().map_err(|e| e.to_string())?;

    // Emit final result
    let _ = app.emit("transcription-final", &result);

    Ok(result)
}

#[tauri::command]
pub fn is_recording(state: State<AppState>) -> bool {
    state.audio.is_recording()
}

// ============================================================================
// Speech Recognition Commands
// ============================================================================

#[tauri::command]
pub fn list_models(state: State<AppState>) -> Vec<ModelInfo> {
    state.model_manager.list_available_models()
}

#[tauri::command]
pub fn list_installed_models(state: State<AppState>) -> Vec<ModelInfo> {
    state.model_manager.list_installed_models()
}

#[tauri::command]
pub fn load_model(state: State<AppState>, model_path: String) -> Result<(), String> {
    state
        .recognizer
        .load_model(std::path::Path::new(&model_path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_model_loaded(state: State<AppState>) -> bool {
    state.recognizer.is_model_loaded()
}

#[tauri::command]
pub fn reset_recognizer(state: State<AppState>) {
    eprintln!("[DEBUG] reset_recognizer called");
    state.recognizer.reset();
    eprintln!("[DEBUG] reset_recognizer completed");
}

// ============================================================================
// Claude Code Commands
// ============================================================================

#[tauri::command]
pub fn start_claude(
    app: AppHandle,
    state: State<AppState>,
    working_dir: Option<String>,
) -> Result<(), String> {
    // Set up output callback to emit events
    let app_clone = app.clone();
    state.claude.set_output_callback(Arc::new(move |event| {
        let _ = app_clone.emit("claude-output", &event);
    }));

    state
        .claude
        .start(working_dir.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_claude(state: State<AppState>) -> Result<(), String> {
    state.claude.stop().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn send_to_claude(state: State<AppState>, input: String) -> Result<(), String> {
    // Send input directly - xterm.js already sends appropriate characters
    // (Enter sends \r, arrow keys send escape sequences like \x1b[A, etc.)
    state
        .claude
        .send_input(&input)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resize_claude(state: State<AppState>, cols: u16, rows: u16) -> Result<(), String> {
    state.claude.resize(cols, rows).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn claude_status(state: State<AppState>) -> ProcessStatus {
    state.claude.status()
}

// ============================================================================
// Utility Commands
// ============================================================================

#[tauri::command]
pub fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "icanhastool",
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::tests::MockAudioCapture;
    use crate::claude::tests::MockClaudeProcess;
    use crate::vosk_stt::tests::MockSpeechRecognizer;

    fn create_test_state() -> AppState {
        AppState::with_mocks(
            Arc::new(MockAudioCapture::new()),
            Arc::new(MockSpeechRecognizer::new()),
            Arc::new(MockClaudeProcess::new()),
        )
    }

    #[test]
    fn test_app_state_creation() {
        let state = create_test_state();
        assert!(!state.audio.is_recording());
        assert!(!state.recognizer.is_model_loaded());
        assert_eq!(state.claude.status(), ProcessStatus::Stopped);
    }

    #[test]
    fn test_model_manager_in_state() {
        let state = create_test_state();
        let models = state.model_manager.list_available_models();
        assert!(!models.is_empty());
    }

    #[test]
    fn test_audio_callback_storage() {
        let state = create_test_state();

        assert!(state.audio_callback.lock().is_none());

        let callback: Arc<dyn Fn(Vec<i16>) + Send + Sync> = Arc::new(|_| {});
        *state.audio_callback.lock() = Some(callback);

        assert!(state.audio_callback.lock().is_some());

        *state.audio_callback.lock() = None;
        assert!(state.audio_callback.lock().is_none());
    }

    #[test]
    fn test_get_app_info() {
        let info = get_app_info();

        assert_eq!(info["name"], "icanhastool");
        assert!(info["version"].is_string());
        assert!(info["description"].is_string());
    }
}
