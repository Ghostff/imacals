import { test, expect, type Route, type Page } from '@playwright/test';

const MOCK_TOKEN = 'test-jwt-token';
const MOCK_USER = {
  id: 'u1',
  first_name: 'Test',
  last_name: 'User',
  email: 'admin@imacals.com',
  is_superuser: true,
  is_internal: true,
};

const MOCK_CATEGORIES = [
  { id: 'cat-1', domain_id: 'd1', name: 'Foodstuff', slug: 'foodstuff', description: 'Grains & Staples' },
  { id: 'cat-2', domain_id: 'd1', name: 'Household', slug: 'household', description: 'Detergents & Soaps' },
];

const MOCK_PRODUCTS = [
  {
    id: 'prod-1',
    organization_id: 'org-1',
    domain_id: 'd1',
    category_id: 'cat-1',
    category_name: 'Foodstuff',
    category_slug: 'foodstuff',
    created_by: 'u1',
    name: 'Long Grain Rice — 50kg Bag',
    slug: 'rice-50kg',
    description: 'Parboiled long grain rice in a 50kg bag.',
    unit: 'bag (50kg)',
    unit_price_kobo: 8950000,
    min_order_quantity: 5,
    in_stock: true,
    image_url: null,
    created_at: '2026-08-21T08:00:00Z',
    updated_at: '2026-08-21T08:00:00Z',
  },
  {
    id: 'prod-2',
    organization_id: 'org-1',
    domain_id: 'd1',
    category_id: 'cat-2',
    category_name: 'Household',
    category_slug: 'household',
    created_by: 'u1',
    name: 'Detergent Powder — Carton of 24',
    slug: 'detergent-carton',
    description: 'Carton of 24 × 900g detergent sachets.',
    unit: 'carton (24)',
    unit_price_kobo: 3120000,
    min_order_quantity: 1,
    in_stock: false,
    image_url: null,
    created_at: '2026-08-21T08:00:00Z',
    updated_at: '2026-08-21T08:00:00Z',
  },
];

function mockMe(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: { user: MOCK_USER } }),
  });
}

async function setupAuth(page: Page): Promise<void> {
  await page.goto('/login');
  await page.evaluate((t) => localStorage.setItem('token', t), MOCK_TOKEN);
  await page.route('**/api/auth/me', mockMe);
}

test.describe('Products management page', () => {
  test.beforeEach(async ({ page }) => {
    await setupAuth(page);
    await page.route('**/api/categories', (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: 'true', data: MOCK_CATEGORIES }),
      }),
    );
    await page.route('**/api/products', (route) => {
      if (route.request().method() === 'GET') {
        return route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: 'true', data: MOCK_PRODUCTS }),
        });
      }
      return route.continue();
    });
  });

  test('renders page heading and product rows', async ({ page }) => {
    await page.goto('/products');
    await expect(page.getByRole('heading', { name: 'Products' })).toBeVisible();
    await expect(page.getByText('Long Grain Rice — 50kg Bag')).toBeVisible();
    await expect(page.getByText('Detergent Powder — Carton of 24')).toBeVisible();
    await expect(page.locator('.data-table tbody tr')).toHaveCount(2);
  });

  test('filters products by search input', async ({ page }) => {
    await page.goto('/products');
    const searchInput = page.getByRole('searchbox', { name: 'Search products' });
    await searchInput.fill('Rice');
    await expect(page.getByText('Long Grain Rice — 50kg Bag')).toBeVisible();
    await expect(page.getByText('Detergent Powder — Carton of 24')).not.toBeVisible();
  });

  test('shows empty state when no products match', async ({ page }) => {
    await page.route('**/api/products', (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: 'true', data: [] }),
      }),
    );
    await page.goto('/products');
    await expect(page.getByText('No products found')).toBeVisible();
  });

  test('shows error state when API fails', async ({ page }) => {
    await page.route('**/api/products', (route) =>
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({
          success: 'false',
          code: 'InternalServerError',
          error: { message: 'Database connection failed' },
        }),
      }),
    );
    await page.goto('/products');
    await expect(page.locator('.state-msg--error')).toBeVisible();
  });

  test('opens add product modal on click', async ({ page }) => {
    await page.goto('/products');
    await page.getByRole('button', { name: '+ Add Product', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'Add New Product' })).toBeVisible();
    await expect(page.getByLabel('Product Name *')).toBeVisible();
  });
});
