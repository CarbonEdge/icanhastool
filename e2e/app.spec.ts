import { test, expect } from '@playwright/test';

test.describe('icanhastool Application', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display app header', async ({ page }) => {
    await expect(page.locator('h1')).toHaveText('icanhastool');
  });

  test('should show Claude status indicator', async ({ page }) => {
    const statusIndicator = page.locator('.status-indicator');
    await expect(statusIndicator).toBeVisible();
  });

  test('should have settings button', async ({ page }) => {
    const settingsButton = page.locator('button[aria-label="Open settings"]');
    await expect(settingsButton).toBeVisible();
  });

  test('should open settings panel when clicking settings button', async ({ page }) => {
    await page.click('button[aria-label="Open settings"]');
    await expect(page.locator('text=Settings')).toBeVisible();
  });

  test('should close settings panel when clicking close button', async ({ page }) => {
    await page.click('button[aria-label="Open settings"]');
    await expect(page.locator('.settings-panel')).toBeVisible();

    await page.click('button[aria-label="Close settings"]');
    await expect(page.locator('.settings-panel')).not.toBeVisible();
  });

  test('should close settings when clicking overlay', async ({ page }) => {
    await page.click('button[aria-label="Open settings"]');
    await expect(page.locator('.settings-panel')).toBeVisible();

    // Click on the overlay (outside the panel)
    await page.click('.settings-overlay', { position: { x: 10, y: 10 } });
    await expect(page.locator('.settings-panel')).not.toBeVisible();
  });

  test('should have terminal section', async ({ page }) => {
    const terminalSection = page.locator('.terminal-section');
    await expect(terminalSection).toBeVisible();
  });

  test('should have voice control section', async ({ page }) => {
    const voiceSection = page.locator('.voice-section');
    await expect(voiceSection).toBeVisible();
  });

  test('should show record button', async ({ page }) => {
    const recordButton = page.locator('.record-button');
    await expect(recordButton).toBeVisible();
  });

  test('should show model warning when no model loaded', async ({ page }) => {
    const warning = page.locator('.model-warning');
    await expect(warning).toBeVisible();
    await expect(warning).toContainText('Load a speech model');
  });
});

test.describe('Settings Panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('button[aria-label="Open settings"]');
  });

  test('should display Audio section', async ({ page }) => {
    await expect(page.locator('text=Audio')).toBeVisible();
    await expect(page.locator('text=Microphone')).toBeVisible();
  });

  test('should display Speech Recognition section', async ({ page }) => {
    await expect(page.locator('text=Speech Recognition')).toBeVisible();
  });

  test('should display Voice Input section', async ({ page }) => {
    await expect(page.locator('text=Voice Input')).toBeVisible();
    await expect(page.locator('text=Recording Mode')).toBeVisible();
  });

  test('should display Appearance section', async ({ page }) => {
    await expect(page.locator('text=Appearance')).toBeVisible();
    await expect(page.locator('text=Theme')).toBeVisible();
  });

  test('should have theme options', async ({ page }) => {
    const themeSelect = page.locator('select').filter({ hasText: 'System' });
    await expect(themeSelect).toBeVisible();
  });

  test('should have recording mode options', async ({ page }) => {
    const modeSelect = page.locator('select').filter({ hasText: 'Toggle' });
    await expect(modeSelect).toBeVisible();
  });

  test('should show push-to-talk key option when mode is push-to-talk', async ({ page }) => {
    const modeSelect = page.locator('select').filter({ hasText: 'Toggle' });
    await modeSelect.selectOption('push-to-talk');

    await expect(page.locator('text=Push to Talk Key')).toBeVisible();
  });
});

test.describe('Accessibility', () => {
  test('settings button should have aria-label', async ({ page }) => {
    await page.goto('/');
    const settingsButton = page.locator('button[aria-label="Open settings"]');
    await expect(settingsButton).toHaveAttribute('aria-label', 'Open settings');
  });

  test('close button should have aria-label', async ({ page }) => {
    await page.goto('/');
    await page.click('button[aria-label="Open settings"]');

    const closeButton = page.locator('button[aria-label="Close settings"]');
    await expect(closeButton).toHaveAttribute('aria-label', 'Close settings');
  });

  test('record button should have title', async ({ page }) => {
    await page.goto('/');
    const recordButton = page.locator('.record-button');
    await expect(recordButton).toHaveAttribute('title');
  });
});
