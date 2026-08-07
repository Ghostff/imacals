import { test, expect, type Page, type Route } from '@playwright/test';

const RICE = {
  id: 'p1', slug: 'rice-50kg', name: 'Long Grain Rice — 50kg Bag',
  description: 'Parboiled long grain rice.',
  category_slug: 'foodstuff', category_name: 'Foodstuff',
  unit: 'bag (50kg)', unit_price_kobo: 8_950_000, min_order_quantity: 5,
  in_stock: true, image_url: null,
};

function ok(body: unknown) {
  return {
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: body }),
  };
}

// Seeds one cart line without walking the product page each time. goto() first: localStorage is
// unreachable on about:blank.
async function seedCart(page: Page, quantity: number): Promise<void> {
  await page.route('**/api/catalog/products*', (route: Route) => route.fulfill(ok([RICE])));
  await page.route('**/api/catalog/categories', (route: Route) => route.fulfill(ok([])));
  await page.goto('/catalog');
  await page.evaluate(
    ([product, qty]) => localStorage.setItem('cart', JSON.stringify([{ product, quantity: qty }])),
    [RICE, quantity] as const,
  );
}

test.describe('Cart', () => {
  test('empty cart shows an empty state', async ({ page }) => {
    await page.route('**/api/catalog/products*', (route: Route) => route.fulfill(ok([RICE])));
    await page.route('**/api/catalog/categories', (route: Route) => route.fulfill(ok([])));
    await page.goto('/cart');
    await expect(page.getByText('Your cart is empty.')).toBeVisible();
  });

  test('renders a line per cart item with the correct line total', async ({ page }) => {
    await seedCart(page, 5);
    await page.goto('/cart');
    await expect(page.locator('.line')).toHaveCount(1);
    // 5 × ₦89,500 = ₦447,500
    await expect(page.locator('.line-total')).toContainText('447,500');
  });

  test('changing quantity updates the subtotal', async ({ page }) => {
    await seedCart(page, 5);
    await page.goto('/cart');
    await page.locator('.qty-input').fill('10');
    await page.locator('.qty-input').blur();
    await expect(page.locator('.summary-value').nth(1)).toContainText('895,000');
  });

  test('dropping below the minimum order quantity removes the line', async ({ page }) => {
    await seedCart(page, 5);
    await page.goto('/cart');
    await page.locator('.qty-input').fill('2');
    await page.locator('.qty-input').blur();
    await expect(page.getByText('Your cart is empty.')).toBeVisible();
  });

  test('remove empties the cart', async ({ page }) => {
    await seedCart(page, 5);
    await page.goto('/cart');
    await page.getByRole('button', { name: 'Remove' }).click();
    await expect(page.getByText('Your cart is empty.')).toBeVisible();
  });

  test('cart survives a reload', async ({ page }) => {
    await seedCart(page, 5);
    await page.goto('/cart');
    await page.reload();
    await expect(page.locator('.line')).toHaveCount(1);
  });
});

test.describe('Checkout', () => {
  test('checkout with an empty cart offers the catalogue instead', async ({ page }) => {
    await page.route('**/api/catalog/products*', (route: Route) => route.fulfill(ok([RICE])));
    await page.route('**/api/catalog/categories', (route: Route) => route.fulfill(ok([])));
    await page.goto('/checkout');
    await expect(page.getByText('Your cart is empty, so there is nothing to check out.')).toBeVisible();
  });

  test('place order is disabled until the required fields are filled', async ({ page }) => {
    await seedCart(page, 5);
    await page.goto('/checkout');
    await expect(page.getByRole('button', { name: 'Place order' })).toBeDisabled();

    await page.getByLabel('Full name').fill('Chidi Okeke');
    await page.getByLabel('Phone number').fill('08030000000');
    await page.getByLabel('Delivery address').fill('12 Faulks Road');
    await page.getByLabel('Town / city').fill('Aba');

    await expect(page.getByRole('button', { name: 'Place order' })).toBeEnabled();
  });

  test('a successful order shows the reference', async ({ page }) => {
    await seedCart(page, 5);
    await page.route('**/api/orders', (route: Route) =>
      route.fulfill(ok({
        id: 'o1', reference: 'IMC-000123', status: 'pending',
        total_kobo: 45_000_000, delivery_fee_kobo: 250_000,
        placed_at: '2026-08-07T10:00:00Z',
      })));

    await page.goto('/checkout');
    await page.getByLabel('Full name').fill('Chidi Okeke');
    await page.getByLabel('Phone number').fill('08030000000');
    await page.getByLabel('Delivery address').fill('12 Faulks Road');
    await page.getByLabel('Town / city').fill('Aba');
    await page.getByRole('button', { name: 'Place order' }).click();

    await expect(page.getByRole('heading', { name: 'Reference IMC-000123' })).toBeVisible();
  });

  test('a failed order surfaces the API message and keeps the cart', async ({ page }) => {
    await seedCart(page, 5);
    await page.route('**/api/orders', (route: Route) =>
      route.fulfill({
        status: 422,
        contentType: 'application/json',
        body: JSON.stringify({
          success: 'false', code: 'Validation',
          error: { message: 'We do not deliver to that state yet.' },
        }),
      }));

    await page.goto('/checkout');
    await page.getByLabel('Full name').fill('Chidi Okeke');
    await page.getByLabel('Phone number').fill('08030000000');
    await page.getByLabel('Delivery address').fill('12 Faulks Road');
    await page.getByLabel('Town / city').fill('Aba');
    await page.getByRole('button', { name: 'Place order' }).click();

    await expect(page.locator('.form-error')).toHaveText('We do not deliver to that state yet.');
    await expect(page.locator('.cart-badge')).toHaveText('5');
  });
});
