import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Terminal from '$lib/Terminal.svelte';
import { claudeStatus, terminalOutput } from '$lib/stores/app';

// Mock xterm.js
vi.mock('@xterm/xterm', () => ({
  Terminal: vi.fn().mockImplementation(() => ({
    loadAddon: vi.fn(),
    open: vi.fn(),
    onData: vi.fn(),
    write: vi.fn(),
    dispose: vi.fn(),
    focus: vi.fn(),
    clear: vi.fn(),
  })),
}));

vi.mock('@xterm/addon-fit', () => ({
  FitAddon: vi.fn().mockImplementation(() => ({
    fit: vi.fn(),
    proposeDimensions: vi.fn(() => ({ cols: 80, rows: 24 })),
  })),
}));

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
    // Wait for async operations
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(invoke).toHaveBeenCalledWith('start_claude', expect.any(Object));
  });

  it('should listen for claude-output events', async () => {
    const { listen } = await import('@tauri-apps/api/event');
    render(Terminal);
    expect(listen).toHaveBeenCalledWith('claude-output', expect.any(Function));
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
