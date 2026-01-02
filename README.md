# icanhastool

**Voice interface for Claude Code using local speech recognition.**

A cross-platform desktop application that enables voice input to Claude Code, powered by Vosk for offline speech recognition. Speak naturally and have your words sent directly to Claude Code.

---

## 30-Second Quick Start

```bash
# Clone and install
git clone https://github.com/anthropics/icanhastool.git && cd icanhastool
npm install

# Run (requires Node.js 20+, Rust 1.70+)
npm run tauri dev
```

Then: **Settings → Load a Vosk model → Click microphone → Speak!**

---

## Full Setup Guide

### Prerequisites

| Platform | Requirements |
|----------|-------------|
| **Windows** | [Node.js 20+](https://nodejs.org), [Rust](https://rustup.rs), [VS Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ |
| **macOS** | `brew install node rust` + Xcode CLI (`xcode-select --install`) |
| **Linux** | Node.js 20+, Rust, `libwebkit2gtk-4.1-dev libasound2-dev build-essential` |

### Step 1: Clone and Install

```bash
git clone https://github.com/anthropics/icanhastool.git
cd icanhastool
npm install
```

### Step 2: Download a Vosk Model

Download a speech recognition model from [alphacephei.com/vosk/models](https://alphacephei.com/vosk/models):

| Model | Size | Speed | Accuracy |
|-------|------|-------|----------|
| `vosk-model-small-en-us-0.15` | 40 MB | Fast | Good for testing |
| `vosk-model-en-us-0.22` | 1.8 GB | Slower | Production quality |

Extract to the models directory:

```bash
# Windows
mkdir "%APPDATA%\com.icanhastool.app\models"

# macOS
mkdir -p ~/Library/Application\ Support/com.icanhastool.app/models

# Linux
mkdir -p ~/.local/share/com.icanhastool.app/models
```

### Step 3: Run

```bash
# Development mode (hot reload)
npm run tauri dev

# Production build
npm run tauri build
```

### Step 4: Configure

1. Click **Settings** (gear icon)
2. Under **Speech Recognition**, click **Load** next to your model
3. Click the **microphone button** to start recording
4. Speak your command — it sends to Claude Code when you stop

## Features

- **Voice Input**: Control Claude Code with your voice
- **Local Speech Recognition**: Uses Vosk for offline, privacy-preserving speech-to-text (no data sent to cloud)
- **Full Terminal Emulator**: Complete terminal experience with ANSI colors and scrollback
- **Cross-Platform**: Works on Windows and macOS
- **Two Input Modes**:
  - **Toggle mode**: Click to start, click to stop
  - **Push-to-talk**: Hold a key while speaking (configurable)
- **Configurable**: Choose audio devices, speech models, hotkeys, and theme

## Voice Input Modes

### Toggle Mode (Default)
1. Click the microphone button to start recording
2. Speak your command
3. Click again to stop and send

### Push-to-Talk Mode
1. Open Settings and change "Recording Mode" to "Push to Talk"
2. Configure your preferred hotkey (default: Space)
3. Hold the key while speaking
4. Release to send

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/your-username/icanhastool/releases) page:

| Platform | File |
|----------|------|
| Windows | `icanhastool_x.x.x_x64.msi` |
| macOS (Intel) | `icanhastool_x.x.x_x64.dmg` |
| macOS (Apple Silicon) | `icanhastool_x.x.x_aarch64.dmg` |
| Linux | `icanhastool_x.x.x_amd64.AppImage` |

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) v20+
- [Rust](https://rustup.rs/) stable (1.70+)
- Platform-specific requirements:
  - **Windows**: Visual Studio Build Tools with C++ workload
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `build-essential`, `libwebkit2gtk-4.1-dev`, `libasound2-dev`

#### Build Steps

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

Built files will be in `src-tauri/target/release/bundle/`.

## Development

### Commands

```bash
# Start development server with hot reload
npm run tauri dev

# Run frontend unit tests
npm test

# Run frontend tests with coverage
npm run test:coverage

# Run Rust backend tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run E2E tests (requires app to be built)
npm run test:e2e

# Type checking
npm run check

# Build for production
npm run tauri build
```

### Project Structure

```
icanhastool/
├── src/                          # Svelte frontend
│   ├── lib/
│   │   ├── Terminal.svelte       # xterm.js terminal emulator
│   │   ├── VoiceControl.svelte   # Recording button & controls
│   │   ├── Settings.svelte       # Configuration panel
│   │   ├── Waveform.svelte       # Audio visualization
│   │   └── stores/
│   │       └── app.ts            # Svelte stores for state
│   ├── routes/
│   │   └── +page.svelte          # Main app page
│   └── __tests__/                # Frontend unit tests
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── audio.rs              # Audio capture (cpal)
│   │   ├── vosk_stt.rs           # Speech recognition (Vosk)
│   │   ├── claude.rs             # Claude Code PTY management
│   │   ├── commands.rs           # Tauri IPC commands
│   │   └── lib.rs                # App initialization
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── e2e/                          # Playwright E2E tests
├── .github/workflows/            # CI/CD workflows
└── package.json                  # Node.js dependencies
```

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Application                     │
├─────────────────────────────────────────────────────────┤
│  Frontend (Svelte + TypeScript)                         │
│  ├── Voice controls (toggle + push-to-talk)             │
│  ├── Terminal emulator (xterm.js)                       │
│  ├── Settings panel                                     │
│  └── Audio waveform visualization                       │
├─────────────────────────────────────────────────────────┤
│  Backend (Rust)                                         │
│  ├── Audio capture (cpal) → 16kHz mono                  │
│  ├── Speech recognition (Vosk) → text                   │
│  ├── PTY management (portable-pty) → Claude Code        │
│  └── IPC commands ↔ frontend                            │
└─────────────────────────────────────────────────────────┘
```

### Testing

All backend modules use **trait-based abstractions** with mock implementations for testing without hardware:

- `MockAudioCapture` - Test audio recording without microphone
- `MockSpeechRecognizer` - Test speech recognition without Vosk model
- `MockClaudeProcess` - Test PTY integration without Claude Code

Frontend tests use **Vitest** with `@testing-library/svelte` and mocked Tauri APIs.

## Troubleshooting

### "No model loaded" warning
Download and extract a Vosk model to the models directory (see Quick Start step 3).

### No audio devices found
- Ensure your microphone is connected and enabled
- On Windows, check Privacy Settings → Microphone
- On macOS, check System Preferences → Security & Privacy → Microphone

### Claude Code not starting
- Ensure Claude Code CLI is installed and in your PATH
- Try running `claude` in a terminal to verify it works

### Build errors on Windows
- Install Visual Studio Build Tools with "Desktop development with C++" workload
- Restart your terminal after installation

### Build errors on Linux
```bash
# Install required system dependencies
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev patchelf libasound2-dev build-essential
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`npm test && cargo test --manifest-path src-tauri/Cargo.toml`)
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request
