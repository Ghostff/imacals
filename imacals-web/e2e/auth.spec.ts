import { test, expect, type Page, type Route } from '@playwright/test';

const MOCK_USER = {
  id: 'user-cust-1',
  first_name: 'Chukwudi',
  last_name: 'Okonkwo',
  email: 'chukwudi@example.com',
  phone: '08031234567',
  is_superuser: false,
  is_internal: false,
  created_at: '2026-05-15T09:00:00Z',
  updated_at: '2026-05-15T09:00:00Z',
};

const MOCK_TOKEN = 'customer-jwt-token';

function ok(body: unknown) {
  return {
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: body }),
  };
}

async function mockCatalog(page: Page): Promise<void> {
  await page.route('**/api/catalog/categories', (route: Route) => route.fulfill(ok([])));
  await page.route('**/api/catalog/products*', (route: Route) => route.fulfill(ok([])));
}

test.describe('Storefront Authentication & Registration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await mockCatalog(page);
  });

  // ── Registration Page ─────────────────────────────────────────────────────

  test('renders registration form', async ({ page }) => {
    await page.goto('/register');
    await expect(page.getByRole('heading', { name: 'Create your account' })).toBeVisible();
    await expect(page.getByLabel('First name')).toBeVisible();
    await expect(page.getByLabel('Last name')).toBeVisible();
    await expect(page.getByLabel('Email address')).toBeVisible();
    await expect(page.getByLabel('Phone number')).toBeVisible();
    await expect(page.getByLabel('Password', { exact: true })).toBeVisible();
    await expect(page.getByLabel('Confirm password')).toBeVisible();
  });

  test('register button stays disabled until required fields are filled and valid', async ({ page }) => {
    await page.goto('/register');
    const submitBtn = page.getByRole('button', { name: 'Create account' });
    await expect(submitBtn).toBeDisabled();

    await page.getByLabel('First name').fill('Chukwudi');
    await page.getByLabel('Last name').fill('Okonkwo');
    await page.getByLabel('Email address').fill('chukwudi@example.com');
    await page.getByLabel('Password', { exact: true }).fill('pass123');
    await page.getByLabel('Confirm password').fill('mismatch');
    await expect(submitBtn).toBeDisabled();
    await expect(page.getByText('Passwords do not match')).toBeVisible();

    await page.getByLabel('Confirm password').fill('pass123');
    await expect(submitBtn).toBeEnabled();
  });

  test('successful registration signs the user in and redirects', async ({ page }) => {
    await page.route('**/api/auth/register', (route) =>
      route.fulfill(ok({ user: MOCK_USER, token: MOCK_TOKEN })),
    );
    await page.route('**/api/auth/me', (route) =>
      route.fulfill(ok({ user: MOCK_USER })),
    );

    await page.goto('/register');
    await page.getByLabel('First name').fill('Chukwudi');
    await page.getByLabel('Last name').fill('Okonkwo');
    await page.getByLabel('Email address').fill('chukwudi@example.com');
    await page.getByLabel('Phone number').fill('08031234567');
    await page.getByLabel('Password', { exact: true }).fill('secret123');
    await page.getByLabel('Confirm password').fill('secret123');

    await page.getByRole('button', { name: 'Create account' }).click();

    await expect(page).toHaveURL(/\/account/);
    await expect(page.getByRole('heading', { name: 'Hello, Chukwudi' })).toBeVisible();
  });

  // ── Login Page ────────────────────────────────────────────────────────────

  test('renders sign in form', async ({ page }) => {
    await page.goto('/login');
    await expect(page.getByRole('heading', { name: 'Welcome back' })).toBeVisible();
    await expect(page.getByLabel('Email address')).toBeVisible();
    await expect(page.getByLabel('Password', { exact: true })).toBeVisible();
  });

  test('successful login redirects to account', async ({ page }) => {
    await page.route('**/api/auth/login', (route) =>
      route.fulfill(ok({ user: MOCK_USER, token: MOCK_TOKEN })),
    );
    await page.route('**/api/auth/me', (route) =>
      route.fulfill(ok({ user: MOCK_USER })),
    );

    await page.goto('/login');
    await page.getByLabel('Email address').fill('chukwudi@example.com');
    await page.getByLabel('Password', { exact: true }).fill('secret123');
    await page.getByRole('button', { name: 'Sign in' }).click();

    await expect(page).toHaveURL(/\/account/);
  });

  // ── Header Account Menu ───────────────────────────────────────────────────

  test('header displays user initials and opens dropdown when logged in', async ({ page }) => {
    await page.route('**/api/auth/me', (route) =>
      route.fulfill(ok({ user: MOCK_USER })),
    );

    await page.goto('/');
    await page.evaluate((t) => localStorage.setItem('token', t), MOCK_TOKEN);
    await page.goto('/');

    const accountBtn = page.getByRole('button', { name: 'Account menu' });
    await expect(accountBtn).toBeVisible();
    await expect(accountBtn).toContainText('Chukwudi');

    await accountBtn.click();
    await expect(page.getByRole('link', { name: 'My Account' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();
  });

  // ── Checkout Autofill ─────────────────────────────────────────────────────

  test('checkout pre-fills customer details when logged in', async ({ page }) => {
    await page.route('**/api/auth/me', (route) =>
      route.fulfill(ok({ user: MOCK_USER })),
    );
    await page.route('**/api/catalog/products*', (route) =>
      route.fulfill(ok([
        {
          id: 'p1', slug: 'rice', name: 'Rice',
          description: '', category_slug: 'food', category_name: 'Food',
          unit: 'bag', unit_price_kobo: 500000, min_order_quantity: 1,
          in_stock: true, image_url: null,
        },
      ])),
    );

    await page.goto('/checkout');
    await page.evaluate((t) => {
      localStorage.setItem('token', t);
      localStorage.setItem('cart', JSON.stringify({
        lines: [{
          product: { id: 'p1', slug: 'rice', name: 'Rice', unit: 'bag', unit_price_kobo: 500000, min_order_quantity: 1, in_stock: true, image_url: null },
          quantity: 1,
        }],
      }));
    }, MOCK_TOKEN);

    await page.goto('/checkout');

    await expect(page.locator('#name')).toHaveValue('Chukwudi Okonkwo');
    await expect(page.locator('#email')).toHaveValue('chukwudi@example.com');
    await expect(page.locator('#phone')).toHaveValue('08031234567');
  });
});
