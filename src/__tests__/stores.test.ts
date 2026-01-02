import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import {
  audioDevices,
  selectedDevice,
  isRecording,
  currentTranscription,
  partialTranscription,
  availableModels,
  installedModels,
  selectedModel,
  isModelLoaded,
  claudeStatus,
  terminalOutput,
  settings,
  isClaudeRunning,
  hasError,
  errorMessage,
  canRecord,
  appendTerminalOutput,
  clearTerminalOutput,
  updateTranscription,
  clearTranscription,
  loadSettings,
  saveSettings,
  type RecognitionResult,
  type AppSettings,
} from '$lib/stores/app';

describe('Audio Stores', () => {
  beforeEach(() => {
    audioDevices.set([]);
    selectedDevice.set(null);
    isRecording.set(false);
  });

  it('should initialize audioDevices as empty array', () => {
    expect(get(audioDevices)).toEqual([]);
  });

  it('should update audioDevices', () => {
    const devices = [
      { name: 'Microphone 1', is_default: true },
      { name: 'Microphone 2', is_default: false },
    ];
    audioDevices.set(devices);
    expect(get(audioDevices)).toEqual(devices);
  });

  it('should track recording state', () => {
    expect(get(isRecording)).toBe(false);
    isRecording.set(true);
    expect(get(isRecording)).toBe(true);
  });

  it('should track selected device', () => {
    expect(get(selectedDevice)).toBeNull();
    selectedDevice.set('Test Device');
    expect(get(selectedDevice)).toBe('Test Device');
  });
});

describe('Transcription Stores', () => {
  beforeEach(() => {
    currentTranscription.set('');
    partialTranscription.set('');
  });

  it('should update partial transcription', () => {
    const result: RecognitionResult = {
      text: 'partial text',
      is_final: false,
      confidence: 0.8,
    };
    updateTranscription(result);
    expect(get(partialTranscription)).toBe('partial text');
    expect(get(currentTranscription)).toBe('');
  });

  it('should update final transcription', () => {
    const result: RecognitionResult = {
      text: 'final text',
      is_final: true,
      confidence: 0.95,
    };
    updateTranscription(result);
    expect(get(currentTranscription)).toBe('final text');
    expect(get(partialTranscription)).toBe('');
  });

  it('should clear transcription', () => {
    currentTranscription.set('some text');
    partialTranscription.set('partial');
    clearTranscription();
    expect(get(currentTranscription)).toBe('');
    expect(get(partialTranscription)).toBe('');
  });
});

describe('Model Stores', () => {
  beforeEach(() => {
    availableModels.set([]);
    installedModels.set([]);
    selectedModel.set(null);
    isModelLoaded.set(false);
  });

  it('should track available models', () => {
    const models = [
      { name: 'model-1', path: '/path/1', language: 'en', size_mb: 100 },
    ];
    availableModels.set(models);
    expect(get(availableModels)).toEqual(models);
  });

  it('should track model loaded state', () => {
    expect(get(isModelLoaded)).toBe(false);
    isModelLoaded.set(true);
    expect(get(isModelLoaded)).toBe(true);
  });
});

describe('Claude Status Stores', () => {
  beforeEach(() => {
    claudeStatus.set('Stopped');
    terminalOutput.set('');
  });

  it('should track Claude status', () => {
    expect(get(claudeStatus)).toBe('Stopped');
    claudeStatus.set('Running');
    expect(get(claudeStatus)).toBe('Running');
  });

  it('should derive isClaudeRunning', () => {
    expect(get(isClaudeRunning)).toBe(false);
    claudeStatus.set('Running');
    expect(get(isClaudeRunning)).toBe(true);
    claudeStatus.set('Starting');
    expect(get(isClaudeRunning)).toBe(false);
  });

  it('should derive hasError', () => {
    expect(get(hasError)).toBe(false);
    claudeStatus.set({ Error: 'test error' });
    expect(get(hasError)).toBe(true);
  });

  it('should derive errorMessage', () => {
    expect(get(errorMessage)).toBeNull();
    claudeStatus.set({ Error: 'test error' });
    expect(get(errorMessage)).toBe('test error');
  });
});

describe('Terminal Output', () => {
  beforeEach(() => {
    terminalOutput.set('');
  });

  it('should append terminal output', () => {
    appendTerminalOutput('line 1\n');
    appendTerminalOutput('line 2\n');
    expect(get(terminalOutput)).toBe('line 1\nline 2\n');
  });

  it('should clear terminal output', () => {
    terminalOutput.set('some output');
    clearTerminalOutput();
    expect(get(terminalOutput)).toBe('');
  });
});

describe('Derived canRecord Store', () => {
  beforeEach(() => {
    isModelLoaded.set(false);
    isRecording.set(false);
  });

  it('should be false when model not loaded', () => {
    expect(get(canRecord)).toBe(false);
  });

  it('should be true when model loaded and not recording', () => {
    isModelLoaded.set(true);
    expect(get(canRecord)).toBe(true);
  });

  it('should be false when recording', () => {
    isModelLoaded.set(true);
    isRecording.set(true);
    expect(get(canRecord)).toBe(false);
  });
});

describe('Settings Store', () => {
  beforeEach(() => {
    settings.set({
      selectedDevice: null,
      selectedModel: null,
      recordingMode: 'toggle',
      pushToTalkKey: 'Space',
      theme: 'system',
    });
    // Clear localStorage mock
    vi.stubGlobal('localStorage', {
      getItem: vi.fn(),
      setItem: vi.fn(),
    });
  });

  it('should have default settings', () => {
    const currentSettings = get(settings);
    expect(currentSettings.recordingMode).toBe('toggle');
    expect(currentSettings.pushToTalkKey).toBe('Space');
    expect(currentSettings.theme).toBe('system');
  });

  it('should update settings', () => {
    const newSettings: AppSettings = {
      selectedDevice: 'Test Device',
      selectedModel: '/path/to/model',
      recordingMode: 'push-to-talk',
      pushToTalkKey: 'ControlLeft',
      theme: 'dark',
    };
    settings.set(newSettings);
    expect(get(settings)).toEqual(newSettings);
  });

  it('should save settings to localStorage', () => {
    const newSettings: AppSettings = {
      selectedDevice: null,
      selectedModel: null,
      recordingMode: 'toggle',
      pushToTalkKey: 'Space',
      theme: 'dark',
    };
    saveSettings(newSettings);
    expect(localStorage.setItem).toHaveBeenCalledWith(
      'icanhastool_settings',
      JSON.stringify(newSettings)
    );
  });

  it('should load settings from localStorage', () => {
    const storedSettings = {
      theme: 'light',
      recordingMode: 'push-to-talk',
    };
    vi.mocked(localStorage.getItem).mockReturnValue(JSON.stringify(storedSettings));
    loadSettings();
    const current = get(settings);
    expect(current.theme).toBe('light');
    expect(current.recordingMode).toBe('push-to-talk');
  });

  it('should handle invalid localStorage data', () => {
    vi.mocked(localStorage.getItem).mockReturnValue('invalid json');
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    loadSettings();
    expect(consoleSpy).toHaveBeenCalled();
    consoleSpy.mockRestore();
  });
});
