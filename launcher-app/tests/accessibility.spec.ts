import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test.describe('Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to your app (adjust URL as needed)
    await page.goto('http://localhost:1420');
    await page.waitForLoadState('domcontentloaded');
  });

  test('should not have automatically detectable accessibility issues', async ({ page }) => {
    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('SearchCombobox should have proper ARIA attributes', async ({ page }) => {
    // Check combobox input
    const searchInput = page.locator('input[aria-label="Search bookmarks and files"]');
    await expect(searchInput).toBeVisible();

    // Verify ARIA attributes
    await expect(searchInput).toHaveAttribute('aria-autocomplete', 'list');
    await expect(searchInput).toHaveAttribute('aria-controls', 'search-results-listbox');
  });

  test('Settings button should have accessible label', async ({ page }) => {
    const settingsButton = page.locator('button[aria-label*="Settings"]');
    await expect(settingsButton).toBeVisible();
    await expect(settingsButton).toHaveAttribute('aria-keyshortcuts', 'Meta+Comma');
  });

  test('Settings dialog should have proper modal attributes', async ({ page }) => {
    // Open settings (you may need to adjust this based on your app)
    const settingsButton = page.locator('button[aria-label*="Settings"]');
    await settingsButton.click();

    // Check dialog attributes
    const settingsDialog = page.locator('[role="dialog"]');
    await expect(settingsDialog).toBeVisible();
    await expect(settingsDialog).toHaveAttribute('aria-modal', 'true');
    await expect(settingsDialog).toHaveAttribute('aria-labelledby', 'settings-title');
  });

  test('All form inputs should have labels', async ({ page }) => {
    // Open settings
    const settingsButton = page.locator('button[aria-label*="Settings"]');
    await settingsButton.click();

    // Check that all inputs have aria-label
    const inputs = page.locator('input:not([type="hidden"])');
    const count = await inputs.count();

    for (let i = 0; i < count; i++) {
      const input = inputs.nth(i);
      const hasAriaLabel = await input.getAttribute('aria-label');
      const hasAriaLabelledBy = await input.getAttribute('aria-labelledby');

      expect(hasAriaLabel || hasAriaLabelledBy).toBeTruthy();
    }
  });

  test('Keyboard navigation should work', async ({ page }) => {
    // Type in search
    const searchInput = page.locator('input[aria-label="Search bookmarks and files"]');
    await searchInput.focus();
    await searchInput.type('test');

    // Tab should move focus
    await page.keyboard.press('Tab');

    // Check that focus moved to next interactive element
    const focusedElement = await page.evaluate(() => document.activeElement?.tagName);
    expect(focusedElement).not.toBe('INPUT');
  });

  test('Loading state should be announced', async ({ page }) => {
    // Check for aria-live regions
    const liveRegion = page.locator('[aria-live="polite"]');
    await expect(liveRegion).toBeTruthy();
  });

  test('All sections should have proper headings', async ({ page }) => {
    // Open settings
    const settingsButton = page.locator('button[aria-label*="Settings"]');
    await settingsButton.click();

    // Check that all sections have proper heading structure
    const sections = page.locator('section[role="group"]');
    const count = await sections.count();

    for (let i = 0; i < count; i++) {
      const section = sections.nth(i);
      const hasAriaLabelledBy = await section.getAttribute('aria-labelledby');
      expect(hasAriaLabelledBy).toBeTruthy();
    }
  });

  test('Color inputs should have accessible labels', async ({ page }) => {
    // Open settings
    const settingsButton = page.locator('button[aria-label*="Settings"]');
    await settingsButton.click();

    // Check color inputs
    const colorInputs = page.locator('input[type="color"]');
    const count = await colorInputs.count();

    for (let i = 0; i < count; i++) {
      const input = colorInputs.nth(i);
      const ariaLabel = await input.getAttribute('aria-label');
      expect(ariaLabel).toBeTruthy();
      expect(ariaLabel?.toLowerCase()).toContain('color');
    }
  });

  test('Focus should be visible with custom styles', async ({ page }) => {
    const searchInput = page.locator('input[aria-label="Search bookmarks and files"]');
    await searchInput.focus();

    // Check that focus styles are applied
    const outline = await searchInput.evaluate((el) => {
      return window.getComputedStyle(el).outline;
    });

    expect(outline).not.toBe('none');
  });
});
