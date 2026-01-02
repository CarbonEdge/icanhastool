<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { claudeStatus, terminalOutput, appendTerminalOutput } from './stores/app';
  import '@xterm/xterm/css/xterm.css';

  let terminalContainer: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let unlisten: UnlistenFn | null = null;
  let resizeObserver: ResizeObserver | null = null;

  // Props
  export let workingDir: string | undefined = undefined;

  // Initialize terminal
  onMount(async () => {
    terminal = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Consolas, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#ffffff',
        cursorAccent: '#000000',
        selectionBackground: '#264f78',
      },
      scrollback: 10000,
      convertEol: true,
    });

    fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);

    terminal.open(terminalContainer);
    fitAddon.fit();

    // Handle terminal input
    terminal.onData((data) => {
      invoke('send_to_claude', { input: data }).catch((e) => {
        console.error('Failed to send input:', e);
      });
    });

    // Listen for Claude output events
    unlisten = await listen<{ data: string; is_error: boolean }>('claude-output', (event) => {
      if (terminal) {
        terminal.write(event.payload.data);
        appendTerminalOutput(event.payload.data);
      }
    });

    // Handle window resize
    resizeObserver = new ResizeObserver(() => {
      if (fitAddon && terminal) {
        fitAddon.fit();
        const dims = fitAddon.proposeDimensions();
        if (dims) {
          invoke('resize_claude', { cols: dims.cols, rows: dims.rows }).catch(() => {
            // Ignore resize errors when not running
          });
        }
      }
    });
    resizeObserver.observe(terminalContainer);

    // Start Claude Code
    await startClaude();
  });

  onDestroy(() => {
    if (unlisten) unlisten();
    if (resizeObserver) resizeObserver.disconnect();
    if (terminal) terminal.dispose();
    stopClaude();
  });

  async function startClaude() {
    try {
      await invoke('start_claude', { workingDir });
      claudeStatus.set('Running');

      // Set initial size
      if (fitAddon) {
        const dims = fitAddon.proposeDimensions();
        if (dims) {
          await invoke('resize_claude', { cols: dims.cols, rows: dims.rows });
        }
      }
    } catch (e) {
      console.error('Failed to start Claude:', e);
      claudeStatus.set({ Error: String(e) });
    }
  }

  async function stopClaude() {
    try {
      await invoke('stop_claude');
      claudeStatus.set('Stopped');
    } catch (e) {
      console.error('Failed to stop Claude:', e);
    }
  }

  // Send transcribed text to Claude
  export async function sendText(text: string) {
    if (!text.trim()) return;

    try {
      // Echo the command in the terminal
      if (terminal) {
        terminal.write(`\r\n> ${text}\r\n`);
      }
      await invoke('send_to_claude', { input: text + '\n' });
    } catch (e) {
      console.error('Failed to send text:', e);
    }
  }

  // Focus the terminal
  export function focus() {
    terminal?.focus();
  }

  // Clear terminal
  export function clear() {
    terminal?.clear();
  }
</script>

<div class="terminal-wrapper" bind:this={terminalContainer}></div>

<style>
  .terminal-wrapper {
    width: 100%;
    height: 100%;
    background-color: #1e1e1e;
    border-radius: 4px;
    overflow: hidden;
  }

  :global(.terminal-wrapper .xterm) {
    padding: 8px;
    height: 100%;
  }

  :global(.terminal-wrapper .xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
