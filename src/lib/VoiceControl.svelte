<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import {
    isRecording,
    isModelLoaded,
    isClaudeRunning,
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
  let pendingTranscription = '';  // Holds transcription for preview before sending
  let isFiring = false;  // Debounce guard for fire button

  // Subscribe to stores
  let recording = false;
  let modelLoaded = false;
  let claudeRunning = false;
  let partial = '';
  let currentSettings: { recordingMode: 'toggle' | 'push-to-talk'; pushToTalkKey: string } = {
    recordingMode: 'toggle',
    pushToTalkKey: 'Space'
  };

  isRecording.subscribe((v) => (recording = v));
  isModelLoaded.subscribe((v) => (modelLoaded = v));
  isClaudeRunning.subscribe((v) => (claudeRunning = v));
  partialTranscription.subscribe((v) => (partial = v));
  settings.subscribe((v) => (currentSettings = v));

  onMount(async () => {
    // Listen for transcription events
    unlisten = await listen<RecognitionResult>('transcription', (event) => {
      updateTranscription(event.payload);
    });

    unlistenFinal = await listen<RecognitionResult>('transcription-final', (event) => {
      updateTranscription(event.payload);
      // Don't send immediately - store for preview
      if (event.payload.text.trim()) {
        pendingTranscription = event.payload.text;
        console.log('[VoiceControl] Transcription ready for preview:', pendingTranscription);
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
      // The transcription-final event listener handles dispatching the text
      // so we don't dispatch here to avoid duplication
      await invoke<RecognitionResult>('stop_recording');
      isRecording.set(false);
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

  async function handleReset() {
    console.log('[VoiceControl] Reset button clicked');
    pendingTranscription = '';
    clearTranscription();
    try {
      await invoke('reset_recognizer');
      console.log('[VoiceControl] Recognizer reset successfully');
    } catch (e) {
      console.error('[VoiceControl] Failed to reset recognizer:', e);
    }
  }

  function handleSend() {
    if (pendingTranscription.trim()) {
      console.log('[VoiceControl] Sending transcription:', pendingTranscription);
      dispatch('transcription', pendingTranscription);
      pendingTranscription = '';
      clearTranscription();
    }
  }

  async function handleFire() {
    if (isFiring || !claudeRunning) return;
    isFiring = true;
    console.log('[VoiceControl] Fire button clicked - sending Enter to Claude');
    try {
      await invoke('send_to_claude', { input: '\r' });
    } catch (e) {
      console.error('[VoiceControl] Failed to send Enter to Claude:', e);
    } finally {
      isFiring = false;
    }
  }

  // Expose methods for external control
  export { startRecording, stopRecording };
</script>

<div class="voice-control">
  <div class="button-group">
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

    <button
      class="fire-button"
      class:disabled={!claudeRunning}
      on:click={handleFire}
      disabled={!claudeRunning}
      aria-label="Submit prompt to Claude"
      title="Send Enter to submit the prompt"
    >
      <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
        <!-- Fire icon -->
        <path d="M12 23c-3.9 0-7-3.1-7-7 0-2.8 1.6-5.6 4.1-7.8.5-.4 1.2-.4 1.6.1.4.5.3 1.1-.1 1.5C8.5 11.5 7 13.8 7 16c0 2.8 2.2 5 5 5s5-2.2 5-5c0-1.4-.6-2.9-1.5-4.3-.3-.5-.2-1.1.3-1.5.5-.3 1.1-.2 1.5.3 1.3 1.8 2.2 3.9 2.2 5.5 0 3.9-3.1 7-7.5 7zM12 12c-1.7 0-3-1.3-3-3 0-1.2.8-2.5 2-3.5V2c0-.6.4-1 1-1s1 .4 1 1v3.5c1.2 1 2 2.3 2 3.5 0 1.7-1.3 3-3 3z"/>
      </svg>
    </button>
  </div>

  <div class="transcription-display">
    {#if recording}
      <div class="recording-indicator">
        <span class="pulse"></span>
        Recording...
      </div>
    {:else if pendingTranscription}
      <div class="pending-transcription">
        <div class="pending-text">{pendingTranscription}</div>
        <div class="action-buttons">
          <button class="send-button" on:click={handleSend} title="Send to Claude">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
            </svg>
            Send
          </button>
          <button class="reset-button" on:click={handleReset} title="Discard and try again">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M17.65 6.35A7.958 7.958 0 0012 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0112 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            Reset
          </button>
        </div>
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

  .button-group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
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

  .fire-button {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    border: none;
    background-color: #f97316;
    color: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    box-shadow: 0 4px 12px rgba(249, 115, 22, 0.4);
  }

  .fire-button:hover:not(.disabled) {
    transform: scale(1.05);
    box-shadow: 0 6px 16px rgba(249, 115, 22, 0.5);
    background-color: #ea580c;
  }

  .fire-button.disabled {
    background-color: #6b7280;
    cursor: not-allowed;
    box-shadow: none;
  }

  .reset-button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    background-color: #374151;
    color: #d1d5db;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .reset-button:hover:not(:disabled) {
    background-color: #4b5563;
    color: #f3f4f6;
  }

  .reset-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .pending-transcription {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background-color: #1f2937;
    border-radius: 8px;
    max-width: 400px;
  }

  .pending-text {
    color: #f3f4f6;
    font-size: 14px;
    text-align: center;
    word-wrap: break-word;
  }

  .action-buttons {
    display: flex;
    gap: 8px;
  }

  .send-button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    background-color: #10b981;
    color: white;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .send-button:hover {
    background-color: #059669;
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
