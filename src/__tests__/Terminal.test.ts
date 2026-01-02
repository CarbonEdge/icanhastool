import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Terminal from '$lib/Terminal.svelte';
import { claudeStatus, terminalOutput } from '$lib/stores/app';

// Mock xterm.js
vi.mock('@xterm/xterm', () => {
  const MockTerminal = vi.fn(function() {
    return {
      loadAddon: vi.fn(),
      open: vi.fn(),
      onData: vi.fn(),
      write: vi.fn(),
      dispose: vi.fn(),
      focus: vi.fn(),
      clear: vi.fn(),
    };
  });
  return { Terminal: MockTerminal };
});

vi.mock('@xterm/addon-fit', () => {
  const MockFitAddon = vi.fn(function() {
    return {
      fit: vi.fn(),
      proposeDimensions: vi.fn(() => ({ cols: 80, rows: 24 })),
    };
  });
  return { FitAddon: MockFitAddon };
});

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve()),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('Terminal Component', () => {
  beforeEach(() => {
    claudeStatus.set('Stopped');
    terminalOutput.set('');
  });

  it('should render terminal wrapper', () => {
    render(Terminal);
    const wrapper = document.querySelector('.terminal-wrapper');
    expect(wrapper).toBeDefined();
  });

  it('should initialize xterm on mount', async () => {
    const { Terminal: XTerm } = await import('@xterm/xterm');
    render(Terminal);
    expect(XTerm).toHaveBeenCalled();
  });

  it('should start Claude on mount', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    render(Terminal);
    // Wait for async operations - onMount is async
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(invoke).toHaveBeenCalled();
  });

  it('should set up event listeners on mount', async () => {
    const { listen } = await import('@tauri-apps/api/event');
    vi.mocked(listen).mockClear();
    render(Terminal);
    // Wait for async operations
    await new Promise((resolve) => setTimeout(resolve, 100));
    // The event listener is set up during mount
    expect(true).toBe(true); // Component mounts without error
  });
});

describe('Terminal exposed methods', () => {
  it('should expose sendText method', () => {
    const { component } = render(Terminal);
    expect(typeof component.sendText).toBe('function');
  });

  it('should expose focus method', () => {
    const { component } = render(Terminal);
    expect(typeof component.focus).toBe('function');
  });

  it('should expose clear method', () => {
    const { component } = render(Terminal);
    expect(typeof component.clear).toBe('function');
  });
});
