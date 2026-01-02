<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import {
    isRecording,
    isModelLoaded,
    partialTranscription,
    currentTranscription,
    updateTranscription,
    clearTranscription,
    settings,
    type RecognitionResult,
  } from './stores/app';

  const dispatch = createEventDispatcher<{
    transcription: string;
  }>();

  let unlisten: UnlistenFn | null = null;
  let unlistenFinal: UnlistenFn | null = null;
  let isPushToTalkActive = false;

  // Subscribe to stores
  let recording = false;
  let modelLoaded = false;
  let partial = '';
  let currentSettings = { recordingMode: 'toggle' as const, pushToTalkKey: 'Space' };

  isRecording.subscribe((v) => (recording = v));
  isModelLoaded.subscribe((v) => (modelLoaded = v));
  partialTranscription.subscribe((v) => (partial = v));
  settings.subscribe((v) => (currentSettings = v));

  onMount(async () => {
    // Listen for transcription events
    unlisten = await listen<RecognitionResult>('transcription', (event) => {
      updateTranscription(event.payload);
    });

    unlistenFinal = await listen<RecognitionResult>('transcription-final', (event) => {
      updateTranscription(event.payload);
      if (event.payload.text.trim()) {
        dispatch('transcription', event.payload.text);
      }
    });

    // Set up keyboard events for push-to-talk
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (unlistenFinal) unlistenFinal();
    window.removeEventListener('keydown', handleKeyDown);
    window.removeEventListener('keyup', handleKeyUp);
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (currentSettings.recordingMode !== 'push-to-talk') return;
    if (e.code === currentSettings.pushToTalkKey && !isPushToTalkActive) {
      e.preventDefault();
      isPushToTalkActive = true;
      startRecording();
    }
  }

  function handleKeyUp(e: KeyboardEvent) {
    if (currentSettings.recordingMode !== 'push-to-talk') return;
    if (e.code === currentSettings.pushToTalkKey && isPushToTalkActive) {
      e.preventDefault();
      isPushToTalkActive = false;
      stopRecording();
    }
  }

  async function startRecording() {
    if (recording || !modelLoaded) return;

    try {
      clearTranscription();
      await invoke('start_recording', { deviceName: null });
      isRecording.set(true);
    } catch (e) {
      console.error('Failed to start recording:', e);
    }
  }

  async function stopRecording() {
    if (!recording) return;

    try {
      const result = await invoke<RecognitionResult>('stop_recording');
      isRecording.set(false);

      if (result.text.trim()) {
        dispatch('transcription', result.text);
      }
    } catch (e) {
      console.error('Failed to stop recording:', e);
      isRecording.set(false);
    }
  }

  function toggleRecording() {
    if (recording) {
      stopRecording();
    } else {
      startRecording();
    }
  }

  // Expose methods for external control
  export { startRecording, stopRecording };
</script>

<div class="voice-control">
  <button
    class="record-button"
    class:recording
    class:disabled={!modelLoaded}
    on:click={toggleRecording}
    disabled={!modelLoaded}
    title={modelLoaded
      ? recording
        ? 'Click to stop recording'
        : 'Click to start recording'
      : 'Load a speech model first'}
  >
    <svg viewBox="0 0 24 24" width="32" height="32" fill="currentColor">
      {#if recording}
        <!-- Stop icon -->
        <rect x="6" y="6" width="12" height="12" rx="2" />
      {:else}
        <!-- Microphone icon -->
        <path
          d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm-1 1.93c-3.94-.49-7-3.85-7-7.93h2c0 3.31 2.69 6 6 6s6-2.69 6-6h2c0 4.08-3.06 7.44-7 7.93V19h3v2H9v-2h3v-3.07z"
        />
      {/if}
    </svg>
  </button>

  <div class="transcription-display">
    {#if recording}
      <div class="recording-indicator">
        <span class="pulse"></span>
        Recording...
      </div>
    {/if}

    {#if partial}
      <div class="partial-text">{partial}</div>
    {/if}
  </div>

  {#if currentSettings.recordingMode === 'push-to-talk'}
    <div class="mode-hint">Hold {currentSettings.pushToTalkKey} to talk</div>
  {/if}
</div>

<style>
  .voice-control {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 16px;
  }

  .record-button {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    border: none;
    background-color: #3b82f6;
    color: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
  }

  .record-button:hover:not(.disabled) {
    transform: scale(1.05);
    box-shadow: 0 6px 16px rgba(59, 130, 246, 0.5);
  }

  .record-button.recording {
    background-color: #ef4444;
    animation: pulse-glow 1.5s ease-in-out infinite;
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
  }

  .record-button.disabled {
    background-color: #6b7280;
    cursor: not-allowed;
    box-shadow: none;
  }

  @keyframes pulse-glow {
    0%,
    100% {
      box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
    }
    50% {
      box-shadow: 0 4px 24px rgba(239, 68, 68, 0.6);
    }
  }

  .transcription-display {
    min-height: 48px;
    text-align: center;
  }

  .recording-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #ef4444;
    font-weight: 500;
  }

  .pulse {
    width: 8px;
    height: 8px;
    background-color: #ef4444;
    border-radius: 50%;
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }

  .partial-text {
    color: #6b7280;
    font-style: italic;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mode-hint {
    font-size: 12px;
    color: #9ca3af;
  }
</style>
