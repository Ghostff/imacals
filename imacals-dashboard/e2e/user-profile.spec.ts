import { test, expect, type Route } from '@playwright/test';

const MOCK_TOKEN = 'test-jwt-token';

const MOCK_ME = {
  id: 'user-1',
  first_name: 'Admin',
  last_name: 'User',
  email: 'admin@imacals.com',
  is_superuser: true,
  is_internal: true,
};

const MOCK_USER = {
  id: 'user-1',
  first_name: 'Alice',
  last_name: 'Smith',
  email: 'alice@imacals.com',
  phone: '08012345678',
  date_of_birth: '1995-05-15',
  is_superuser: true,
  is_internal: true,
  last_logged_in_at: '2026-04-01T10:00:00Z',
  current_logged_in_at: null,
  created_at: '2025-01-15T08:00:00Z',
  updated_at: '2026-04-01T10:00:00Z',
  organizations: [{ id: 'org-1', name: 'Imacals', slug: 'imacals' }],
  role: { id: 'r-1', name: 'admin', title: 'Admin' },
  user_role: { id: 'ur-1', name: 'backoffice', title: 'Back Office' },
};

function mockMe(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: { user: MOCK_ME } }),
  });
}

function mockUsers(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: [MOCK_USER] }),
  });
}

function mockDocuments(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: [] }),
  });
}

function mockBankAccounts(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: [] }),
  });
}

test.describe('Users — User Profile page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.evaluate((token) => localStorage.setItem('token', token), MOCK_TOKEN);
    await page.route('**/api/auth/me', mockMe);
    await page.route('**/api/users', mockUsers);
    await page.route('**/api/users/user-1/documents', mockDocuments);
    await page.route('**/api/users/user-1/bank-accounts', mockBankAccounts);
  });

  test('renders user name and email in header', async ({ page }) => {
    await page.goto('/users/user-1');
    await expect(page.getByRole('heading', { name: 'Alice Smith' })).toBeVisible();
    await expect(page.getByText('alice@imacals.com')).toBeVisible();
  });

  test('renders tabs and basic info form fields', async ({ page }) => {
    await page.goto('/users/user-1');
    await expect(page.getByRole('button', { name: 'Profile' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Documents' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Bank' })).toBeVisible();

    await expect(page.locator('input[type="email"]')).toHaveValue('alice@imacals.com');
    await expect(page.locator('input[type="tel"]')).toHaveValue('08012345678');
  });

  test('saving basic info submits update request', async ({ page }) => {
    let updateCalled = false;
    await page.route('**/api/users/user-1', async (route) => {
      if (route.request().method() === 'PUT') {
        updateCalled = true;
        const postData = route.request().postDataJSON();
        expect(postData.first_name).toBe('Alice');
        expect(postData.last_name).toBe('Smithson');
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: 'true', data: { message: 'User updated successfully' } }),
        });
      } else {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: 'true', data: { user: MOCK_USER, organizations: [] } }),
        });
      }
    });

    await page.goto('/users/user-1');
    const lastNameInput = page.locator('input[type="text"]').nth(1);
    await lastNameInput.fill('Smithson');
    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(page.locator('.form-success')).toContainText('Saved successfully.');
    expect(updateCalled).toBe(true);
  });

  test('back link navigates to /users/all', async ({ page }) => {
    await page.goto('/users/user-1');
    await page.getByRole('button', { name: '← Back to All Users' }).click();
    await expect(page).toHaveURL(/\/users\/all/);
  });
});
