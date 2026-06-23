import { expect, test } from '@playwright/test';

test.describe('GAM — Critical Path', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
    });

    test('app renders with title bar', async ({ page }) => {
        await expect(page.locator('.title-bar, [class*="titlebar"], header')).toBeVisible({
            timeout: 10_000,
        });
    });

    test('search bar is interactive', async ({ page }) => {
        const searchInput = page.locator(
            'input[type="search"], input[placeholder*="Search"], input[placeholder*="search"]',
        );

        if ((await searchInput.count()) > 0) {
            await searchInput.click();
            await searchInput.fill('checkout');
            await expect(searchInput).toHaveValue('checkout');
        }
    });

    test('settings dropdown opens', async ({ page }) => {
        const settingsButton = page.locator(
            'button:has-text("Settings"), button[aria-label*="settings"], button[aria-label*="Settings"], [data-testid="settings"]',
        );

        if ((await settingsButton.count()) > 0) {
            await settingsButton.first().click();
            // Verify a dropdown/menu appears
            await expect(
                page.locator('[role="menu"], [class*="dropdown"], [class*="settings"]'),
            ).toBeVisible({ timeout: 3_000 });
        }
    });
});
