<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    audioDevices,
    availableModels,
    installedModels,
    isModelLoaded,
    settings,
    saveSettings,
    type AudioDevice,
    type ModelInfo,
    type AppSettings,
  } from './stores/app';

  export let isOpen = false;

  let devices: AudioDevice[] = [];
  let models: ModelInfo[] = [];
  let installed: ModelInfo[] = [];
  let currentSettings: AppSettings;
  let modelLoaded = false;
  let loading = false;
  let error = '';

  // Subscribe to stores
  audioDevices.subscribe((v) => (devices = v));
  availableModels.subscribe((v) => (models = v));
  installedModels.subscribe((v) => (installed = v));
  settings.subscribe((v) => (currentSettings = v));
  isModelLoaded.subscribe((v) => (modelLoaded = v));

  onMount(async () => {
    await refreshDevices();
    const installedList = await refreshModels();

    // Auto-load previously selected model if it exists
    if (currentSettings.selectedModel && installedList) {
      const modelExists = installedList.some(m => m.path === currentSettings.selectedModel);
      if (modelExists) {
        await loadModel(currentSettings.selectedModel);
      }
    }
  });

  async function refreshDevices() {
    try {
      const result = await invoke<AudioDevice[]>('list_audio_devices');
      audioDevices.set(result);
    } catch (e) {
      console.error('Failed to list audio devices:', e);
      error = 'Failed to list audio devices';
    }
  }

  async function refreshModels(): Promise<ModelInfo[] | null> {
    try {
      const available = await invoke<ModelInfo[]>('list_models');
      const installedList = await invoke<ModelInfo[]>('list_installed_models');
      availableModels.set(available);
      installedModels.set(installedList);
      return installedList;
    } catch (e) {
      console.error('Failed to list models:', e);
      return null;
    }
  }

  async function loadModel(modelPath: string) {
    loading = true;
    error = '';

    try {
      await invoke('load_model', { modelPath });
      isModelLoaded.set(true);
      currentSettings.selectedModel = modelPath;
      saveSettings(currentSettings);
    } catch (e) {
      error = `Failed to load model: ${e}`;
      isModelLoaded.set(false);
    } finally {
      loading = false;
    }
  }

  function handleDeviceChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    currentSettings.selectedDevice = target.value || null;
    saveSettings(currentSettings);
  }

  function handleModeChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    currentSettings.recordingMode = target.value as 'toggle' | 'push-to-talk';
    saveSettings(currentSettings);
  }

  function handleHotkeyChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    currentSettings.pushToTalkKey = target.value;
    saveSettings(currentSettings);
  }

  function handleThemeChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    currentSettings.theme = target.value as 'light' | 'dark' | 'system';
    saveSettings(currentSettings);
    applyTheme(currentSettings.theme);
  }

  function applyTheme(theme: 'light' | 'dark' | 'system') {
    if (typeof window === 'undefined') return;

    const root = document.documentElement;
    if (theme === 'system') {
      root.removeAttribute('data-theme');
    } else {
      root.setAttribute('data-theme', theme);
    }
  }

  function close() {
    isOpen = false;
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="settings-overlay" on:click={close} on:keydown={(e) => e.key === 'Escape' && close()} role="dialog" aria-modal="true" tabindex="-1">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="settings-panel" on:click|stopPropagation>
      <header>
        <h2>Settings</h2>
        <button class="close-button" on:click={close} aria-label="Close settings">
          <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
            <path
              d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
            />
          </svg>
        </button>
      </header>

      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      <section>
        <h3>Audio</h3>

        <label>
          <span>Microphone</span>
          <select value={currentSettings.selectedDevice || ''} on:change={handleDeviceChange}>
            <option value="">Default</option>
            {#each devices as device}
              <option value={device.name}>
                {device.name}
                {device.is_default ? '(Default)' : ''}
              </option>
            {/each}
          </select>
        </label>

        <button class="refresh-button" on:click={refreshDevices}>Refresh Devices</button>
      </section>

      <section>
        <h3>Speech Recognition</h3>

        <div class="model-status">
          Status:
          {#if loading}
            <span class="loading">Loading...</span>
          {:else if modelLoaded}
            <span class="loaded">Model loaded</span>
          {:else}
            <span class="not-loaded">No model loaded</span>
          {/if}
        </div>

        {#if installed.length > 0}
          <div class="model-list">
            <h4>Installed Models</h4>
            {#each installed as model}
              <div class="model-item">
                <span>{model.name}</span>
                <span class="model-lang">{model.language}</span>
                <button
                  on:click={() => loadModel(model.path)}
                  disabled={loading || currentSettings.selectedModel === model.path}
                >
                  {currentSettings.selectedModel === model.path ? 'Loaded' : 'Load'}
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <p class="no-models">
            No models installed. Download a Vosk model and place it in the models directory.
          </p>
        {/if}

        <div class="model-list">
          <h4>Available Models</h4>
          {#each models as model}
            <div class="model-item">
              <span>{model.name}</span>
              <span class="model-size">{model.size_mb} MB</span>
            </div>
          {/each}
        </div>
      </section>

      <section>
        <h3>Voice Input</h3>

        <label>
          <span>Recording Mode</span>
          <select value={currentSettings.recordingMode} on:change={handleModeChange}>
            <option value="toggle">Toggle (click to start/stop)</option>
            <option value="push-to-talk">Push to Talk (hold key)</option>
          </select>
        </label>

        {#if currentSettings.recordingMode === 'push-to-talk'}
          <label>
            <span>Push to Talk Key</span>
            <select value={currentSettings.pushToTalkKey} on:change={handleHotkeyChange}>
              <option value="Space">Space</option>
              <option value="ControlLeft">Left Ctrl</option>
              <option value="ControlRight">Right Ctrl</option>
              <option value="AltLeft">Left Alt</option>
              <option value="ShiftLeft">Left Shift</option>
            </select>
          </label>
        {/if}
      </section>

      <section>
        <h3>Appearance</h3>

        <label>
          <span>Theme</span>
          <select value={currentSettings.theme} on:change={handleThemeChange}>
            <option value="system">System</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </label>
      </section>
    </div>
  </div>
{/if}

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .settings-panel {
    background-color: var(--bg-color, #ffffff);
    color: var(--text-color, #1f2937);
    border-radius: 12px;
    padding: 24px;
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }

  h2 {
    margin: 0;
    font-size: 24px;
  }

  h3 {
    margin: 0 0 12px 0;
    font-size: 16px;
    color: var(--text-muted, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  h4 {
    margin: 8px 0;
    font-size: 14px;
  }

  .close-button {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    color: var(--text-muted, #6b7280);
    border-radius: 4px;
  }

  .close-button:hover {
    background-color: var(--bg-hover, #f3f4f6);
  }

  section {
    margin-bottom: 24px;
    padding-bottom: 24px;
    border-bottom: 1px solid var(--border-color, #e5e7eb);
  }

  section:last-child {
    border-bottom: none;
    margin-bottom: 0;
    padding-bottom: 0;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
  }

  label span {
    font-size: 14px;
    font-weight: 500;
  }

  select {
    padding: 8px 12px;
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 6px;
    background-color: var(--input-bg, #ffffff);
    color: var(--text-color, #1f2937);
    font-size: 14px;
  }

  .refresh-button {
    padding: 8px 16px;
    background-color: var(--bg-secondary, #f3f4f6);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .refresh-button:hover {
    background-color: var(--bg-hover, #e5e7eb);
  }

  .model-status {
    margin-bottom: 12px;
    font-size: 14px;
  }

  .loading {
    color: #f59e0b;
  }

  .loaded {
    color: #10b981;
  }

  .not-loaded {
    color: #6b7280;
  }

  .model-list {
    margin-top: 12px;
  }

  .model-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    background-color: var(--bg-secondary, #f9fafb);
    border-radius: 6px;
    margin-bottom: 8px;
  }

  .model-item span:first-child {
    flex: 1;
    font-size: 14px;
  }

  .model-lang,
  .model-size {
    font-size: 12px;
    color: var(--text-muted, #6b7280);
  }

  .model-item button {
    padding: 4px 12px;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .model-item button:disabled {
    background-color: #9ca3af;
    cursor: not-allowed;
  }

  .no-models {
    font-size: 14px;
    color: var(--text-muted, #6b7280);
    font-style: italic;
  }

  .error-message {
    background-color: #fef2f2;
    color: #dc2626;
    padding: 12px;
    border-radius: 6px;
    margin-bottom: 16px;
    font-size: 14px;
  }

  @media (prefers-color-scheme: dark) {
    .settings-panel {
      --bg-color: #1f2937;
      --text-color: #f9fafb;
      --text-muted: #9ca3af;
      --border-color: #374151;
      --bg-secondary: #374151;
      --bg-hover: #4b5563;
      --input-bg: #374151;
    }
  }

  :global([data-theme='dark']) .settings-panel {
    --bg-color: #1f2937;
    --text-color: #f9fafb;
    --text-muted: #9ca3af;
    --border-color: #374151;
    --bg-secondary: #374151;
    --bg-hover: #4b5563;
    --input-bg: #374151;
  }

  :global([data-theme='light']) .settings-panel {
    --bg-color: #ffffff;
    --text-color: #1f2937;
    --text-muted: #6b7280;
    --border-color: #e5e7eb;
    --bg-secondary: #f9fafb;
    --bg-hover: #f3f4f6;
    --input-bg: #ffffff;
  }
</style>
