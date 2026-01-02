//! Claude Code PTY integration module.
//!
//! Spawns Claude Code in a pseudo-terminal and handles bidirectional communication.

use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

/// Claude Code process errors
#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("Failed to spawn PTY: {0}")]
    PtySpawnError(String),
    #[error("Process not running")]
    NotRunning,
    #[error("Failed to write to process: {0}")]
    WriteError(String),
    #[error("Failed to read from process: {0}")]
    ReadError(String),
    #[error("Claude Code not found in PATH")]
    ClaudeNotFound,
}

/// Output event from Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEvent {
    pub data: String,
    pub is_error: bool,
}

/// Process status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopped,
    Error(String),
}

/// Trait for Claude Code process management (enables testing)
pub trait ClaudeProcess: Send + Sync {
    fn start(&self, working_dir: Option<&str>) -> Result<(), ClaudeError>;
    fn stop(&self) -> Result<(), ClaudeError>;
    fn send_input(&self, input: &str) -> Result<(), ClaudeError>;
    fn resize(&self, cols: u16, rows: u16) -> Result<(), ClaudeError>;
    fn status(&self) -> ProcessStatus;
    fn set_output_callback(&self, callback: Arc<dyn Fn(OutputEvent) + Send + Sync>);
}

/// Real Claude Code process implementation
pub struct ClaudeCodeProcess {
    pty_system: Box<dyn PtySystem + Send + Sync>,
    master: Mutex<Option<Box<dyn portable_pty::MasterPty + Send>>>,
    child: Mutex<Option<Box<dyn portable_pty::Child + Send + Sync>>>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    status: Mutex<ProcessStatus>,
    running: AtomicBool,
    output_callback: Mutex<Option<Arc<dyn Fn(OutputEvent) + Send + Sync>>>,
}

impl ClaudeCodeProcess {
    pub fn new() -> Self {
        Self {
            pty_system: native_pty_system(),
            master: Mutex::new(None),
            child: Mutex::new(None),
            writer: Mutex::new(None),
            status: Mutex::new(ProcessStatus::Stopped),
            running: AtomicBool::new(false),
            output_callback: Mutex::new(None),
        }
    }

    fn find_claude_command() -> Result<String, ClaudeError> {
        // Try common Claude Code command names
        let commands = ["claude", "claude-code"];

        for cmd in commands {
            if which::which(cmd).is_ok() {
                return Ok(cmd.to_string());
            }
        }

        // On Windows, also check for .exe and .cmd
        #[cfg(windows)]
        {
            for cmd in commands {
                for ext in [".exe", ".cmd", ".bat"] {
                    let full_cmd = format!("{}{}", cmd, ext);
                    if which::which(&full_cmd).is_ok() {
                        return Ok(full_cmd);
                    }
                }
            }
        }

        Err(ClaudeError::ClaudeNotFound)
    }
}

impl Default for ClaudeCodeProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeProcess for ClaudeCodeProcess {
    fn start(&self, working_dir: Option<&str>) -> Result<(), ClaudeError> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        *self.status.lock() = ProcessStatus::Starting;

        let claude_cmd = Self::find_claude_command()?;

        let pair = self
            .pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| ClaudeError::PtySpawnError(e.to_string()))?;

        let mut cmd = CommandBuilder::new(&claude_cmd);

        if let Some(dir) = working_dir {
            cmd.cwd(dir);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| ClaudeError::PtySpawnError(e.to_string()))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| ClaudeError::PtySpawnError(e.to_string()))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| ClaudeError::PtySpawnError(e.to_string()))?;

        *self.master.lock() = Some(pair.master);
        *self.child.lock() = Some(child);
        *self.writer.lock() = Some(writer);
        *self.status.lock() = ProcessStatus::Running;
        self.running.store(true, Ordering::SeqCst);

        // Start output reader thread
        let callback = self.output_callback.lock().clone();
        let running = self.running.clone();

        std::thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            let mut buffer = [0u8; 4096];

            while running.load(Ordering::SeqCst) {
                match std::io::Read::read(&mut reader, &mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        if let Some(ref cb) = callback {
                            cb(OutputEvent {
                                data,
                                is_error: false,
                            });
                        }
                    }
                    Err(e) => {
                        if let Some(ref cb) = callback {
                            cb(OutputEvent {
                                data: format!("Read error: {}", e),
                                is_error: true,
                            });
                        }
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn stop(&self) -> Result<(), ClaudeError> {
        self.running.store(false, Ordering::SeqCst);

        if let Some(mut child) = self.child.lock().take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        *self.master.lock() = None;
        *self.writer.lock() = None;
        *self.status.lock() = ProcessStatus::Stopped;

        Ok(())
    }

    fn send_input(&self, input: &str) -> Result<(), ClaudeError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(ClaudeError::NotRunning);
        }

        let mut writer_guard = self.writer.lock();
        let writer = writer_guard
            .as_mut()
            .ok_or(ClaudeError::NotRunning)?;

        writer
            .write_all(input.as_bytes())
            .map_err(|e| ClaudeError::WriteError(e.to_string()))?;

        writer
            .flush()
            .map_err(|e| ClaudeError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<(), ClaudeError> {
        let master_guard = self.master.lock();
        let master = master_guard.as_ref().ok_or(ClaudeError::NotRunning)?;

        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| ClaudeError::PtySpawnError(e.to_string()))
    }

    fn status(&self) -> ProcessStatus {
        self.status.lock().clone()
    }

    fn set_output_callback(&self, callback: Arc<dyn Fn(OutputEvent) + Send + Sync>) {
        *self.output_callback.lock() = Some(callback);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    /// Mock Claude process for testing
    pub struct MockClaudeProcess {
        status: Mutex<ProcessStatus>,
        running: AtomicBool,
        input_history: Mutex<Vec<String>>,
        output_callback: Mutex<Option<Arc<dyn Fn(OutputEvent) + Send + Sync>>>,
        should_fail: AtomicBool,
        start_count: AtomicUsize,
        stop_count: AtomicUsize,
        current_size: Mutex<(u16, u16)>,
    }

    impl MockClaudeProcess {
        pub fn new() -> Self {
            Self {
                status: Mutex::new(ProcessStatus::Stopped),
                running: AtomicBool::new(false),
                input_history: Mutex::new(Vec::new()),
                output_callback: Mutex::new(None),
                should_fail: AtomicBool::new(false),
                start_count: AtomicUsize::new(0),
                stop_count: AtomicUsize::new(0),
                current_size: Mutex::new((80, 24)),
            }
        }

        pub fn set_should_fail(&self, fail: bool) {
            self.should_fail.store(fail, Ordering::SeqCst);
        }

        pub fn input_history(&self) -> Vec<String> {
            self.input_history.lock().clone()
        }

        pub fn start_count(&self) -> usize {
            self.start_count.load(Ordering::SeqCst)
        }

        pub fn stop_count(&self) -> usize {
            self.stop_count.load(Ordering::SeqCst)
        }

        pub fn current_size(&self) -> (u16, u16) {
            *self.current_size.lock()
        }

        pub fn simulate_output(&self, data: &str, is_error: bool) {
            if let Some(cb) = self.output_callback.lock().as_ref() {
                cb(OutputEvent {
                    data: data.to_string(),
                    is_error,
                });
            }
        }
    }

    impl ClaudeProcess for MockClaudeProcess {
        fn start(&self, _working_dir: Option<&str>) -> Result<(), ClaudeError> {
            self.start_count.fetch_add(1, Ordering::SeqCst);

            if self.should_fail.load(Ordering::SeqCst) {
                *self.status.lock() = ProcessStatus::Error("Mock error".to_string());
                return Err(ClaudeError::ClaudeNotFound);
            }

            *self.status.lock() = ProcessStatus::Running;
            self.running.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn stop(&self) -> Result<(), ClaudeError> {
            self.stop_count.fetch_add(1, Ordering::SeqCst);
            *self.status.lock() = ProcessStatus::Stopped;
            self.running.store(false, Ordering::SeqCst);
            Ok(())
        }

        fn send_input(&self, input: &str) -> Result<(), ClaudeError> {
            if !self.running.load(Ordering::SeqCst) {
                return Err(ClaudeError::NotRunning);
            }

            if self.should_fail.load(Ordering::SeqCst) {
                return Err(ClaudeError::WriteError("Mock write error".to_string()));
            }

            self.input_history.lock().push(input.to_string());
            Ok(())
        }

        fn resize(&self, cols: u16, rows: u16) -> Result<(), ClaudeError> {
            if !self.running.load(Ordering::SeqCst) {
                return Err(ClaudeError::NotRunning);
            }

            *self.current_size.lock() = (cols, rows);
            Ok(())
        }

        fn status(&self) -> ProcessStatus {
            self.status.lock().clone()
        }

        fn set_output_callback(&self, callback: Arc<dyn Fn(OutputEvent) + Send + Sync>) {
            *self.output_callback.lock() = Some(callback);
        }
    }

    #[test]
    fn test_mock_start_stop() {
        let process = MockClaudeProcess::new();

        assert_eq!(process.status(), ProcessStatus::Stopped);

        process.start(None).unwrap();
        assert_eq!(process.status(), ProcessStatus::Running);
        assert_eq!(process.start_count(), 1);

        process.stop().unwrap();
        assert_eq!(process.status(), ProcessStatus::Stopped);
        assert_eq!(process.stop_count(), 1);
    }

    #[test]
    fn test_mock_start_fails() {
        let process = MockClaudeProcess::new();
        process.set_should_fail(true);

        let result = process.start(None);
        assert!(matches!(result, Err(ClaudeError::ClaudeNotFound)));
        assert!(matches!(
            process.status(),
            ProcessStatus::Error(_)
        ));
    }

    #[test]
    fn test_mock_send_input() {
        let process = MockClaudeProcess::new();
        process.start(None).unwrap();

        process.send_input("Hello Claude").unwrap();
        process.send_input("How are you?").unwrap();

        let history = process.input_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], "Hello Claude");
        assert_eq!(history[1], "How are you?");
    }

    #[test]
    fn test_mock_send_input_not_running() {
        let process = MockClaudeProcess::new();

        let result = process.send_input("Hello");
        assert!(matches!(result, Err(ClaudeError::NotRunning)));
    }

    #[test]
    fn test_mock_resize() {
        let process = MockClaudeProcess::new();
        process.start(None).unwrap();

        process.resize(120, 40).unwrap();
        assert_eq!(process.current_size(), (120, 40));
    }

    #[test]
    fn test_mock_resize_not_running() {
        let process = MockClaudeProcess::new();

        let result = process.resize(120, 40);
        assert!(matches!(result, Err(ClaudeError::NotRunning)));
    }

    #[test]
    fn test_mock_output_callback() {
        let process = MockClaudeProcess::new();
        let received = Arc::new(Mutex::new(Vec::new()));

        let received_clone = received.clone();
        process.set_output_callback(Arc::new(move |event| {
            received_clone.lock().push(event);
        }));

        process.simulate_output("Test output", false);
        process.simulate_output("Error output", true);

        let events = received.lock();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data, "Test output");
        assert!(!events[0].is_error);
        assert_eq!(events[1].data, "Error output");
        assert!(events[1].is_error);
    }

    #[test]
    fn test_output_event_serialization() {
        let event = OutputEvent {
            data: "Test data".to_string(),
            is_error: false,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Test data"));
        assert!(json.contains("false"));

        let deserialized: OutputEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.data, event.data);
        assert_eq!(deserialized.is_error, event.is_error);
    }

    #[test]
    fn test_process_status_serialization() {
        let status = ProcessStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Running"));

        let status = ProcessStatus::Error("test error".to_string());
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Error"));
        assert!(json.contains("test error"));
    }

    #[test]
    fn test_claude_error_display() {
        let err = ClaudeError::NotRunning;
        assert_eq!(err.to_string(), "Process not running");

        let err = ClaudeError::ClaudeNotFound;
        assert_eq!(err.to_string(), "Claude Code not found in PATH");

        let err = ClaudeError::WriteError("io error".to_string());
        assert!(err.to_string().contains("io error"));
    }
}
