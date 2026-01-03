<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import Terminal from '$lib/Terminal.svelte';
  import VoiceControl from '$lib/VoiceControl.svelte';
  import Settings from '$lib/Settings.svelte';
  import Waveform from '$lib/Waveform.svelte';
  import {
    loadSettings,
    claudeStatus,
    isClaudeRunning,
    isModelLoaded,
    settings,
    addRecentWorkspace,
    type AppSettings,
  } from '$lib/stores/app';

  let terminal: Terminal;
  let settingsOpen = false;
  let claudeRunning = false;
  let modelLoaded = false;
  let currentSettings: AppSettings;
  let showRecentWorkspaces = false;

  isClaudeRunning.subscribe((v) => (claudeRunning = v));
  isModelLoaded.subscribe((v) => (modelLoaded = v));
  settings.subscribe((v) => (currentSettings = v));

  onMount(() => {
    loadSettings();
  });

  async function selectWorkspace() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Workspace Directory',
    });

    if (selected && typeof selected === 'string') {
      addRecentWorkspace(selected);
      // Terminal will restart with new working directory
      // Force a re-render by binding to the key
    }
  }

  function selectRecentWorkspace(workspace: string) {
    addRecentWorkspace(workspace);
    showRecentWorkspaces = false;
  }

  function getWorkspaceDisplayName(path: string | null): string {
    if (!path) return 'No workspace';
    // Get the last folder name from the path
    const parts = path.replace(/\\/g, '/').split('/').filter(Boolean);
    return parts[parts.length - 1] || path;
  }

  function handleTranscription(event: CustomEvent<string>) {
    const text = event.detail;
    if (terminal && text.trim()) {
      terminal.sendText(text);
    }
  }

  function openSettings() {
    settingsOpen = true;
  }
</script>

<div class="app">
  <header class="app-header">
    <h1>icanhastool</h1>

    <div class="workspace-section">
      <div class="workspace-picker">
        <button class="workspace-button" on:click={selectWorkspace} aria-label="Select workspace folder">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" />
          </svg>
          <span class="workspace-name" title={currentSettings?.currentWorkspace || 'No workspace'}>
            {getWorkspaceDisplayName(currentSettings?.currentWorkspace)}
          </span>
        </button>

        {#if currentSettings?.recentWorkspaces?.length > 0}
          <div class="recent-dropdown">
            <button
              class="dropdown-toggle"
              on:click={() => (showRecentWorkspaces = !showRecentWorkspaces)}
              aria-label="Show recent workspaces"
              aria-expanded={showRecentWorkspaces}
            >
              <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                <path d="M7 10l5 5 5-5z" />
              </svg>
            </button>

            {#if showRecentWorkspaces}
              <div class="dropdown-menu">
                <div class="dropdown-header">Recent Workspaces</div>
                {#each currentSettings.recentWorkspaces as workspace}
                  <button
                    class="dropdown-item"
                    on:click={() => selectRecentWorkspace(workspace)}
                    title={workspace}
                  >
                    {getWorkspaceDisplayName(workspace)}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <div class="header-status">
      <span class="status-indicator" class:running={claudeRunning}>
        {claudeRunning ? 'Claude Running' : 'Claude Stopped'}
      </span>
      <button class="settings-button" on:click={openSettings} aria-label="Open settings">
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
          <path
            d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"
          />
        </svg>
      </button>
    </div>
  </header>

  <main class="app-main">
    <div class="terminal-section">
      {#key currentSettings?.currentWorkspace}
        <Terminal
          bind:this={terminal}
          workingDir={currentSettings?.currentWorkspace ?? undefined}
          fontSize={currentSettings?.fontSize ?? 1}
        />
      {/key}
    </div>

    <aside class="voice-section">
      <VoiceControl on:transcription={handleTranscription} />
      <Waveform />

      {#if !modelLoaded}
        <div class="model-warning">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path
              d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"
            />
          </svg>
          <span>Load a speech model in settings to enable voice input</span>
        </div>
      {/if}
    </aside>
  </main>

  <Settings bind:isOpen={settingsOpen} />
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell,
      sans-serif;
    background-color: var(--bg-primary, #0f172a);
    color: var(--text-primary, #f8fafc);
    overflow: hidden;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 20px;
    background-color: var(--bg-secondary, #1e293b);
    border-bottom: 1px solid var(--border-color, #334155);
  }

  h1 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary, #f8fafc);
  }

  .workspace-section {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .workspace-picker {
    display: flex;
    align-items: center;
  }

  .workspace-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background-color: var(--bg-tertiary, #334155);
    border: 1px solid var(--border-color, #334155);
    border-radius: 6px;
    color: var(--text-primary, #f8fafc);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  .workspace-picker:has(.recent-dropdown) .workspace-button {
    border-radius: 6px 0 0 6px;
  }

  .workspace-button:hover {
    background-color: var(--bg-hover, #475569);
  }

  .workspace-name {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recent-dropdown {
    position: relative;
  }

  .dropdown-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px 8px;
    background-color: var(--bg-tertiary, #334155);
    border: 1px solid var(--border-color, #334155);
    border-left: none;
    border-radius: 0 6px 6px 0;
    color: var(--text-muted, #94a3b8);
    cursor: pointer;
    transition: all 0.2s;
  }

  .dropdown-toggle:hover {
    background-color: var(--bg-hover, #475569);
    color: var(--text-primary, #f8fafc);
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    min-width: 250px;
    max-width: 400px;
    background-color: var(--bg-secondary, #1e293b);
    border: 1px solid var(--border-color, #334155);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 100;
    overflow: hidden;
  }

  .dropdown-header {
    padding: 8px 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted, #94a3b8);
    border-bottom: 1px solid var(--border-color, #334155);
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 10px 12px;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-primary, #f8fafc);
    cursor: pointer;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: background-color 0.15s;
  }

  .dropdown-item:hover {
    background-color: var(--bg-tertiary, #334155);
  }

  .header-status {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .status-indicator {
    font-size: 12px;
    padding: 4px 12px;
    border-radius: 9999px;
    background-color: var(--bg-tertiary, #334155);
    color: var(--text-muted, #94a3b8);
  }

  .status-indicator.running {
    background-color: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .settings-button {
    background: none;
    border: none;
    cursor: pointer;
    padding: 8px;
    border-radius: 6px;
    color: var(--text-muted, #94a3b8);
    transition: all 0.2s;
  }

  .settings-button:hover {
    background-color: var(--bg-tertiary, #334155);
    color: var(--text-primary, #f8fafc);
  }

  .app-main {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .terminal-section {
    flex: 1;
    padding: 16px;
    overflow: hidden;
  }

  .voice-section {
    width: 280px;
    padding: 16px;
    background-color: var(--bg-secondary, #1e293b);
    border-left: 1px solid var(--border-color, #334155);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .model-warning {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 12px;
    background-color: rgba(251, 191, 36, 0.1);
    border: 1px solid rgba(251, 191, 36, 0.3);
    border-radius: 8px;
    font-size: 12px;
    color: #fbbf24;
  }

  .model-warning svg {
    flex-shrink: 0;
    margin-top: 2px;
  }

  @media (prefers-color-scheme: light) {
    :global(body) {
      --bg-primary: #ffffff;
      --bg-secondary: #f8fafc;
      --bg-tertiary: #e2e8f0;
      --text-primary: #0f172a;
      --text-muted: #64748b;
      --border-color: #e2e8f0;
    }
  }

  :global([data-theme='light']) {
    --bg-primary: #ffffff;
    --bg-secondary: #f8fafc;
    --bg-tertiary: #e2e8f0;
    --text-primary: #0f172a;
    --text-muted: #64748b;
    --border-color: #e2e8f0;
  }

  :global([data-theme='dark']) {
    --bg-primary: #0f172a;
    --bg-secondary: #1e293b;
    --bg-tertiary: #334155;
    --text-primary: #f8fafc;
    --text-muted: #94a3b8;
    --border-color: #334155;
  }

  /* High Contrast Dark - Maximum contrast for accessibility */
  :global([data-theme='high-contrast-dark']) {
    --bg-primary: #000000;
    --bg-secondary: #0a0a0a;
    --bg-tertiary: #1a1a1a;
    --bg-hover: #2a2a2a;
    --text-primary: #ffffff;
    --text-muted: #e0e0e0;
    --border-color: #ffffff;
    --focus-ring: #00ffff;
    --accent-color: #00ff00;
    --error-color: #ff6b6b;
    --success-color: #00ff00;
    --warning-color: #ffff00;
  }

  /* High Contrast Light - Maximum contrast for accessibility */
  :global([data-theme='high-contrast-light']) {
    --bg-primary: #ffffff;
    --bg-secondary: #f5f5f5;
    --bg-tertiary: #e0e0e0;
    --bg-hover: #d0d0d0;
    --text-primary: #000000;
    --text-muted: #1a1a1a;
    --border-color: #000000;
    --focus-ring: #0000ff;
    --accent-color: #0000cc;
    --error-color: #cc0000;
    --success-color: #006600;
    --warning-color: #996600;
  }

  /* Enhanced focus visibility for high contrast modes */
  :global([data-theme='high-contrast-dark']) :focus,
  :global([data-theme='high-contrast-light']) :focus {
    outline: 3px solid var(--focus-ring) !important;
    outline-offset: 2px !important;
  }

  :global([data-theme='high-contrast-dark']) button,
  :global([data-theme='high-contrast-light']) button {
    border: 2px solid var(--border-color) !important;
  }

  :global([data-theme='high-contrast-dark']) .status-indicator.running,
  :global([data-theme='high-contrast-light']) .status-indicator.running {
    background-color: var(--success-color);
    color: var(--bg-primary);
  }
</style>
