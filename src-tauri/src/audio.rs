//! Audio capture module for recording from the microphone.
//!
//! Uses the cpal crate for cross-platform audio input.
//! Audio is captured at 16kHz mono (required by Vosk).

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, StreamConfig};
use parking_lot::Mutex;
use std::sync::Arc;
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

/// Real audio capture implementation using cpal
pub struct CpalAudioCapture {
    host: Host,
    stream: Mutex<Option<cpal::Stream>>,
    is_recording: Mutex<bool>,
}

impl CpalAudioCapture {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
            stream: Mutex::new(None),
            is_recording: Mutex::new(false),
        }
    }

    fn get_device(&self, device_name: Option<&str>) -> Result<Device, AudioError> {
        match device_name {
            Some(name) => self
                .host
                .input_devices()
                .map_err(|e| AudioError::ConfigError(e.to_string()))?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| AudioError::DeviceNotFound(name.to_string())),
            None => self
                .host
                .default_input_device()
                .ok_or(AudioError::NoInputDevice),
        }
    }

    fn create_config(device: &Device) -> Result<StreamConfig, AudioError> {
        let supported_config = device
            .default_input_config()
            .map_err(|e| AudioError::ConfigError(e.to_string()))?;

        // Vosk requires 16kHz mono
        Ok(StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Default,
        })
    }
}

impl Default for CpalAudioCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCapture for CpalAudioCapture {
    fn list_devices(&self) -> Result<Vec<AudioDeviceInfo>, AudioError> {
        let default_name = self
            .host
            .default_input_device()
            .and_then(|d| d.name().ok());

        let devices = self
            .host
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
        if *self.is_recording.lock() {
            return Ok(());
        }

        let device = self.get_device(device_name)?;
        let config = Self::create_config(&device)?;
        let sample_format = device
            .default_input_config()
            .map_err(|e| AudioError::ConfigError(e.to_string()))?
            .sample_format();

        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        let stream = match sample_format {
            SampleFormat::I16 => device
                .build_input_stream(
                    &config,
                    move |data: &[i16], _| {
                        callback(data.to_vec());
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| AudioError::StreamError(e.to_string()))?,
            SampleFormat::F32 => {
                let callback = callback.clone();
                device
                    .build_input_stream(
                        &config,
                        move |data: &[f32], _| {
                            let samples: Vec<i16> = data
                                .iter()
                                .map(|&s| (s * i16::MAX as f32) as i16)
                                .collect();
                            callback(samples);
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| AudioError::StreamError(e.to_string()))?
            }
            _ => {
                return Err(AudioError::ConfigError(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )))
            }
        };

        stream
            .play()
            .map_err(|e| AudioError::PlayError(e.to_string()))?;

        *self.stream.lock() = Some(stream);
        *self.is_recording.lock() = true;

        Ok(())
    }

    fn stop_recording(&self) {
        *self.stream.lock() = None;
        *self.is_recording.lock() = false;
    }

    fn is_recording(&self) -> bool {
        *self.is_recording.lock()
    }
}

#[cfg(test)]
mod tests {
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
