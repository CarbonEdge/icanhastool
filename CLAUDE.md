# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

icanhastool is a cross-platform desktop application that provides voice input to Claude Code using local speech recognition (Vosk). Built with Tauri v2 (Rust backend) and Svelte/TypeScript frontend.

## Build Commands

```bash
# Install dependencies
npm install

# Development mode (hot reload)
npm run tauri dev

# Build for production
npm run tauri build

# Run frontend tests
npm test

# Run frontend tests with coverage (100% required)
npm run test:coverage

# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run E2E tests
npm run test:e2e

# Type checking
npm run check
```

## Architecture

### Backend (Rust - src-tauri/src/)
- `audio.rs` - Audio capture using cpal crate, 16kHz mono for Vosk
- `vosk_stt.rs` - Speech recognition using Vosk library
- `claude.rs` - Claude Code PTY spawning and management via portable-pty
- `commands.rs` - Tauri IPC commands exposed to frontend
- `lib.rs` - App initialization and plugin setup

### Frontend (Svelte/TypeScript - src/)
- `lib/Terminal.svelte` - xterm.js terminal emulator
- `lib/VoiceControl.svelte` - Recording controls (toggle + push-to-talk)
- `lib/Settings.svelte` - Configuration panel
- `lib/Waveform.svelte` - Audio visualization
- `lib/stores/app.ts` - Svelte stores for application state

### Key Dependencies
- **Rust**: tauri, cpal, vosk, portable-pty, tauri-plugin-global-shortcut
- **Frontend**: @xterm/xterm, @tauri-apps/api, svelte

## Testing Strategy

All modules use trait-based abstractions with mock implementations for testing without hardware:
- `MockAudioCapture` in audio.rs
- `MockSpeechRecognizer` in vosk_stt.rs
- `MockClaudeProcess` in claude.rs

Frontend tests use vitest + @testing-library/svelte with mocked Tauri APIs.

## Voice Model Setup

Vosk models should be placed in the app data directory:
- Windows: `%APPDATA%\com.icanhastool.app\models\`
- macOS: `~/Library/Application Support/com.icanhastool.app/models/`
