import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import VoiceControl from '$lib/VoiceControl.svelte';
import { isRecording, isModelLoaded, settings } from '$lib/stores/app';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('VoiceControl Component', () => {
  beforeEach(() => {
    isRecording.set(false);
    isModelLoaded.set(false);
    settings.set({
      selectedDevice: null,
      selectedModel: null,
      recordingMode: 'toggle',
      pushToTalkKey: 'Space',
      theme: 'system',
    });
  });

  it('should render record button', () => {
    render(VoiceControl);
    const button = screen.getByRole('button');
    expect(button).toBeDefined();
  });

  it('should disable button when model not loaded', () => {
    render(VoiceControl);
    const button = screen.getByRole('button');
    expect(button.hasAttribute('disabled')).toBe(true);
  });

  it('should enable button when model is loaded', async () => {
    isModelLoaded.set(true);
    render(VoiceControl);
    const button = screen.getByRole('button');
    expect(button.hasAttribute('disabled')).toBe(false);
  });

  it('should show correct title when model not loaded', () => {
    render(VoiceControl);
    const button = screen.getByRole('button');
    expect(button.getAttribute('title')).toBe('Load a speech model first');
  });

  it('should show correct title when ready to record', () => {
    isModelLoaded.set(true);
    render(VoiceControl);
    const button = screen.getByRole('button');
    expect(button.getAttribute('title')).toBe('Click to start recording');
  });

  it('should show push-to-talk hint when in that mode', () => {
    settings.update((s) => ({ ...s, recordingMode: 'push-to-talk' }));
    render(VoiceControl);
    expect(screen.getByText(/Hold Space to talk/)).toBeDefined();
  });

  it('should not show push-to-talk hint in toggle mode', () => {
    render(VoiceControl);
    expect(screen.queryByText(/Hold.*to talk/)).toBeNull();
  });

  it('should show recording indicator when recording', () => {
    isRecording.set(true);
    render(VoiceControl);
    expect(screen.getByText('Recording...')).toBeDefined();
  });

  it('should have recording class when recording', () => {
    isRecording.set(true);
    isModelLoaded.set(true);
    render(VoiceControl);
    const button = screen.getByRole('button');
    expect(button.classList.contains('recording')).toBe(true);
  });
});

describe('VoiceControl keyboard events', () => {
  beforeEach(() => {
    isRecording.set(false);
    isModelLoaded.set(true);
    settings.set({
      selectedDevice: null,
      selectedModel: null,
      recordingMode: 'push-to-talk',
      pushToTalkKey: 'Space',
      theme: 'system',
    });
  });

  it('should respond to configured hotkey in push-to-talk mode', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    render(VoiceControl);

    // Simulate keydown
    await fireEvent.keyDown(window, { code: 'Space' });

    // Should have called start_recording
    expect(invoke).toHaveBeenCalledWith('start_recording', { deviceName: null });
  });
});
