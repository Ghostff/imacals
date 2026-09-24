import { test, expect, type Route, type Page } from '@playwright/test';

const MOCK_TOKEN = 'test-jwt-token';
const MOCK_USER = {
  id: 'u1',
  first_name: 'Admin',
  last_name: 'Staff',
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

const MOCK_USERS = [
  {
    id: 'user-1',
    first_name: 'Alice',
    last_name: 'Smith',
    email: 'alice@imacals.com',
    phone: '+234 801 234 5678',
    is_superuser: true,
    is_internal: true,
    last_logged_in_at: '2026-04-01T10:00:00Z',
    current_logged_in_at: null,
    created_at: '2025-01-15T08:00:00Z',
    updated_at: '2026-04-01T10:00:00Z',
    organizations: [{ id: 'org-1', name: 'Imacals', slug: 'imacals' }],
    role: { id: 'r-1', name: 'admin', title: 'Admin' },
    user_role: { id: 'ur-1', name: 'order-desk', title: 'Order Desk' },
  },
];

const MOCK_INTEGRATIONS = [
  {
    id: 'int-1',
    organization_id: 'org-1',
    domain_id: 'd1',
    name: 'Paystack Production',
    provider_type: 'paystack',
    category: 'other',
    is_enabled: true,
    credentials: {},
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  },
];

function mockMe(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: { user: MOCK_USER } }),
  });
}

async function setupPage(page: Page): Promise<void> {
  await page.route('**/api/auth/me', mockMe);
  await page.route('**/api/products', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: 'true', data: MOCK_PRODUCTS }),
    }),
  );
  await page.route('**/api/categories', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: 'true', data: MOCK_CATEGORIES }),
    }),
  );
  await page.route('**/api/users', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: 'true', data: MOCK_USERS }),
    }),
  );
  await page.route('**/api/integrations', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: 'true', data: MOCK_INTEGRATIONS }),
    }),
  );

  await page.goto('/login');
  await page.evaluate((tok) => localStorage.setItem('token', tok), MOCK_TOKEN);
}

test.describe('Admin Dashboard Home Overview', () => {
  test.beforeEach(async ({ page }) => {
    await setupPage(page);
  });

  test('renders dashboard overview title, hub banner, and metric stat cards', async ({ page }) => {
    await page.goto('/');

    // Title and hub banner
    await expect(page.getByRole('heading', { name: 'Admin Dashboard' })).toBeVisible();
    await expect(page.locator('.hub-location')).toContainText('Aba, Abia State');
    await expect(page.locator('.hub-title')).toContainText('Central Warehouse & Logistics Base');

    // Stat cards values
    await expect(page.locator('.stat-val').first()).toContainText('2'); // 2 products
    await expect(page.locator('.stat-val').nth(1)).toContainText('2'); // 2 categories
    await expect(page.locator('.stat-val').nth(2)).toContainText('1'); // 1 user
    await expect(page.locator('.stat-val').nth(3)).toContainText('1'); // 1 integration
  });

  test('renders quick action buttons and operational channels', async ({ page }) => {
    await page.goto('/');

    // Quick action cards
    await expect(page.getByText('Create Product')).toBeVisible();
    await expect(page.getByText('Add Team / User')).toBeVisible();
    await expect(page.getByText('Manage Catalogue')).toBeVisible();
    await expect(page.getByText('Integrations & API')).toBeVisible();

    // Channel cards
    await expect(page.getByText('Base Warehouse (Aba)')).toBeVisible();
    await expect(page.getByText('Phone Order Desk')).toBeVisible();
    await expect(page.getByText('Online Storefront')).toBeVisible();
  });

  test('renders recent products and recent users summary tables', async ({ page }) => {
    await page.goto('/');

    await expect(page.getByText('Long Grain Rice — 50kg Bag')).toBeVisible();
    await expect(page.getByText('Alice Smith')).toBeVisible();
    await expect(page.locator('.overview-table').getByText('Order Desk')).toBeVisible();
  });
});
