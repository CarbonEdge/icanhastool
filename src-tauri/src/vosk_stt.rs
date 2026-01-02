//! Vosk speech-to-text module.
//!
//! Handles speech recognition using the Vosk library.
//! Requires a Vosk model to be downloaded and configured.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
                let result = Self::parse_result(partial.partial);
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
    additional_dirs: Vec<PathBuf>,
}

impl ModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        // Also check for models in common development locations
        let mut additional_dirs = Vec::new();

        eprintln!("[DEBUG] ModelManager::new - Primary dir: {:?}", models_dir);

        // Helper to add a models dir if it exists and isn't already added
        let mut try_add_models_dir = |dir: PathBuf| {
            eprintln!("[DEBUG] Checking dir: {:?}, exists: {}", dir, dir.exists());
            if dir.exists() && dir != models_dir && !additional_dirs.contains(&dir) {
                eprintln!("[DEBUG] Adding additional dir: {:?}", dir);
                additional_dirs.push(dir);
            }
        };

        // Check current working directory's models folder
        if let Ok(cwd) = std::env::current_dir() {
            try_add_models_dir(cwd.join("models"));

            // Also check parent directory (for when CWD is src-tauri)
            if let Some(parent) = cwd.parent() {
                try_add_models_dir(parent.join("models"));
            }
        }

        // Check executable directory's models folder
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                try_add_models_dir(exe_dir.join("models"));

                // Check parent directories (for target/debug -> src-tauri -> project root)
                if let Some(parent1) = exe_dir.parent() {
                    try_add_models_dir(parent1.join("models"));
                    if let Some(parent2) = parent1.parent() {
                        try_add_models_dir(parent2.join("models"));
                        if let Some(parent3) = parent2.parent() {
                            try_add_models_dir(parent3.join("models"));
                        }
                    }
                }
            }
        }

        Self { models_dir, additional_dirs }
    }

    /// Create a ModelManager that only scans the specified directory (for testing)
    #[cfg(test)]
    pub fn new_isolated(models_dir: PathBuf) -> Self {
        Self {
            models_dir,
            additional_dirs: Vec::new(),
        }
    }

    pub fn get_models_dir(&self) -> &Path {
        &self.models_dir
    }

    /// Get all directories where models are searched
    pub fn get_all_model_dirs(&self) -> Vec<&Path> {
        let mut dirs = vec![self.models_dir.as_path()];
        for dir in &self.additional_dirs {
            dirs.push(dir.as_path());
        }
        dirs
    }

    /// Check if a directory contains a valid Vosk model
    fn is_valid_vosk_model(path: &Path) -> bool {
        if !path.is_dir() {
            return false;
        }
        // Vosk models typically contain these indicators
        let has_am = path.join("am").is_dir() || path.join("am").with_extension("mdl").exists();
        let has_conf = path.join("conf").is_dir();
        let has_graph = path.join("graph").is_dir();
        let has_model_conf = path.join("mfcc.conf").exists() || path.join("conf/mfcc.conf").exists();

        // A valid model has at least am + graph, or model.conf files
        (has_am && has_graph) || (has_conf && has_graph) || has_model_conf
    }

    /// Detect language from model folder name
    fn detect_language(name: &str) -> String {
        let name_lower = name.to_lowercase();
        // Use word boundaries with dashes/underscores for more accurate matching
        if name_lower.contains("-en-us") || name_lower.contains("_en_us") || name_lower.contains("-en-us-") {
            "English (US)".to_string()
        } else if name_lower.contains("-en-in") || name_lower.contains("_en_in") {
            "English (India)".to_string()
        } else if name_lower.contains("-en-") || name_lower.contains("_en_") || name_lower.ends_with("-en") {
            "English".to_string()
        } else if name_lower.contains("-de-") || name_lower.contains("_de_") || name_lower.ends_with("-de") || name_lower.contains("-de-") {
            "German".to_string()
        } else if name_lower.contains("-fr-") || name_lower.contains("_fr_") || name_lower.ends_with("-fr") || name_lower.contains("-fr-") {
            "French".to_string()
        } else if name_lower.contains("-es-") || name_lower.contains("_es_") || name_lower.ends_with("-es") || name_lower.contains("-es-") {
            "Spanish".to_string()
        } else if name_lower.contains("-cn-") || name_lower.contains("-zh-") || name_lower.contains("_cn_") || name_lower.contains("_zh_") {
            "Chinese".to_string()
        } else if name_lower.contains("-ru-") || name_lower.contains("_ru_") || name_lower.ends_with("-ru") {
            "Russian".to_string()
        } else if name_lower.contains("-ja-") || name_lower.contains("-jp-") || name_lower.contains("_ja_") || name_lower.contains("_jp_") {
            "Japanese".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Calculate directory size in MB
    fn get_dir_size_mb(path: &Path) -> u64 {
        fn dir_size(path: &Path) -> u64 {
            let mut size = 0;
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        size += dir_size(&path);
                    } else if let Ok(meta) = entry.metadata() {
                        size += meta.len();
                    }
                }
            }
            size
        }
        dir_size(path) / (1024 * 1024)
    }

    pub fn list_available_models(&self) -> Vec<ModelInfo> {
        // Return suggestions for downloadable models
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
        let mut models = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        eprintln!("[DEBUG] list_installed_models - scanning {} dirs", self.get_all_model_dirs().len());

        // Scan all model directories
        for dir in self.get_all_model_dirs() {
            eprintln!("[DEBUG] Scanning: {:?}", dir);
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let is_valid = Self::is_valid_vosk_model(&path);
                    eprintln!("[DEBUG] Found {:?}, valid: {}", path, is_valid);
                    if is_valid {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        // Avoid duplicates if same model is in multiple dirs
                        if seen_names.insert(name.clone()) {
                            models.push(ModelInfo {
                                language: Self::detect_language(&name),
                                size_mb: Self::get_dir_size_mb(&path),
                                name,
                                path,
                            });
                        }
                    }
                }
            }
        }

        // Sort by name for consistent ordering
        models.sort_by(|a, b| a.name.cmp(&b.name));
        models
    }

    pub fn get_default_model(&self) -> Option<ModelInfo> {
        self.list_installed_models().into_iter().next()
    }

    pub fn ensure_models_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.models_dir)
    }
}

#[cfg(test)]
pub(crate) mod tests {
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
    fn test_detect_language() {
        assert_eq!(ModelManager::detect_language("vosk-model-en-us-0.22"), "English (US)");
        assert_eq!(ModelManager::detect_language("vosk-model-small-en-us-0.15"), "English (US)");
        assert_eq!(ModelManager::detect_language("vosk-model-de-0.21"), "German");
        assert_eq!(ModelManager::detect_language("vosk-model-fr-0.22"), "French");
        assert_eq!(ModelManager::detect_language("vosk-model-es-0.42"), "Spanish");
        assert_eq!(ModelManager::detect_language("vosk-model-cn-0.22"), "Chinese");
        assert_eq!(ModelManager::detect_language("vosk-model-ru-0.42"), "Russian");
        assert_eq!(ModelManager::detect_language("vosk-model-ja-0.22"), "Japanese");
        assert_eq!(ModelManager::detect_language("some-random-model"), "Unknown");
    }

    #[test]
    fn test_is_valid_vosk_model_nonexistent() {
        assert!(!ModelManager::is_valid_vosk_model(Path::new("/nonexistent/path")));
    }

    #[test]
    fn test_is_valid_vosk_model_with_am_and_graph() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_dir = temp_dir.path().join("vosk-model-en-us-0.22");

        // Create model directory structure
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::create_dir_all(model_dir.join("am")).unwrap();
        std::fs::create_dir_all(model_dir.join("graph")).unwrap();

        assert!(ModelManager::is_valid_vosk_model(&model_dir),
            "Model with am/ and graph/ directories should be valid");
    }

    #[test]
    fn test_is_valid_vosk_model_with_conf_and_graph() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_dir = temp_dir.path().join("vosk-model-small-en-us-0.15");

        // Create model directory structure (some models use conf instead of am)
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::create_dir_all(model_dir.join("conf")).unwrap();
        std::fs::create_dir_all(model_dir.join("graph")).unwrap();

        assert!(ModelManager::is_valid_vosk_model(&model_dir),
            "Model with conf/ and graph/ directories should be valid");
    }

    #[test]
    fn test_is_valid_vosk_model_with_mfcc_conf() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_dir = temp_dir.path().join("vosk-model-test");

        // Create model with mfcc.conf file
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("mfcc.conf"), "test").unwrap();

        assert!(ModelManager::is_valid_vosk_model(&model_dir),
            "Model with mfcc.conf should be valid");
    }

    #[test]
    fn test_is_valid_vosk_model_empty_dir_is_invalid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_dir = temp_dir.path().join("empty-model");

        std::fs::create_dir_all(&model_dir).unwrap();

        assert!(!ModelManager::is_valid_vosk_model(&model_dir),
            "Empty directory should NOT be a valid model");
    }

    #[test]
    fn test_is_valid_vosk_model_only_am_is_invalid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_dir = temp_dir.path().join("partial-model");

        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::create_dir_all(model_dir.join("am")).unwrap();
        // Missing graph/ directory

        assert!(!ModelManager::is_valid_vosk_model(&model_dir),
            "Model with only am/ (no graph/) should NOT be valid");
    }

    #[test]
    fn test_list_installed_models_finds_valid_models() {
        let temp_dir = tempfile::tempdir().unwrap();
        let models_dir = temp_dir.path();

        // Create a valid model
        let model1_dir = models_dir.join("vosk-model-en-us-0.22");
        std::fs::create_dir_all(model1_dir.join("am")).unwrap();
        std::fs::create_dir_all(model1_dir.join("graph")).unwrap();

        // Create another valid model
        let model2_dir = models_dir.join("vosk-model-small-de-0.15");
        std::fs::create_dir_all(model2_dir.join("conf")).unwrap();
        std::fs::create_dir_all(model2_dir.join("graph")).unwrap();

        // Create an invalid directory (should be ignored)
        let invalid_dir = models_dir.join("not-a-model");
        std::fs::create_dir_all(&invalid_dir).unwrap();

        // Use isolated manager to avoid picking up models from dev environment
        let manager = ModelManager::new_isolated(models_dir.to_path_buf());
        let installed = manager.list_installed_models();

        assert_eq!(installed.len(), 2, "Should find exactly 2 valid models");
        assert!(installed.iter().any(|m| m.name == "vosk-model-en-us-0.22"),
            "Should find en-us model");
        assert!(installed.iter().any(|m| m.name == "vosk-model-small-de-0.15"),
            "Should find de model");

        // Check language detection
        let en_model = installed.iter().find(|m| m.name.contains("en-us")).unwrap();
        assert_eq!(en_model.language, "English (US)");

        let de_model = installed.iter().find(|m| m.name.contains("-de-")).unwrap();
        assert_eq!(de_model.language, "German");
    }

    #[test]
    fn test_list_installed_models_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        // Use isolated manager to avoid picking up models from dev environment
        let manager = ModelManager::new_isolated(temp_dir.path().to_path_buf());

        let installed = manager.list_installed_models();
        assert!(installed.is_empty(), "Empty models dir should return no models");
    }

    #[test]
    fn test_list_installed_models_nonexistent_dir() {
        // Use isolated manager to avoid picking up models from dev environment
        let manager = ModelManager::new_isolated(PathBuf::from("/this/path/does/not/exist"));

        let installed = manager.list_installed_models();
        assert!(installed.is_empty(), "Nonexistent dir should return no models");
    }

    #[test]
    fn test_get_default_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let models_dir = temp_dir.path();

        // Create a valid model
        let model_dir = models_dir.join("vosk-model-en-us-0.22");
        std::fs::create_dir_all(model_dir.join("am")).unwrap();
        std::fs::create_dir_all(model_dir.join("graph")).unwrap();

        // Use isolated manager to avoid picking up models from dev environment
        let manager = ModelManager::new_isolated(models_dir.to_path_buf());
        let default = manager.get_default_model();

        assert!(default.is_some(), "Should return a default model");
        assert_eq!(default.unwrap().name, "vosk-model-en-us-0.22");
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
