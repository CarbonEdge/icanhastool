//! Vosk speech-to-text module.
//!
//! Handles speech recognition using the Vosk library.
//! Requires a Vosk model to be downloaded and configured.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

/// Speech recognition errors
#[derive(Error, Debug)]
pub enum SpeechError {
    #[error("Model not found at path: {0}")]
    ModelNotFound(String),
    #[error("Failed to initialize model: {0}")]
    ModelInitError(String),
    #[error("Failed to create recognizer: {0}")]
    RecognizerError(String),
    #[error("Recognition failed: {0}")]
    RecognitionError(String),
    #[error("Model download failed: {0}")]
    DownloadError(String),
}

/// Vosk model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub language: String,
    pub size_mb: u64,
}

/// Speech recognition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionResult {
    pub text: String,
    pub is_final: bool,
    pub confidence: Option<f32>,
}

/// Trait for speech recognition abstraction (enables testing)
pub trait SpeechRecognizer: Send + Sync {
    fn load_model(&self, model_path: &Path) -> Result<(), SpeechError>;
    fn is_model_loaded(&self) -> bool;
    fn process_audio(&self, samples: &[i16]) -> Result<Option<RecognitionResult>, SpeechError>;
    fn get_final_result(&self) -> Result<RecognitionResult, SpeechError>;
    fn reset(&self);
}

/// Real Vosk recognizer implementation
pub struct VoskRecognizer {
    model: Mutex<Option<vosk::Model>>,
    recognizer: Mutex<Option<vosk::Recognizer>>,
    sample_rate: f32,
}

impl VoskRecognizer {
    pub fn new() -> Self {
        Self {
            model: Mutex::new(None),
            recognizer: Mutex::new(None),
            sample_rate: 16000.0,
        }
    }

    pub fn with_sample_rate(sample_rate: f32) -> Self {
        Self {
            model: Mutex::new(None),
            recognizer: Mutex::new(None),
            sample_rate,
        }
    }

    fn parse_result(json: &str) -> RecognitionResult {
        #[derive(Deserialize)]
        struct VoskResult {
            text: Option<String>,
            partial: Option<String>,
        }

        let parsed: VoskResult = serde_json::from_str(json).unwrap_or(VoskResult {
            text: None,
            partial: None,
        });

        let (text, is_final) = if let Some(text) = parsed.text {
            (text, true)
        } else if let Some(partial) = parsed.partial {
            (partial, false)
        } else {
            (String::new(), false)
        };

        RecognitionResult {
            text,
            is_final,
            confidence: None,
        }
    }
}

impl Default for VoskRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeechRecognizer for VoskRecognizer {
    fn load_model(&self, model_path: &Path) -> Result<(), SpeechError> {
        if !model_path.exists() {
            return Err(SpeechError::ModelNotFound(
                model_path.to_string_lossy().to_string(),
            ));
        }

        let model = vosk::Model::new(model_path.to_string_lossy().as_ref())
            .ok_or_else(|| SpeechError::ModelInitError("Failed to load Vosk model".to_string()))?;

        let recognizer = vosk::Recognizer::new(&model, self.sample_rate)
            .ok_or_else(|| SpeechError::RecognizerError("Failed to create recognizer".to_string()))?;

        *self.model.lock() = Some(model);
        *self.recognizer.lock() = Some(recognizer);

        Ok(())
    }

    fn is_model_loaded(&self) -> bool {
        self.model.lock().is_some()
    }

    fn process_audio(&self, samples: &[i16]) -> Result<Option<RecognitionResult>, SpeechError> {
        let mut recognizer_guard = self.recognizer.lock();
        let recognizer = recognizer_guard
            .as_mut()
            .ok_or_else(|| SpeechError::RecognizerError("Recognizer not initialized".to_string()))?;

        let state = recognizer.accept_waveform(samples);

        match state {
            vosk::DecodingState::Running => {
                let partial = recognizer.partial_result();
                let result = Self::parse_result(partial.as_str());
                if result.text.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(result))
                }
            }
            vosk::DecodingState::Finalized => {
                let final_result = recognizer.result();
                let result = Self::parse_result(final_result.single().map(|r| r.text).unwrap_or(""));
                Ok(Some(RecognitionResult {
                    text: result.text,
                    is_final: true,
                    confidence: None,
                }))
            }
            vosk::DecodingState::Failed => {
                Err(SpeechError::RecognitionError("Decoding failed".to_string()))
            }
        }
    }

    fn get_final_result(&self) -> Result<RecognitionResult, SpeechError> {
        let mut recognizer_guard = self.recognizer.lock();
        let recognizer = recognizer_guard
            .as_mut()
            .ok_or_else(|| SpeechError::RecognizerError("Recognizer not initialized".to_string()))?;

        let final_result = recognizer.final_result();
        let text = final_result.single().map(|r| r.text.to_string()).unwrap_or_default();

        Ok(RecognitionResult {
            text,
            is_final: true,
            confidence: None,
        })
    }

    fn reset(&self) {
        if let Some(recognizer) = self.recognizer.lock().as_mut() {
            recognizer.reset();
        }
    }
}

/// Model manager for downloading and managing Vosk models
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    pub fn get_models_dir(&self) -> &Path {
        &self.models_dir
    }

    pub fn list_available_models(&self) -> Vec<ModelInfo> {
        // These are the most common Vosk models
        vec![
            ModelInfo {
                name: "vosk-model-small-en-us-0.15".to_string(),
                path: self.models_dir.join("vosk-model-small-en-us-0.15"),
                language: "English (US)".to_string(),
                size_mb: 40,
            },
            ModelInfo {
                name: "vosk-model-en-us-0.22".to_string(),
                path: self.models_dir.join("vosk-model-en-us-0.22"),
                language: "English (US)".to_string(),
                size_mb: 1800,
            },
        ]
    }

    pub fn list_installed_models(&self) -> Vec<ModelInfo> {
        self.list_available_models()
            .into_iter()
            .filter(|m| m.path.exists())
            .collect()
    }

    pub fn get_default_model(&self) -> Option<ModelInfo> {
        self.list_installed_models().into_iter().next()
    }

    pub fn ensure_models_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.models_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// Mock speech recognizer for testing
    pub struct MockSpeechRecognizer {
        model_loaded: AtomicBool,
        process_count: AtomicUsize,
        mock_text: Mutex<String>,
        should_fail: AtomicBool,
    }

    impl MockSpeechRecognizer {
        pub fn new() -> Self {
            Self {
                model_loaded: AtomicBool::new(false),
                process_count: AtomicUsize::new(0),
                mock_text: Mutex::new("Hello world".to_string()),
                should_fail: AtomicBool::new(false),
            }
        }

        pub fn set_mock_text(&self, text: &str) {
            *self.mock_text.lock() = text.to_string();
        }

        pub fn set_should_fail(&self, fail: bool) {
            self.should_fail.store(fail, Ordering::SeqCst);
        }

        pub fn process_count(&self) -> usize {
            self.process_count.load(Ordering::SeqCst)
        }
    }

    impl SpeechRecognizer for MockSpeechRecognizer {
        fn load_model(&self, model_path: &Path) -> Result<(), SpeechError> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(SpeechError::ModelNotFound(
                    model_path.to_string_lossy().to_string(),
                ));
            }
            self.model_loaded.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn is_model_loaded(&self) -> bool {
            self.model_loaded.load(Ordering::SeqCst)
        }

        fn process_audio(&self, _samples: &[i16]) -> Result<Option<RecognitionResult>, SpeechError> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(SpeechError::RecognitionError("Mock error".to_string()));
            }

            self.process_count.fetch_add(1, Ordering::SeqCst);

            let text = self.mock_text.lock().clone();
            if text.is_empty() {
                Ok(None)
            } else {
                Ok(Some(RecognitionResult {
                    text,
                    is_final: false,
                    confidence: Some(0.95),
                }))
            }
        }

        fn get_final_result(&self) -> Result<RecognitionResult, SpeechError> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(SpeechError::RecognizerError("Mock error".to_string()));
            }

            Ok(RecognitionResult {
                text: self.mock_text.lock().clone(),
                is_final: true,
                confidence: Some(0.98),
            })
        }

        fn reset(&self) {
            self.process_count.store(0, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_mock_load_model() {
        let recognizer = MockSpeechRecognizer::new();
        assert!(!recognizer.is_model_loaded());

        recognizer.load_model(Path::new("/test/model")).unwrap();
        assert!(recognizer.is_model_loaded());
    }

    #[test]
    fn test_mock_load_model_fails() {
        let recognizer = MockSpeechRecognizer::new();
        recognizer.set_should_fail(true);

        let result = recognizer.load_model(Path::new("/test/model"));
        assert!(matches!(result, Err(SpeechError::ModelNotFound(_))));
    }

    #[test]
    fn test_mock_process_audio() {
        let recognizer = MockSpeechRecognizer::new();
        recognizer.load_model(Path::new("/test/model")).unwrap();

        let samples = vec![0i16; 1600]; // 100ms of audio at 16kHz
        let result = recognizer.process_audio(&samples).unwrap();

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.text, "Hello world");
        assert!(!result.is_final);
        assert_eq!(recognizer.process_count(), 1);
    }

    #[test]
    fn test_mock_process_audio_empty() {
        let recognizer = MockSpeechRecognizer::new();
        recognizer.set_mock_text("");

        let samples = vec![0i16; 1600];
        let result = recognizer.process_audio(&samples).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_mock_get_final_result() {
        let recognizer = MockSpeechRecognizer::new();
        recognizer.set_mock_text("Final transcription");

        let result = recognizer.get_final_result().unwrap();

        assert_eq!(result.text, "Final transcription");
        assert!(result.is_final);
        assert_eq!(result.confidence, Some(0.98));
    }

    #[test]
    fn test_mock_reset() {
        let recognizer = MockSpeechRecognizer::new();

        let samples = vec![0i16; 1600];
        recognizer.process_audio(&samples).unwrap();
        recognizer.process_audio(&samples).unwrap();
        assert_eq!(recognizer.process_count(), 2);

        recognizer.reset();
        assert_eq!(recognizer.process_count(), 0);
    }

    #[test]
    fn test_model_manager_list_available() {
        let manager = ModelManager::new(PathBuf::from("/test/models"));
        let models = manager.list_available_models();

        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.name.contains("small-en-us")));
    }

    #[test]
    fn test_recognition_result_serialization() {
        let result = RecognitionResult {
            text: "Test text".to_string(),
            is_final: true,
            confidence: Some(0.95),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Test text"));
        assert!(json.contains("true"));
        assert!(json.contains("0.95"));

        let deserialized: RecognitionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text, result.text);
        assert_eq!(deserialized.is_final, result.is_final);
    }

    #[test]
    fn test_speech_error_display() {
        let err = SpeechError::ModelNotFound("/path/to/model".to_string());
        assert!(err.to_string().contains("/path/to/model"));

        let err = SpeechError::RecognizerError("init failed".to_string());
        assert!(err.to_string().contains("init failed"));
    }

    #[test]
    fn test_model_info_serialization() {
        let info = ModelInfo {
            name: "test-model".to_string(),
            path: PathBuf::from("/test/path"),
            language: "English".to_string(),
            size_mb: 100,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("English"));
    }
}
