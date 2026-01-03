# icanhastool Feature Roadmap

This roadmap outlines planned features focused on **accessibility for children and disabled users**, **workspace management**, **Claude Code integration**, and **long-term AI enhancements**.

---

## Current State Assessment

### What Works Well
- Voice input with local Vosk speech recognition
- Terminal emulator with PTY for Claude Code
- Basic dark/light theme support
- Push-to-talk and toggle recording modes
- Audio device selection

### Critical Gaps Identified
- No workspace/directory selection UI (backend supports it)
- No .claude settings detection or integration
- No MCP, agents, or skills support
- No accessibility controls (font size, contrast, keyboard nav)
- Hardcoded UI sizes and limited customization

---

## Phase 1: Accessibility Foundation

**Goal**: Make the app usable for children, elderly, and users with disabilities.

### 1.1 Visual Accessibility

#### Font Size Control
- [ ] Add font size multiplier setting (0.75x - 2.0x)
- [ ] Apply to terminal (currently hardcoded to 14px)
- [ ] Apply to all UI text (h1, buttons, labels, status)
- [ ] Persist in settings store
- [ ] Add quick-access buttons: `A-` `A` `A+` in toolbar

**Files to modify:**
- `src/lib/stores/app.ts` - Add `fontSize: number` to AppSettings
- `src/lib/Terminal.svelte` - Make fontSize dynamic
- `src/lib/Settings.svelte` - Add font size slider
- `src/routes/+page.svelte` - Apply CSS variable for global font scale

#### High Contrast Themes
- [ ] Create "High Contrast Dark" theme (bright text on black)
- [ ] Create "High Contrast Light" theme (dark text on white)
- [ ] Increase focus ring visibility
- [ ] Add theme preview in settings

#### Large UI Mode
- [ ] Double-size mode for buttons and controls
- [ ] Larger touch/click targets (min 48px per WCAG)
- [ ] Increased spacing between interactive elements
- [ ] "Kids Mode" toggle that enables all accessibility features at once

### 1.2 Keyboard Accessibility

#### Global Shortcuts
- [ ] `Ctrl+Shift+R` - Start/stop recording
- [ ] `Ctrl+Shift+T` - Focus terminal
- [ ] `Ctrl+Shift+S` - Open settings
- [ ] `Ctrl+Shift+C` - Clear terminal
- [ ] `Ctrl+Plus/Minus` - Increase/decrease font size
- [ ] `Escape` - Close modals, cancel recording
- [ ] Document all shortcuts in Help panel

**Implementation:**
- Use existing `tauri-plugin-global-shortcut`
- Add shortcut configuration in Settings
- Show shortcut hints on hover/focus

#### Focus Management
- [ ] Visible focus rings on all interactive elements
- [ ] Logical tab order through UI
- [ ] Focus trap in settings modal
- [ ] Announce focus changes for screen readers

### 1.3 Screen Reader Support

#### ARIA Enhancements
- [ ] Add `role` and `aria-label` to all buttons
- [ ] Live regions for transcription updates
- [ ] Announce recording state changes
- [ ] Announce Claude status changes

#### Terminal Accessibility
- [ ] Option to copy terminal output to accessible text panel
- [ ] Voice readback of Claude responses (via TTS)
- [ ] Simplified text-only output mode

### 1.4 Motor Accessibility

#### Voice-First Interface
- [ ] All actions performable by voice
- [ ] Voice commands: "start recording", "send", "clear", "settings"
- [ ] Confirmation sounds for actions
- [ ] Adjustable hold duration for push-to-talk

---

## Phase 2: Workspace Management

**Goal**: Easily change working directories and automatically detect .claude settings.

### 2.1 Directory Selection UI

#### Workspace Picker
- [ ] Add "Open Folder" button in header/toolbar
- [ ] Use Tauri dialog plugin for native folder picker
- [ ] Show current workspace path in header
- [ ] Recent workspaces dropdown (last 10)
- [ ] Persist last workspace in settings

**Implementation:**
```rust
// Add to Cargo.toml
tauri-plugin-dialog = "2"

// New command in commands.rs
#[tauri::command]
pub async fn select_workspace(app: AppHandle) -> Result<Option<String>, String>
```

**UI Changes:**
- Add workspace display in `+page.svelte` header
- Add "Change Workspace" button
- Pass workspace to `Terminal.svelte` as `workingDir` prop

### 2.2 Quick Directory Access

#### Favorites/Bookmarks
- [ ] Star frequently used directories
- [ ] Show favorites in dropdown
- [ ] Keyboard shortcut to open favorites (`Ctrl+Shift+F`)

#### Drag & Drop
- [ ] Drop folder onto app to open
- [ ] Drop file to open its parent directory

### 2.3 Workspace State

#### Per-Workspace Settings
- [ ] Remember terminal state per workspace
- [ ] Auto-load last workspace on startup (optional)
- [ ] Workspace-specific Claude session persistence

---

## Phase 3: Claude Code Integration

**Goal**: Detect and use .claude settings, support agents, MCP, and skills.

### 3.1 .claude Directory Detection

#### Auto-Detection
- [ ] On workspace change, scan for `.claude/` directory
- [ ] Parse `.claude/settings.json` if present
- [ ] Parse `.claude/settings.local.json` for local overrides
- [ ] Show indicator when .claude config detected

**Implementation:**
```rust
// New module: src-tauri/src/claude_config.rs
pub struct ClaudeConfig {
    pub settings: Option<ClaudeSettings>,
    pub agents: Vec<AgentDefinition>,
    pub mcp_servers: Vec<McpServer>,
}

pub fn detect_claude_config(workspace: &Path) -> Result<ClaudeConfig, Error>
```

### 3.2 Agent Support

#### Agent Discovery
- [ ] Parse `.claude/agents.md` for agent definitions
- [ ] List available agents in sidebar panel
- [ ] One-click to invoke agent
- [ ] Show agent descriptions on hover

#### Agent UI Panel
- [ ] Collapsible "Agents" panel in sidebar
- [ ] Agent icons/avatars for visual recognition (kid-friendly)
- [ ] "Run Agent" button for each
- [ ] Voice command: "run [agent name]"

### 3.3 MCP Server Integration

#### MCP Configuration
- [ ] Read MCP servers from .claude/settings.json
- [ ] Pass MCP config to Claude Code process
- [ ] Show connected MCP servers in status bar
- [ ] Manual MCP server connection UI

**Implementation:**
- Pass MCP config via environment variables to Claude process
- Or use Claude Code's native MCP handling

### 3.4 Skills Support

#### Skill Discovery
- [ ] Detect available skills from Claude Code
- [ ] List skills in "Tools" panel
- [ ] Quick-invoke skills by name
- [ ] Voice command: "use [skill name]"

---

## Phase 4: AI Workflow Generation

**Goal**: Create, save, and run repeatable AI workflows.

### 4.1 Workflow Builder

#### Visual Workflow Editor
- [ ] Drag-and-drop workflow steps
- [ ] Step types: Prompt, Agent, Skill, File Operation, Wait
- [ ] Connect steps with arrows
- [ ] Save workflows to `.claude/workflows/`

#### Simple Mode (for Kids)
- [ ] Pre-built workflow templates
- [ ] "Recipe" metaphor - ingredients (inputs) and steps
- [ ] Colorful, icon-based interface
- [ ] Voice-guided workflow creation

### 4.2 Workflow Execution

#### Run Workflows
- [ ] One-click workflow execution
- [ ] Progress indicator for each step
- [ ] Pause/resume capability
- [ ] Error handling with retry options

#### Scheduled Workflows
- [ ] Run workflows on schedule
- [ ] Trigger workflows on file changes
- [ ] Webhook triggers for automation

### 4.3 Workflow Library

#### Sharing
- [ ] Export workflows as JSON
- [ ] Import community workflows
- [ ] Rate and review workflows
- [ ] Built-in template library

---

## Phase 5: Vector Database Integration (Long-term)

**Goal**: Make codebases easier for Claude to understand via semantic search.

### 5.1 Local Embedding Generation

#### File Indexing
- [ ] Index workspace files on open
- [ ] Generate embeddings using local model (e.g., sentence-transformers)
- [ ] Store in local SQLite + vector extension (sqlite-vec)
- [ ] Incremental updates on file changes

**Rust Dependencies:**
```toml
sqlite-vec = "0.1"  # Vector similarity in SQLite
```

### 5.2 Semantic Search

#### Search UI
- [ ] "Search codebase" input in toolbar
- [ ] Semantic search results (not just keyword)
- [ ] Preview results in side panel
- [ ] "Add to context" button for results

#### Context Enhancement
- [ ] Auto-suggest relevant files for current conversation
- [ ] "Smart context" mode - automatically include related code
- [ ] Show context usage/limits

### 5.3 Knowledge Base

#### Documentation Indexing
- [ ] Index README, docs/, wiki content
- [ ] Create searchable knowledge base
- [ ] Link to relevant docs in Claude responses

#### Learning from History
- [ ] Index past Claude conversations
- [ ] Find similar past problems/solutions
- [ ] Build project-specific knowledge over time

### 5.4 Multi-Codebase Support

#### Cross-Project Search
- [ ] Index multiple workspaces
- [ ] Search across all indexed codebases
- [ ] Find similar code patterns across projects

---

## Implementation Priority Matrix

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| Font size control | High | Low | P0 |
| Workspace picker | High | Low | P0 |
| High contrast themes | High | Low | P0 |
| Keyboard shortcuts | High | Medium | P0 |
| .claude detection | High | Medium | P1 |
| Agent panel | Medium | Medium | P1 |
| Kids mode | High | Medium | P1 |
| Screen reader support | High | High | P1 |
| MCP integration | Medium | Medium | P2 |
| Skills panel | Medium | Medium | P2 |
| Workflow builder | Medium | High | P2 |
| Voice commands | Medium | High | P2 |
| Vector DB indexing | Medium | High | P3 |
| Semantic search | Medium | High | P3 |
| Cross-project search | Low | High | P3 |

---

## Quick Wins (Can implement now)

These features require minimal changes:

1. **Font size setting** - Add slider in Settings, apply to Terminal and CSS vars
2. **Workspace picker** - Add dialog plugin, button in header, pass to Terminal
3. **Recent workspaces** - Store in localStorage, show in dropdown
4. **High contrast CSS** - Add theme variants, no Rust changes
5. **Keyboard shortcuts** - Already have global-shortcut plugin registered

---

## Technical Architecture Updates

### New Rust Modules Needed

```
src-tauri/src/
├── claude_config.rs    # .claude directory parsing
├── workspace.rs        # Workspace state management
├── embeddings.rs       # Vector embeddings (Phase 5)
└── workflow.rs         # Workflow execution engine (Phase 4)
```

### New Frontend Components

```
src/lib/
├── WorkspacePicker.svelte    # Directory selection UI
├── AgentPanel.svelte         # Agent list and execution
├── SkillsPanel.svelte        # Skills list
├── AccessibilitySettings.svelte  # Dedicated a11y settings
├── WorkflowBuilder.svelte    # Visual workflow editor
└── KidsMode.svelte           # Simplified UI wrapper
```

### New Store Fields

```typescript
interface AppSettings {
  // Existing
  selectedDevice: string | null;
  selectedModel: string | null;
  recordingMode: 'toggle' | 'push-to-talk';
  pushToTalkKey: string;
  theme: 'light' | 'dark' | 'system' | 'high-contrast-dark' | 'high-contrast-light';

  // New - Accessibility
  fontSize: number;           // 0.75 - 2.0 multiplier
  largeUIMode: boolean;       // Bigger buttons/spacing
  kidsMode: boolean;          // Enable all accessibility
  reduceMotion: boolean;      // Disable animations

  // New - Workspace
  currentWorkspace: string | null;
  recentWorkspaces: string[];
  favoriteWorkspaces: string[];
  autoLoadLastWorkspace: boolean;

  // New - Shortcuts
  shortcuts: {
    startRecording: string;
    focusTerminal: string;
    openSettings: string;
    clearTerminal: string;
  };
}
```

---

## Success Metrics

### Accessibility
- [ ] WCAG 2.1 AA compliance
- [ ] Usable with screen reader (VoiceOver, NVDA)
- [ ] Usable with keyboard only
- [ ] Tested with children ages 8-12

### Usability
- [ ] Change workspace in < 3 clicks
- [ ] .claude settings auto-detected in < 1 second
- [ ] First-time setup < 5 minutes

### Performance
- [ ] App launch < 2 seconds
- [ ] Workspace indexing < 30 seconds for 10k files
- [ ] Semantic search results < 500ms

---

## Next Steps

1. **Immediate**: Implement Phase 1.1 (Font size control) and Phase 2.1 (Workspace picker)
2. **Short-term**: Complete accessibility features and .claude detection
3. **Medium-term**: Agent/MCP integration and workflow builder
4. **Long-term**: Vector database and semantic search

---

*This roadmap is a living document. Update as features are completed or priorities change.*
