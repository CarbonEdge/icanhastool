# icanhastool

Voice interface for Claude Code using local speech recognition.

A cross-platform desktop application that enables voice input to Claude Code, powered by Vosk for offline speech recognition.

## Features

- **Voice Input**: Control Claude Code with your voice
- **Local Speech Recognition**: Uses Vosk for offline, privacy-preserving speech-to-text
- **Full Terminal Emulator**: Complete terminal experience with colors and scrollback
- **Cross-Platform**: Works on Windows and macOS
- **Two Input Modes**: Toggle recording or push-to-talk
- **Configurable**: Choose audio devices, speech models, and hotkeys

## Installation

### Download Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/your-username/icanhastool/releases) page:

- **Windows**: `.msi` installer
- **macOS**: `.dmg` disk image

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) v20+
- [Rust](https://rustup.rs/) stable
- [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

#### Steps

```bash
# Clone the repository
git clone https://github.com/your-username/icanhastool.git
cd icanhastool

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Setting Up Speech Recognition

1. Download a Vosk model from [alphacephei.com/vosk/models](https://alphacephei.com/vosk/models)
2. Extract the model to the app's models directory:
   - Windows: `%APPDATA%\com.icanhastool.app\models\`
   - macOS: `~/Library/Application Support/com.icanhastool.app/models/`
3. Open Settings in the app and select your model

Recommended models:
- **vosk-model-small-en-us-0.15** (~40MB) - Fast, good accuracy
- **vosk-model-en-us-0.22** (~1.8GB) - Slower, excellent accuracy

## Usage

1. Start the application
2. Load a speech model in Settings
3. Click the record button or use push-to-talk to speak
4. Your speech will be transcribed and sent to Claude Code

### Keyboard Shortcuts

- **Space** (configurable): Push-to-talk (when enabled)

## Development

```bash
# Run frontend tests
npm test

# Run frontend tests with coverage
npm run test:coverage

# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run E2E tests
npm run test:e2e

# Type checking
npm run check
```

## Architecture

```
icanhastool/
├── src/                    # Svelte frontend
│   ├── lib/               # Components and stores
│   └── routes/            # SvelteKit routes
├── src-tauri/             # Rust backend
│   └── src/
│       ├── audio.rs       # Audio capture
│       ├── vosk_stt.rs    # Speech recognition
│       ├── claude.rs      # Claude Code PTY
│       └── commands.rs    # Tauri IPC
└── e2e/                   # Playwright E2E tests
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
