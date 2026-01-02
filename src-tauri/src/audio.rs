//! Audio capture module for recording from the microphone.
//!
//! Uses the cpal crate for cross-platform audio input.
//! Audio is captured at 16kHz mono (required by Vosk).

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, StreamConfig};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use thiserror::Error;

/// Audio capture errors
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No input device available")]
    NoInputDevice,
    #[error("Failed to get default input config: {0}")]
    ConfigError(String),
    #[error("Failed to build input stream: {0}")]
    StreamError(String),
    #[error("Failed to play stream: {0}")]
    PlayError(String),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}

/// Audio device information
#[derive(Debug, Clone, serde::Serialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

/// Trait for audio capture abstraction (enables testing)
pub trait AudioCapture: Send + Sync {
    fn list_devices(&self) -> Result<Vec<AudioDeviceInfo>, AudioError>;
    fn start_recording(
        &self,
        device_name: Option<&str>,
        callback: Arc<dyn Fn(Vec<i16>) + Send + Sync>,
    ) -> Result<(), AudioError>;
    fn stop_recording(&self);
    fn is_recording(&self) -> bool;
}

/// Command sent to the audio capture thread
enum AudioCommand {
    Start {
        device_name: Option<String>,
        callback: Arc<dyn Fn(Vec<i16>) + Send + Sync>,
    },
    Stop,
}

/// Real audio capture implementation using cpal.
/// Uses a dedicated thread to manage the stream since cpal::Stream is not Send+Sync.
pub struct CpalAudioCapture {
    command_sender: Mutex<Option<Sender<AudioCommand>>>,
    is_recording: Arc<AtomicBool>,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
}

impl CpalAudioCapture {
    pub fn new() -> Self {
        Self {
            command_sender: Mutex::new(None),
            is_recording: Arc::new(AtomicBool::new(false)),
            thread_handle: Mutex::new(None),
        }
    }

    fn get_device(host: &Host, device_name: Option<&str>) -> Result<Device, AudioError> {
        match device_name {
            Some(name) => host
                .input_devices()
                .map_err(|e| AudioError::ConfigError(e.to_string()))?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| AudioError::DeviceNotFound(name.to_string())),
            None => host
                .default_input_device()
                .ok_or(AudioError::NoInputDevice),
        }
    }

    fn create_config(_device: &Device) -> Result<StreamConfig, AudioError> {
        // Vosk requires 16kHz mono
        Ok(StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Default,
        })
    }

    fn start_audio_thread(&self) -> Sender<AudioCommand> {
        let (tx, rx) = mpsc::channel::<AudioCommand>();
        let is_recording = self.is_recording.clone();

        let handle = thread::spawn(move || {
            let host = cpal::default_host();
            let mut _current_stream: Option<cpal::Stream> = None;

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::Start { device_name, callback } => {
                        // Stop any existing stream
                        _current_stream = None;

                        let device = match Self::get_device(&host, device_name.as_deref()) {
                            Ok(d) => d,
                            Err(e) => {
                                eprintln!("Failed to get audio device: {}", e);
                                continue;
                            }
                        };

                        let config = match Self::create_config(&device) {
                            Ok(c) => c,
                            Err(e) => {
                                eprintln!("Failed to create config: {}", e);
                                continue;
                            }
                        };

                        let sample_format = match device.default_input_config() {
                            Ok(c) => c.sample_format(),
                            Err(e) => {
                                eprintln!("Failed to get input config: {}", e);
                                continue;
                            }
                        };

                        let err_fn = |err| eprintln!("Audio stream error: {}", err);

                        let stream = match sample_format {
                            SampleFormat::I16 => {
                                let cb = callback.clone();
                                device.build_input_stream(
                                    &config,
                                    move |data: &[i16], _| {
                                        cb(data.to_vec());
                                    },
                                    err_fn,
                                    None,
                                )
                            }
                            SampleFormat::F32 => {
                                let cb = callback.clone();
                                device.build_input_stream(
                                    &config,
                                    move |data: &[f32], _| {
                                        let samples: Vec<i16> = data
                                            .iter()
                                            .map(|&s| (s * i16::MAX as f32) as i16)
                                            .collect();
                                        cb(samples);
                                    },
                                    err_fn,
                                    None,
                                )
                            }
                            _ => {
                                eprintln!("Unsupported sample format: {:?}", sample_format);
                                continue;
                            }
                        };

                        match stream {
                            Ok(s) => {
                                if let Err(e) = s.play() {
                                    eprintln!("Failed to start stream: {}", e);
                                    continue;
                                }
                                _current_stream = Some(s);
                                is_recording.store(true, Ordering::SeqCst);
                            }
                            Err(e) => {
                                eprintln!("Failed to build stream: {}", e);
                            }
                        }
                    }
                    AudioCommand::Stop => {
                        _current_stream = None;
                        is_recording.store(false, Ordering::SeqCst);
                    }
                }
            }
        });

        *self.thread_handle.lock() = Some(handle);
        tx
    }
}

impl Default for CpalAudioCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCapture for CpalAudioCapture {
    fn list_devices(&self) -> Result<Vec<AudioDeviceInfo>, AudioError> {
        let host = cpal::default_host();
        let default_name = host
            .default_input_device()
            .and_then(|d| d.name().ok());

        let devices = host
            .input_devices()
            .map_err(|e| AudioError::ConfigError(e.to_string()))?
            .filter_map(|device| {
                device.name().ok().map(|name| AudioDeviceInfo {
                    is_default: default_name.as_ref() == Some(&name),
                    name,
                })
            })
            .collect();

        Ok(devices)
    }

    fn start_recording(
        &self,
        device_name: Option<&str>,
        callback: Arc<dyn Fn(Vec<i16>) + Send + Sync>,
    ) -> Result<(), AudioError> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Ensure thread is running and get/create sender
        let mut sender_guard = self.command_sender.lock();
        let sender = match sender_guard.as_ref() {
            Some(s) => s.clone(),
            None => {
                let s = self.start_audio_thread();
                *sender_guard = Some(s.clone());
                s
            }
        };

        sender
            .send(AudioCommand::Start {
                device_name: device_name.map(|s| s.to_string()),
                callback,
            })
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        Ok(())
    }

    fn stop_recording(&self) {
        if let Some(sender) = self.command_sender.lock().as_ref() {
            let _ = sender.send(AudioCommand::Stop);
        }
    }

    fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// Mock audio capture for testing
    pub struct MockAudioCapture {
        devices: Vec<AudioDeviceInfo>,
        is_recording: AtomicBool,
        start_called: AtomicUsize,
        stop_called: AtomicUsize,
        should_fail: AtomicBool,
    }

    impl MockAudioCapture {
        pub fn new() -> Self {
            Self {
                devices: vec![
                    AudioDeviceInfo {
                        name: "Test Microphone".to_string(),
                        is_default: true,
                    },
                    AudioDeviceInfo {
                        name: "Secondary Mic".to_string(),
                        is_default: false,
                    },
                ],
                is_recording: AtomicBool::new(false),
                start_called: AtomicUsize::new(0),
                stop_called: AtomicUsize::new(0),
                should_fail: AtomicBool::new(false),
            }
        }

        pub fn set_should_fail(&self, fail: bool) {
            self.should_fail.store(fail, Ordering::SeqCst);
        }

        pub fn start_call_count(&self) -> usize {
            self.start_called.load(Ordering::SeqCst)
        }

        pub fn stop_call_count(&self) -> usize {
            self.stop_called.load(Ordering::SeqCst)
        }
    }

    impl AudioCapture for MockAudioCapture {
        fn list_devices(&self) -> Result<Vec<AudioDeviceInfo>, AudioError> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(AudioError::NoInputDevice);
            }
            Ok(self.devices.clone())
        }

        fn start_recording(
            &self,
            device_name: Option<&str>,
            _callback: Arc<dyn Fn(Vec<i16>) + Send + Sync>,
        ) -> Result<(), AudioError> {
            self.start_called.fetch_add(1, Ordering::SeqCst);

            if self.should_fail.load(Ordering::SeqCst) {
                return Err(AudioError::NoInputDevice);
            }

            if let Some(name) = device_name {
                if !self.devices.iter().any(|d| d.name == name) {
                    return Err(AudioError::DeviceNotFound(name.to_string()));
                }
            }

            self.is_recording.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn stop_recording(&self) {
            self.stop_called.fetch_add(1, Ordering::SeqCst);
            self.is_recording.store(false, Ordering::SeqCst);
        }

        fn is_recording(&self) -> bool {
            self.is_recording.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn test_mock_list_devices() {
        let capture = MockAudioCapture::new();
        let devices = capture.list_devices().unwrap();

        assert_eq!(devices.len(), 2);
        assert!(devices[0].is_default);
        assert!(!devices[1].is_default);
    }

    #[test]
    fn test_mock_list_devices_fails() {
        let capture = MockAudioCapture::new();
        capture.set_should_fail(true);

        let result = capture.list_devices();
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_start_stop_recording() {
        let capture = MockAudioCapture::new();
        let callback = Arc::new(|_: Vec<i16>| {});

        assert!(!capture.is_recording());

        capture.start_recording(None, callback).unwrap();
        assert!(capture.is_recording());
        assert_eq!(capture.start_call_count(), 1);

        capture.stop_recording();
        assert!(!capture.is_recording());
        assert_eq!(capture.stop_call_count(), 1);
    }

    #[test]
    fn test_mock_start_with_specific_device() {
        let capture = MockAudioCapture::new();
        let callback = Arc::new(|_: Vec<i16>| {});

        capture
            .start_recording(Some("Test Microphone"), callback)
            .unwrap();
        assert!(capture.is_recording());
    }

    #[test]
    fn test_mock_start_with_invalid_device() {
        let capture = MockAudioCapture::new();
        let callback = Arc::new(|_: Vec<i16>| {});

        let result = capture.start_recording(Some("Nonexistent Device"), callback);
        assert!(matches!(result, Err(AudioError::DeviceNotFound(_))));
    }

    #[test]
    fn test_mock_start_fails() {
        let capture = MockAudioCapture::new();
        capture.set_should_fail(true);
        let callback = Arc::new(|_: Vec<i16>| {});

        let result = capture.start_recording(None, callback);
        assert!(result.is_err());
    }

    #[test]
    fn test_audio_device_info_serialization() {
        let info = AudioDeviceInfo {
            name: "Test Device".to_string(),
            is_default: true,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("Test Device"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_audio_error_display() {
        let err = AudioError::NoInputDevice;
        assert_eq!(err.to_string(), "No input device available");

        let err = AudioError::DeviceNotFound("test".to_string());
        assert_eq!(err.to_string(), "Device not found: test");
    }
}
