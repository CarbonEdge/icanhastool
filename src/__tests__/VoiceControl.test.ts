import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import VoiceControl from '$lib/VoiceControl.svelte';
import { isRecording, isModelLoaded, settings, claudeStatus } from '$lib/stores/app';

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
    claudeStatus.set('Stopped');
    settings.set({
      selectedDevice: null,
      selectedModel: null,
      recordingMode: 'toggle',
      pushToTalkKey: 'Space',
      theme: 'system',
      fontSize: 1.0,
      currentWorkspace: null,
      recentWorkspaces: [],
    });
  });

  it('should render record button', () => {
    render(VoiceControl);
    const button = screen.getByTitle('Load a speech model first');
    expect(button).toBeDefined();
  });

  it('should disable button when model not loaded', () => {
    render(VoiceControl);
    const button = screen.getByTitle('Load a speech model first');
    expect(button.hasAttribute('disabled')).toBe(true);
  });

  it('should enable button when model is loaded', async () => {
    isModelLoaded.set(true);
    render(VoiceControl);
    const button = screen.getByTitle('Click to start recording');
    expect(button.hasAttribute('disabled')).toBe(false);
  });

  it('should show correct title when model not loaded', () => {
    render(VoiceControl);
    const button = screen.getByTitle('Load a speech model first');
    expect(button.getAttribute('title')).toBe('Load a speech model first');
  });

  it('should show correct title when ready to record', () => {
    isModelLoaded.set(true);
    render(VoiceControl);
    const button = screen.getByTitle('Click to start recording');
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
    const button = screen.getByTitle('Click to stop recording');
    expect(button.classList.contains('recording')).toBe(true);
  });

  it('should not show action buttons when no pending transcription', () => {
    render(VoiceControl);
    // Reset and Send buttons only appear when there's a pending transcription
    expect(screen.queryByTitle('Reset and try again')).toBeNull();
    expect(screen.queryByTitle('Send to Claude')).toBeNull();
  });

  it('should not show action buttons while recording', () => {
    isRecording.set(true);
    isModelLoaded.set(true);
    render(VoiceControl);
    // Action buttons should not appear during recording
    expect(screen.queryByTitle('Discard and try again')).toBeNull();
    expect(screen.queryByTitle('Send to Claude')).toBeNull();
  });

  it('should render fire button', () => {
    render(VoiceControl);
    const button = screen.getByTitle('Send Enter to submit the prompt');
    expect(button).toBeDefined();
  });

  it('should disable fire button when Claude is not running', () => {
    claudeStatus.set('Stopped');
    render(VoiceControl);
    const button = screen.getByTitle('Send Enter to submit the prompt');
    expect(button.hasAttribute('disabled')).toBe(true);
  });

  it('should enable fire button when Claude is running', () => {
    claudeStatus.set('Running');
    render(VoiceControl);
    const button = screen.getByTitle('Send Enter to submit the prompt');
    expect(button.hasAttribute('disabled')).toBe(false);
  });

  it('should call send_to_claude with Enter when fire button clicked', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    claudeStatus.set('Running');
    render(VoiceControl);
    const button = screen.getByTitle('Send Enter to submit the prompt');

    await fireEvent.click(button);

    // Wait for async invoke call
    await new Promise((resolve) => setTimeout(resolve, 50));

    expect(invoke).toHaveBeenCalledWith('send_to_claude', { input: '\r' });
  });
});

describe('VoiceControl keyboard events', () => {
  beforeEach(() => {
    isRecording.set(false);
    isModelLoaded.set(true);
    claudeStatus.set('Stopped');
    settings.set({
      selectedDevice: null,
      selectedModel: null,
      recordingMode: 'push-to-talk',
      pushToTalkKey: 'Space',
      theme: 'system',
      fontSize: 1.0,
      currentWorkspace: null,
      recentWorkspaces: [],
    });
  });

  it('should respond to configured hotkey in push-to-talk mode', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    render(VoiceControl);

    // Wait for store subscriptions to settle
    await new Promise((resolve) => setTimeout(resolve, 50));

    // Simulate keydown
    await fireEvent.keyDown(window, { code: 'Space' });

    // Wait for async invoke call
    await new Promise((resolve) => setTimeout(resolve, 50));

    // Should have called start_recording
    expect(invoke).toHaveBeenCalledWith('start_recording', { deviceName: null });
  });
});
