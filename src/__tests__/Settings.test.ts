import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Settings from '$lib/Settings.svelte';
import { audioDevices, availableModels, installedModels, settings, isModelLoaded, type AppSettings } from '$lib/stores/app';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === 'list_audio_devices') {
      return Promise.resolve([
        { name: 'Test Microphone', is_default: true },
        { name: 'Secondary Mic', is_default: false },
      ]);
    }
    if (cmd === 'list_models') {
      return Promise.resolve([
        { name: 'vosk-model-small-en', path: '/models/small', language: 'English', size_mb: 40 },
      ]);
    }
    if (cmd === 'list_installed_models') {
      return Promise.resolve([]);
    }
    if (cmd === 'load_model') {
      return Promise.resolve();
    }
    return Promise.resolve();
  }),
}));

describe('Settings Component', () => {
  beforeEach(() => {
    audioDevices.set([]);
    availableModels.set([]);
    installedModels.set([]);
    isModelLoaded.set(false);
    settings.set({
      selectedDevice: null,
      selectedModel: null,
      recordingMode: 'toggle',
      pushToTalkKey: 'Space',
      theme: 'system',
    });
  });

  it('should not render when closed', () => {
    render(Settings, { isOpen: false });
    expect(screen.queryByText('Settings')).toBeNull();
  });

  it('should render when open', async () => {
    render(Settings, { isOpen: true });
    expect(await screen.findByText('Settings')).toBeDefined();
  });

  it('should display audio section', async () => {
    render(Settings, { isOpen: true });
    expect(await screen.findByText('Audio')).toBeDefined();
    expect(await screen.findByText('Microphone')).toBeDefined();
  });

  it('should display speech recognition section', async () => {
    render(Settings, { isOpen: true });
    expect(await screen.findByText('Speech Recognition')).toBeDefined();
  });

  it('should display voice input section', async () => {
    render(Settings, { isOpen: true });
    expect(await screen.findByText('Voice Input')).toBeDefined();
    expect(await screen.findByText('Recording Mode')).toBeDefined();
  });

  it('should display appearance section', async () => {
    render(Settings, { isOpen: true });
    expect(await screen.findByText('Appearance')).toBeDefined();
    expect(await screen.findByText('Theme')).toBeDefined();
  });

  it('should have close button', async () => {
    render(Settings, { isOpen: true });
    const closeButton = await screen.findByLabelText('Close settings');
    expect(closeButton).toBeDefined();
  });

  it('should have refresh devices button', async () => {
    render(Settings, { isOpen: true });
    expect(await screen.findByText('Refresh Devices')).toBeDefined();
  });

  it('should show model status when not loaded', async () => {
    render(Settings, { isOpen: true });
    expect(await screen.findByText('No model loaded')).toBeDefined();
  });

  it('should show model status when loaded', async () => {
    isModelLoaded.set(true);
    render(Settings, { isOpen: true });
    expect(await screen.findByText('Model loaded')).toBeDefined();
  });

  it('should show push-to-talk key option when mode is push-to-talk', async () => {
    settings.update((s) => ({ ...s, recordingMode: 'push-to-talk' }));
    render(Settings, { isOpen: true });
    expect(await screen.findByText('Push to Talk Key')).toBeDefined();
  });
});

describe('Settings interactions', () => {
  beforeEach(() => {
    settings.set({
      selectedDevice: null,
      selectedModel: null,
      recordingMode: 'toggle',
      pushToTalkKey: 'Space',
      theme: 'system',
    });
  });

  it('should change recording mode', async () => {
    render(Settings, { isOpen: true });

    const modeSelect = await screen.findByDisplayValue('Toggle (click to start/stop)');
    await fireEvent.change(modeSelect, { target: { value: 'push-to-talk' } });

    // Now push-to-talk key option should be visible
    expect(await screen.findByText('Push to Talk Key')).toBeDefined();
  });

  it('should change theme', async () => {
    render(Settings, { isOpen: true });

    const themeSelect = await screen.findByDisplayValue('System');
    await fireEvent.change(themeSelect, { target: { value: 'dark' } });

    // Theme should be updated in store
    let currentSettings: AppSettings | undefined;
    settings.subscribe((s) => (currentSettings = s))();
    expect(currentSettings?.theme).toBe('dark');
  });
});
