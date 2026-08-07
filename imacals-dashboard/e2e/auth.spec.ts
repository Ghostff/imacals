import { test, expect, type Route } from '@playwright/test';

const MOCK_USER = {
  id: 'user-1',
  first_name: 'Test',
  last_name: 'User',
  email: 'test@imacals.com',
  is_superuser: true,
  is_internal: false,
};

const MOCK_TOKEN = 'test-jwt-token';

function mockLogin(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      success: 'true',
      data: { user: MOCK_USER, organizations: [], token: MOCK_TOKEN },
    }),
  });
}

function mockMe(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: { user: MOCK_USER } }),
  });
}

// The integrations page loads immediately after sign-in, so every redirect assertion needs its
// three calls stubbed or the landing page errors instead of rendering.
async function mockIntegrationsPage(page: import('@playwright/test').Page): Promise<void> {
  await page.route('**/api/integrations/provider-types', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        success: 'true',
        data: [
          {
            integration_type: 'smtp',
            integration_category: 'email',
            fields: [
              { name: 'SMTP_HOST', label: 'Host', type: 'text', is_encrypted: false, is_required: true },
            ],
          },
        ],
      }),
    }),
  );
  await page.route('**/api/integrations', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: 'true', data: [] }),
    }),
  );
  await page.route('**/api/domains', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: 'true', data: [] }),
    }),
  );
}

test.describe('Auth page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.evaluate(() => localStorage.clear());
  });

  test('renders the sign-in form', async ({ page }) => {
    await page.goto('/login');
    await expect(page.getByRole('heading', { name: 'Sign in' })).toBeVisible();
    await expect(page.getByLabel('Email')).toBeVisible();
    await expect(page.getByLabel('Password', { exact: true })).toBeVisible();
  });

  test('submit stays disabled until both fields are filled', async ({ page }) => {
    await page.goto('/login');
    const submit = page.getByRole('button', { name: 'Sign in' });
    await expect(submit).toBeDisabled();

    await page.getByLabel('Email').fill('test@imacals.com');
    await expect(submit).toBeDisabled();

    await page.getByLabel('Password', { exact: true }).fill('secret123');
    await expect(submit).toBeEnabled();
  });

  test('password can be revealed', async ({ page }) => {
    await page.goto('/login');
    const password = page.getByLabel('Password', { exact: true });
    await expect(password).toHaveAttribute('type', 'password');
    await page.getByRole('button', { name: 'Show password' }).click();
    await expect(password).toHaveAttribute('type', 'text');
  });

  test('successful sign-in lands on the integrations page', async ({ page }) => {
    await page.route('**/api/auth/login', mockLogin);
    await page.route('**/api/auth/me', mockMe);
    await mockIntegrationsPage(page);

    await page.goto('/login');
    await page.getByLabel('Email').fill('test@imacals.com');
    await page.getByLabel('Password', { exact: true }).fill('secret123');
    await page.getByRole('button', { name: 'Sign in' }).click();

    await expect(page).toHaveURL(/\/integrations$/);
    await expect(page.getByRole('heading', { name: 'Integrations' })).toBeVisible();
  });

  test('a deep link is restored after sign-in', async ({ page }) => {
    await page.route('**/api/auth/login', mockLogin);
    await page.route('**/api/auth/me', mockMe);
    await page.route('**/api/domains', (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: 'true', data: [] }),
      }),
    );

    // The guard bounces an unauthenticated deep link to /login with ?redirect=…
    await page.goto('/models/domain');
    await expect(page).toHaveURL(/\/login\?redirect=\/models\/domain/);

    await page.getByLabel('Email').fill('test@imacals.com');
    await page.getByLabel('Password', { exact: true }).fill('secret123');
    await page.getByRole('button', { name: 'Sign in' }).click();

    await expect(page).toHaveURL(/\/models\/domain$/);
  });

  test('shows the API error message when credentials are rejected', async ({ page }) => {
    await page.route('**/api/auth/login', (route) =>
      route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({
          success: 'false',
          code: 'InvalidEmailOrPassword',
          error: { message: 'Invalid email or password' },
        }),
      }),
    );

    await page.goto('/login');
    await page.getByLabel('Email').fill('test@imacals.com');
    await page.getByLabel('Password', { exact: true }).fill('wrong-password');
    await page.getByRole('button', { name: 'Sign in' }).click();

    await expect(page.getByRole('alert')).toHaveText('Invalid email or password');
    await expect(page).toHaveURL(/\/login/);
  });

  test('an authenticated visit to /login goes straight to integrations', async ({ page }) => {
    await page.route('**/api/auth/me', mockMe);
    await mockIntegrationsPage(page);

    await page.goto('/login');
    await page.evaluate((t) => localStorage.setItem('token', t), MOCK_TOKEN);
    await page.goto('/login');

    await expect(page).toHaveURL(/\/integrations$/);
  });
});
