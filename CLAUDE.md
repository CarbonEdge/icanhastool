# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

icanhastool is a cross-platform desktop application that provides voice input to Claude Code using local speech recognition (Vosk). Built with Tauri v2 (Rust backend) and Svelte 5/TypeScript frontend.

## Development Agents

See `.claude/agents.md` for specialized agents to help with:
- **test-runner** - Run all tests (frontend + Rust)
- **build-check** - Verify builds and type checking
- **code-reviewer** - Review code for quality
- **rust-backend** - Work on Rust code in src-tauri/
- **svelte-frontend** - Work on Svelte code in src/
- **e2e-tester** - Run Playwright E2E tests
- **feature-planner** - Plan new features

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
- Linux: `~/.local/share/com.icanhastool.app/models/`

## Common Workflows

### Adding a New Feature
1. Use the **feature-planner** agent to design the approach
2. Implement Rust backend changes (if needed) using **rust-backend** patterns
3. Implement Svelte frontend changes using **svelte-frontend** patterns
4. Run **test-runner** to verify all tests pass
5. Run **build-check** to verify the build

### Fixing a Bug
1. Reproduce the issue and identify the affected layer (Rust/Svelte)
2. Write a failing test first
3. Fix the bug following existing patterns
4. Run **test-runner** to verify the fix
5. Use **code-reviewer** to check the changes

### Code Review Checklist
- [ ] Tests added/updated for changes
- [ ] Type checking passes (`npm run check`)
- [ ] Frontend coverage at 100% (`npm run test:coverage`)
- [ ] Rust tests pass (`cargo test --manifest-path src-tauri/Cargo.toml`)
- [ ] No security vulnerabilities introduced
- [ ] Cross-platform considerations addressed
