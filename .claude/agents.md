# icanhastool Development Agents

This file defines specialized agents for maintaining and building the icanhastool codebase.

## Agent Definitions

### test-runner
**Purpose**: Run all tests and report results

```
Run all tests for the icanhastool project:

1. Run frontend unit tests: npm test
2. Run frontend tests with coverage: npm run test:coverage
3. Run Rust backend tests: cargo test --manifest-path src-tauri/Cargo.toml
4. Report any failures with file paths and line numbers
5. Suggest fixes for failing tests

Coverage requirement: 100% for frontend code.
```

### build-check
**Purpose**: Verify the project builds and passes type checking

```
Verify the project builds correctly:

1. Run type checking: npm run check
2. Build frontend: npm run build
3. Build Tauri app: npm run tauri build (if full build requested)
4. Report any compilation errors with context
5. Suggest fixes for build issues
```

### code-reviewer
**Purpose**: Review code changes for quality and consistency

```
Review code changes in this Tauri v2 + Svelte 5 + Rust project:

Frontend (src/):
- Check Svelte 5 runes usage ($state, $derived, $effect)
- Verify TypeScript types are correct
- Check for proper Tauri API usage (@tauri-apps/api)
- Ensure components follow existing patterns

Backend (src-tauri/src/):
- Check Rust code for safety and idiomatic patterns
- Verify trait-based abstractions for testability
- Check error handling with proper Result types
- Ensure async code uses proper patterns

General:
- Check for security issues (command injection, XSS)
- Verify tests are included for new functionality
- Check that mocks are updated if interfaces change
- Make sure to leave a well documented and commented code base with good naming conventions
```

### rust-backend
**Purpose**: Develop and maintain Rust backend code

```
Work on the Rust backend in src-tauri/src/:

Key files:
- audio.rs - Audio capture using cpal (16kHz mono for Vosk)
- vosk_stt.rs - Speech recognition with Vosk library
- claude.rs - Claude Code PTY management via portable-pty
- commands.rs - Tauri IPC commands for frontend
- lib.rs - App initialization and plugin setup

Patterns to follow:
- Use trait-based abstractions for testability
- Include mock implementations (MockAudioCapture, MockSpeechRecognizer, MockClaudeProcess)
- Handle errors with Result<T, E> and proper error types
- Use async/await for non-blocking operations
- Emit events to frontend via Tauri event system
- Make sure to leave a well documented and commented code base with good naming conventions

After changes, run: cargo test --manifest-path src-tauri/Cargo.toml
```

### svelte-frontend
**Purpose**: Develop and maintain Svelte frontend code

```
Work on the Svelte frontend in src/:

Key files:
- lib/Terminal.svelte - xterm.js terminal emulator
- lib/VoiceControl.svelte - Recording controls (toggle + push-to-talk)
- lib/Settings.svelte - Configuration panel
- lib/Waveform.svelte - Audio visualization
- lib/stores/app.ts - Svelte stores for state
- routes/+page.svelte - Main app page

Patterns to follow:
- Use Svelte 5 runes: $state, $derived, $effect
- Mock Tauri APIs in tests using vi.mock('@tauri-apps/api')
- Use @testing-library/svelte for component tests
- Handle Tauri events with listen/emit from @tauri-apps/api/event
- Make sure to leave a well documented and commented code base with good naming conventions

After changes, run: npm test && npm run check
```

### e2e-tester
**Purpose**: Run and maintain E2E tests

```
Work with Playwright E2E tests in e2e/:

1. Run E2E tests: npm run test:e2e
2. Debug failing tests with Playwright inspector
3. Add new E2E tests for user workflows
4. Ensure tests work on Windows and macOS

Note: E2E tests require the app to be built first (npm run tauri build)
```

### feature-planner
**Purpose**: Plan new features with proper architecture

```
Plan new features for icanhastool:

1. Understand the request and break it into components
2. Identify which layers need changes:
   - Rust backend (src-tauri/src/)
   - Svelte frontend (src/)
   - Tauri IPC commands
3. Design the data flow between layers
4. Plan the test strategy (unit tests, mocks, E2E)
5. Create a step-by-step implementation plan

Consider:
- Trait abstractions for testability
- Event-based communication between Rust and Svelte
- Cross-platform compatibility (Windows, macOS, Linux)
- Features should also be very usable and accessabile to anyone including children.
- Features should be fun and the end user should enjoy them
```

## Usage

Invoke these agents using Claude Code's Task tool:

```
Task(subagent_type="general-purpose", prompt="[agent prompt]")
```

Or reference them in conversations:
- "Run the test-runner agent"
- "Use the code-reviewer agent to check my changes"
- "Apply the rust-backend agent patterns to add a new feature"

## Adding New Agents

When adding new agents:
1. Define a clear, single purpose
2. List specific files and patterns to follow
3. Include validation steps (tests, type checking)
4. Reference existing code patterns in the codebase
