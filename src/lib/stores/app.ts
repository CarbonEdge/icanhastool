/**
 * Application state stores
 */
import { writable, derived, type Readable } from 'svelte/store';

// ============================================================================
// Types
// ============================================================================

export interface AudioDevice {
  name: string;
  is_default: boolean;
}

export interface ModelInfo {
  name: string;
  path: string;
  language: string;
  size_mb: number;
}

export interface RecognitionResult {
  text: string;
  is_final: boolean;
  confidence: number | null;
}

export interface OutputEvent {
  data: string;
  is_error: boolean;
}

export type ProcessStatus = 'Starting' | 'Running' | 'Stopped' | { Error: string };

export type RecordingMode = 'toggle' | 'push-to-talk';

export type ThemeOption =
  | 'light'
  | 'dark'
  | 'system'
  | 'high-contrast-dark'
  | 'high-contrast-light';

export interface AppSettings {
  selectedDevice: string | null;
  selectedModel: string | null;
  recordingMode: RecordingMode;
  pushToTalkKey: string;
  theme: ThemeOption;
  // Accessibility
  fontSize: number; // 0.75 - 2.0 multiplier
  // Workspace
  currentWorkspace: string | null;
  recentWorkspaces: string[];
}

// ============================================================================
// Stores
// ============================================================================

// Audio devices
export const audioDevices = writable<AudioDevice[]>([]);
export const selectedDevice = writable<string | null>(null);

// Recording state
export const isRecording = writable<boolean>(false);
export const currentTranscription = writable<string>('');
export const partialTranscription = writable<string>('');

// Speech recognition
export const availableModels = writable<ModelInfo[]>([]);
export const installedModels = writable<ModelInfo[]>([]);
export const selectedModel = writable<string | null>(null);
export const isModelLoaded = writable<boolean>(false);

// Claude Code
export const claudeStatus = writable<ProcessStatus>('Stopped');
export const terminalOutput = writable<string>('');

// Settings
export const settings = writable<AppSettings>({
  selectedDevice: null,
  selectedModel: null,
  recordingMode: 'toggle',
  pushToTalkKey: 'Space',
  theme: 'system',
  fontSize: 1.0,
  currentWorkspace: null,
  recentWorkspaces: [],
});

// ============================================================================
// Derived Stores
// ============================================================================

export const isClaudeRunning: Readable<boolean> = derived(
  claudeStatus,
  ($status) => $status === 'Running'
);

export const hasError: Readable<boolean> = derived(
  claudeStatus,
  ($status) => typeof $status === 'object' && 'Error' in $status
);

export const errorMessage: Readable<string | null> = derived(
  claudeStatus,
  ($status) => {
    if (typeof $status === 'object' && 'Error' in $status) {
      return $status.Error;
    }
    return null;
  }
);

export const canRecord: Readable<boolean> = derived(
  [isModelLoaded, isRecording],
  ([$loaded, $recording]) => $loaded && !$recording
);

// ============================================================================
// Actions
// ============================================================================

export function appendTerminalOutput(data: string): void {
  terminalOutput.update((current) => current + data);
}

export function clearTerminalOutput(): void {
  terminalOutput.set('');
}

export function updateTranscription(result: RecognitionResult): void {
  if (result.is_final) {
    currentTranscription.set(result.text);
    partialTranscription.set('');
  } else {
    partialTranscription.set(result.text);
  }
}

export function clearTranscription(): void {
  currentTranscription.set('');
  partialTranscription.set('');
}

export function addRecentWorkspace(workspace: string): void {
  settings.update((current) => {
    const recents = current.recentWorkspaces.filter((w) => w !== workspace);
    recents.unshift(workspace);
    // Keep only last 10 workspaces
    const newRecents = recents.slice(0, 10);
    const newSettings = {
      ...current,
      currentWorkspace: workspace,
      recentWorkspaces: newRecents,
    };
    // Persist immediately
    if (typeof window !== 'undefined') {
      try {
        localStorage.setItem(SETTINGS_KEY, JSON.stringify(newSettings));
      } catch (e) {
        console.error('Failed to save settings:', e);
      }
    }
    return newSettings;
  });
}

// ============================================================================
// Persistence
// ============================================================================

const SETTINGS_KEY = 'icanhastool_settings';

export function loadSettings(): void {
  if (typeof window === 'undefined') return;

  try {
    const stored = localStorage.getItem(SETTINGS_KEY);
    if (stored) {
      const parsed = JSON.parse(stored) as Partial<AppSettings>;
      settings.update((current) => ({ ...current, ...parsed }));
    }
  } catch (e) {
    console.error('Failed to load settings:', e);
  }
}

export function saveSettings(newSettings: AppSettings): void {
  if (typeof window === 'undefined') return;

  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(newSettings));
    settings.set(newSettings);
  } catch (e) {
    console.error('Failed to save settings:', e);
  }
}
