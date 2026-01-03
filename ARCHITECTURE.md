# icanhastool Architecture Review

## Executive Summary

**Architecture Grade: A-** (excellent for desktop)
**Scalability Grade: B** (needs refactoring for multi-workspace)
**Cross-Platform Grade: D** (desktop-only dependencies block mobile)

The current architecture is well-designed for desktop with excellent trait-based abstractions and clean separation of concerns. However, mobile support would require significant refactoring due to desktop-only dependencies.

---

## Current Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (Svelte 5)                  │
│  ┌─────────────┬─────────────┬──────────┬───────────┐  │
│  │  Terminal   │ VoiceControl│ Settings │ Waveform  │  │
│  │  (xterm.js) │             │          │           │  │
│  └──────┬──────┴──────┬──────┴────┬─────┴───────────┘  │
│         │             │           │                     │
│  ┌──────┴─────────────┴───────────┴─────────────────┐  │
│  │              Stores (app.ts)                      │  │
│  │   settings | claudeStatus | transcription | ...   │  │
│  └───────────────────────┬───────────────────────────┘  │
├──────────────────────────┼──────────────────────────────┤
│                   Tauri IPC                             │
├──────────────────────────┼──────────────────────────────┤
│                    Backend (Rust)                       │
│  ┌───────────────────────┴───────────────────────────┐  │
│  │              commands.rs (AppState)               │  │
│  │   Arc<dyn AudioCapture>                           │  │
│  │   Arc<dyn SpeechRecognizer>                       │  │
│  │   Arc<dyn ClaudeProcess>                          │  │
│  └────┬──────────────┬──────────────┬────────────────┘  │
│       │              │              │                   │
│  ┌────┴────┐   ┌─────┴─────┐  ┌─────┴─────┐            │
│  │audio.rs │   │vosk_stt.rs│  │ claude.rs │            │
│  │ (cpal)  │   │  (vosk)   │  │(portable- │            │
│  │         │   │           │  │   pty)    │            │
│  └────┬────┘   └─────┬─────┘  └─────┬─────┘            │
└───────┼──────────────┼──────────────┼──────────────────┘
        │              │              │
   ┌────┴────┐   ┌─────┴─────┐  ┌─────┴─────┐
   │Microphone│   │Vosk Model │  │Claude CLI │
   └──────────┘   └───────────┘  └───────────┘
```

### Strengths

| Aspect | Rating | Notes |
|--------|--------|-------|
| Trait-based abstractions | A+ | `AudioCapture`, `SpeechRecognizer`, `ClaudeProcess` traits enable testing without hardware |
| Mock implementations | A+ | `MockAudioCapture`, `MockSpeechRecognizer`, `MockClaudeProcess` for unit tests |
| Thread safety | A | `Arc`, `Mutex`, `AtomicBool` properly used throughout |
| Error handling | A | Custom `thiserror` types per module |
| IPC design | A | Clean command/event separation between frontend and backend |
| Test coverage | A | 55 frontend + 44 Rust tests |

### Weaknesses

| Aspect | Rating | Issue |
|--------|--------|-------|
| Mobile compatibility | F | cpal, portable-pty, vosk are desktop-only |
| Monolithic state | C | Single `AppState` doesn't scale to multi-workspace |
| Plugin architecture | D | No way to extend functionality without modifying core |
| Configuration | C | No config files, limited to localStorage |

---

## Platform Compatibility Matrix

| Dependency | Windows | macOS | Linux | Android | iOS |
|------------|---------|-------|-------|---------|-----|
| **Tauri v2** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **cpal** (audio) | ✅ | ✅ | ✅ | ⚠️ Unstable | ❌ |
| **vosk** (STT) | ✅ | ✅ | ✅ | ⚠️ NDK build | ⚠️ Large |
| **portable-pty** | ✅ | ✅ | ✅ | ❌ | ❌ |
| **xterm.js** | ✅ | ✅ | ✅ | ⚠️ Poor UX | ⚠️ Poor UX |
| **global-shortcut** | ✅ | ✅ | ✅ | ❌ | ❌ |

**Legend:** ✅ Works | ⚠️ Partial/Issues | ❌ Not supported

### Mobile Blockers

1. **portable-pty**: No PTY concept on mobile (sandboxed apps can't spawn shells)
2. **cpal**: Limited/unstable mobile support
3. **vosk**: Models are 40MB-1.8GB (impractical for mobile)
4. **Claude CLI**: Not installable on mobile devices

---

## Recommended Architecture Changes

### 1. Platform Capability Layer (Priority: High)

Add runtime capability detection so frontend can adapt:

```rust
// src-tauri/src/platform.rs
#[derive(Debug, Clone, Serialize)]
pub struct PlatformCapabilities {
    pub has_local_audio: bool,
    pub has_local_stt: bool,
    pub has_pty: bool,
    pub supports_global_shortcuts: bool,
    pub platform: String, // "windows", "macos", "linux", "android", "ios"
}

#[tauri::command]
pub fn get_platform_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        has_local_audio: cfg!(any(windows, target_os = "macos", target_os = "linux")),
        has_local_stt: cfg!(any(windows, target_os = "macos", target_os = "linux")),
        has_pty: cfg!(not(any(target_os = "android", target_os = "ios"))),
        supports_global_shortcuts: cfg!(not(any(target_os = "android", target_os = "ios"))),
        platform: std::env::consts::OS.to_string(),
    }
}
```

### 2. Service-Oriented State (Priority: High)

Split monolithic `AppState` into domain services:

```rust
// Current (monolithic)
pub struct AppState {
    audio: Arc<dyn AudioCapture>,
    recognizer: Arc<dyn SpeechRecognizer>,
    claude: Arc<dyn ClaudeProcess>,
    model_manager: ModelManager,
    audio_callback: Mutex<Option<...>>,
}

// Proposed (service-oriented)
pub struct AudioService {
    capture: Arc<dyn AudioCapture>,
    callback: Mutex<Option<...>>,
}

pub struct SpeechService {
    recognizer: Arc<dyn SpeechRecognizer>,
    model_manager: ModelManager,
}

pub struct ClaudeService {
    process: Arc<dyn ClaudeProcess>,
    workspace: Option<PathBuf>,
}

pub struct AppServices {
    audio: AudioService,
    speech: SpeechService,
    claude: ClaudeService,
}
```

### 3. Plugin Architecture (Priority: Medium)

Enable extensibility without core modifications:

```rust
// src-tauri/src/plugin.rs
pub trait IcanhastoolPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, app: &AppHandle) -> Result<()>;
    fn commands(&self) -> Vec<PluginCommand>;
    fn shutdown(&mut self);
}

// Example plugins:
// - McpPlugin: MCP server management
// - AgentPlugin: Agent discovery and execution
// - VectorDbPlugin: Codebase indexing and search
```

### 4. Configuration Hierarchy (Priority: Medium)

```
Precedence (highest to lowest):
1. Environment variables
2. Workspace settings (.claude/settings.json)
3. User config (app_data/config.toml)
4. System defaults (compiled-in)
```

```toml
# %APPDATA%/com.icanhastool.app/config.toml
[appearance]
theme = "dark"
font_size = 1.0

[audio]
default_device = "default"

[speech]
model_path = "vosk-model-en-us-0.22"

[workspace]
auto_load_last = true
recent_limit = 10
```

---

## Mobile Support Paths

### Option A: Desktop-Only (Recommended for Now)

**Effort:** 0 hours
**Rationale:** Current architecture is excellent for desktop. Mobile use case is weak (developers use Claude Code on laptops/desktops).

**Action:** Focus on desktop features:
- Workspace management
- .claude settings detection
- Agent/MCP integration
- Accessibility improvements
- Vector database integration

### Option B: Mobile-Friendly Fork

**Effort:** 100-150 hours
**Architecture:**

```
icanhastool-mobile/
├── Frontend: Chat-style UI (no terminal)
├── Audio: Platform-native APIs (SFSpeechRecognizer, Android SpeechRecognizer)
├── Backend: Claude API client (not local CLI)
└── Shared: UI components, theme, icons
```

**Pros:**
- Optimized UX per platform
- No abstraction overhead
- Faster development

**Cons:**
- Two codebases
- Feature parity challenges

### Option C: Unified Hybrid (Complex)

**Effort:** 200-300 hours
**Architecture:**

```
src-tauri/src/
├── platform/
│   ├── desktop.rs    # cpal + vosk + portable-pty
│   ├── android.rs    # JNI + SpeechRecognizer + Claude API
│   └── ios.rs        # ObjC + SFSpeechRecognizer + Claude API
├── services/
│   ├── audio/        # Platform-specific implementations
│   ├── stt/          # Platform-specific implementations
│   └── claude/       # PTY (desktop) vs API (mobile)
```

**Pros:**
- Single codebase
- Shared business logic

**Cons:**
- High complexity
- Difficult testing
- Abstraction overhead

---

## Immediate Action Items

### Phase 1: Foundation (Est. 20-30 hours)

1. **Add platform capabilities API** (2-3 hours)
   - Implement `PlatformCapabilities` struct
   - Add Tauri command
   - Frontend queries on startup

2. **Refactor to service architecture** (8-12 hours)
   - Split `AppState` into `AudioService`, `SpeechService`, `ClaudeService`
   - Update commands to use services
   - Maintain backward compatibility

3. **Add configuration layer** (6-8 hours)
   - Create `config.toml` support
   - Implement config hierarchy
   - Migrate from localStorage to Tauri Store

4. **Workspace state management** (4-6 hours)
   - Per-workspace settings tracking
   - Workspace history persistence
   - .claude directory detection

### Phase 2: Extensibility (Est. 40-60 hours)

5. **Plugin architecture** (16-24 hours)
   - Design plugin trait
   - Plugin discovery and lifecycle
   - Refactor audio/STT/Claude as plugins

6. **Agent/MCP integration** (16-20 hours)
   - Parse .claude/agents.md
   - Agent panel UI
   - MCP server management

7. **Accessibility** (8-16 hours)
   - Complete font size controls ✅ (done)
   - High-contrast themes
   - Keyboard navigation
   - Screen reader improvements

### Phase 3: Advanced Features (Est. 60-80 hours)

8. **Vector database** (24-32 hours)
   - Local embedding with sentence-transformers
   - sqlite-vec integration
   - Semantic search UI

9. **Workflow engine** (32-48 hours)
   - Workflow definition format
   - Execution engine
   - Visual builder

---

## Testing Strategy

### Current (Good)

- Unit tests with mocks for hardware dependencies
- 55 frontend + 44 Rust tests
- 100% coverage on stores

### Recommended Additions

| Layer | Strategy |
|-------|----------|
| Platform abstraction | Compile-time checks per target |
| Services | Integration tests with service mocks |
| Plugins | Plugin test harness |
| Configuration | Property-based testing for config merging |
| E2E | Playwright for critical user flows |

---

## Conclusion

The current architecture is **excellent for desktop** with strong fundamentals (traits, mocks, clean IPC). For scalability:

1. **Short-term:** Add platform capabilities + service architecture
2. **Medium-term:** Add plugin system for extensibility
3. **Long-term:** Evaluate mobile demand before investing in mobile support

**Recommendation:** Stay focused on desktop excellence. The mobile use case (voice input for Claude Code) doesn't translate well because Claude Code itself is desktop-only. A mobile app would require a fundamentally different product (chat with Claude API, not Claude Code CLI).
