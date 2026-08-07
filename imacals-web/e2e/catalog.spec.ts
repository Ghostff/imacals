import { test, expect, type Page, type Route } from '@playwright/test';

const PRODUCTS = [
  {
    id: 'p1', slug: 'rice-50kg', name: 'Long Grain Rice — 50kg Bag',
    description: 'Parboiled long grain rice.',
    category_slug: 'foodstuff', category_name: 'Foodstuff',
    unit: 'bag (50kg)', unit_price_kobo: 8_950_000, min_order_quantity: 5,
    in_stock: true, image_url: null,
  },
  {
    id: 'p2', slug: 'detergent-carton', name: 'Detergent Powder — Carton of 24',
    description: 'Carton of 24 sachets.',
    category_slug: 'household', category_name: 'Household',
    unit: 'carton (24)', unit_price_kobo: 3_120_000, min_order_quantity: 1,
    in_stock: true, image_url: null,
  },
  {
    id: 'p3', slug: 'bar-soap-carton', name: 'Bar Soap — Carton of 48',
    description: 'Carton of 48 bar soaps.',
    category_slug: 'household', category_name: 'Household',
    unit: 'carton (48)', unit_price_kobo: 2_760_000, min_order_quantity: 1,
    in_stock: false, image_url: null,
  },
];

const CATEGORIES = [
  { slug: 'foodstuff', name: 'Foodstuff' },
  { slug: 'household', name: 'Household' },
];

function ok(body: unknown) {
  return {
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: body }),
  };
}

async function mockCatalog(page: Page): Promise<void> {
  await page.route('**/api/catalog/categories', (route: Route) => route.fulfill(ok(CATEGORIES)));
  await page.route('**/api/catalog/products*', (route: Route) => route.fulfill(ok(PRODUCTS)));
}

test.describe('Catalogue', () => {
  test.beforeEach(async ({ page }) => {
    await mockCatalog(page);
  });

  test('home page renders the hero and the Aba positioning', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'Wholesale supply, delivered from Aba.' })).toBeVisible();
    await expect(page.getByText('Aba · Abia State · Nigeria')).toBeVisible();
  });

  test('home page shows the phone order line', async ({ page }) => {
    await page.goto('/');
    // Phone ordering is a first-class channel — it must be on the landing page.
    await expect(page.locator('.order-line-number')).toBeVisible();
  });

  test('catalogue renders a card per product', async ({ page }) => {
    await page.goto('/catalog');
    await expect(page.locator('.card')).toHaveCount(PRODUCTS.length);
  });

  test('category filter narrows the grid', async ({ page }) => {
    await page.goto('/catalog');
    await page.getByRole('button', { name: 'Foodstuff' }).click();
    await expect(page.locator('.card')).toHaveCount(1);
    await expect(page.getByRole('heading', { name: 'Long Grain Rice — 50kg Bag' })).toBeVisible();
  });

  test('search narrows the grid', async ({ page }) => {
    await page.goto('/catalog');
    await page.getByLabel('Search products').fill('soap');
    await expect(page.locator('.card')).toHaveCount(1);
  });

  test('search with no match shows an empty state', async ({ page }) => {
    await page.goto('/catalog');
    await page.getByLabel('Search products').fill('zzzznothing');
    await expect(page.locator('.state-msg')).toBeVisible();
    await expect(page.locator('.card')).toHaveCount(0);
  });

  test('out-of-stock product is labelled', async ({ page }) => {
    await page.goto('/catalog');
    await expect(page.locator('.card-oos')).toHaveText('Out of stock');
  });

  test('shows an error state when the catalogue API fails', async ({ page }) => {
    await page.route('**/api/catalog/products*', (route: Route) =>
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({
          success: 'false', code: 'InternalServerError',
          error: { message: 'Unexpected error' },
        }),
      }));
    await page.goto('/catalog');
    await expect(page.locator('.state-msg--error')).toBeVisible();
  });

  test('empty catalogue points the customer at the phone line', async ({ page }) => {
    await page.route('**/api/catalog/products*', (route: Route) => route.fulfill(ok([])));
    await page.goto('/catalog');
    await expect(page.locator('.state-msg')).toContainText('Call');
  });
});

test.describe('Product detail', () => {
  test.beforeEach(async ({ page }) => {
    await mockCatalog(page);
    await page.route('**/api/catalog/products/rice-50kg', (route: Route) =>
      route.fulfill(ok(PRODUCTS[0])));
  });

  test('renders the product and its minimum order quantity', async ({ page }) => {
    await page.goto('/product/rice-50kg');
    await expect(page.getByRole('heading', { name: 'Long Grain Rice — 50kg Bag' })).toBeVisible();
    // Quantity opens at the MOQ so the warehouse never gets an unpickable line.
    await expect(page.getByLabel('Quantity')).toHaveValue('5');
  });

  test('adding to the cart updates the header badge', async ({ page }) => {
    await page.goto('/product/rice-50kg');
    await page.getByRole('button', { name: 'Add to cart', exact: true }).click();
    await expect(page.locator('.cart-badge')).toHaveText('5');
  });
});
