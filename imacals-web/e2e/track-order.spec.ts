import { test, expect, type Route } from '@playwright/test';

function ok(body: unknown) {
  return {
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: body }),
  };
}

test.describe('Order tracking', () => {
  test('renders the tracking form', async ({ page }) => {
    await page.goto('/track');
    await expect(page.getByRole('heading', { name: 'Where is my order?' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Track order' })).toBeDisabled();
  });

  test('shows the status and the timeline for a known reference', async ({ page }) => {
    await page.route('**/api/orders/IMC-000123/track', (route: Route) =>
      route.fulfill(ok({
        reference: 'IMC-000123',
        status: 'Out for delivery',
        placed_at: '2026-08-05T09:00:00Z',
        total_kobo: 45_000_000,
        history: [
          { status: 'Order placed',     note: null,               occurred_at: '2026-08-05T09:00:00Z' },
          { status: 'Picked in Aba',    note: 'Loaded on truck 3', occurred_at: '2026-08-06T07:30:00Z' },
          { status: 'Out for delivery', note: null,               occurred_at: '2026-08-07T06:15:00Z' },
        ],
      })));

    await page.goto('/track');
    await page.getByLabel('Order reference').fill('IMC-000123');
    await page.getByRole('button', { name: 'Track order' }).click();

    await expect(page.getByRole('heading', { name: 'Out for delivery' })).toBeVisible();
    await expect(page.locator('.event')).toHaveCount(3);
  });

  test('shows an error for an unknown reference', async ({ page }) => {
    await page.route('**/api/orders/*/track', (route: Route) =>
      route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({
          success: 'false', code: 'NotFound',
          error: { message: 'Order not found' },
        }),
      }));

    await page.goto('/track');
    await page.getByLabel('Order reference').fill('IMC-999999');
    await page.getByRole('button', { name: 'Track order' }).click();

    await expect(page.locator('.state-msg--error')).toHaveText('Order not found');
  });
});

test.describe('Delivery page', () => {
  test('states the Aba warehouse and the coverage table', async ({ page }) => {
    await page.goto('/delivery');
    await expect(page.getByRole('heading', { name: 'Everything ships from Aba' })).toBeVisible();
    await expect(page.locator('.table tbody tr')).toHaveCount(4);
  });

  test('documents both ordering channels', async ({ page }) => {
    await page.goto('/delivery');
    await expect(page.getByText('Online', { exact: true })).toBeVisible();
    await expect(page.getByText('By phone', { exact: true })).toBeVisible();
  });
});

test.describe('Not found', () => {
  test('unknown routes render the 404 view', async ({ page }) => {
    await page.goto('/no-such-page');
    await expect(page.getByRole('heading', { name: 'That page is not here' })).toBeVisible();
  });
});
