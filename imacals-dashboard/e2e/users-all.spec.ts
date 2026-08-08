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

const MOCK_ROLES = [
  { id: 'r-1', name: 'admin',      title: 'Admin'      },
  { id: 'r-2', name: 'order-desk', title: 'Order Desk' },
];

const MOCK_USERS = [
  {
    id: 'user-1',
    first_name: 'Alice',
    last_name: 'Smith',
    email: 'alice@imacals.com',
    phone: '+1 555-0100',
    is_superuser: true,
    is_internal: true,
    last_logged_in_at: '2026-04-01T10:00:00Z',
    current_logged_in_at: null,
    created_at: '2025-01-15T08:00:00Z',
    updated_at: '2026-04-01T10:00:00Z',
    role_id: 'r-1',
    role: { id: 'r-1', name: 'admin', title: 'Admin' },
  },
  {
    id: 'user-2',
    first_name: 'Bob',
    last_name: 'Jones',
    email: 'bob@example.com',
    phone: null,
    is_superuser: false,
    is_internal: false,
    last_logged_in_at: null,
    current_logged_in_at: null,
    created_at: '2025-03-10T12:00:00Z',
    updated_at: '2025-03-10T12:00:00Z',
    role_id: 'r-2',
    role: { id: 'r-2', name: 'order-desk', title: 'Order Desk' },
  },
  {
    id: 'user-3',
    first_name: 'Carol',
    last_name: 'White',
    email: 'carol@example.com',
    phone: null,
    is_superuser: false,
    is_internal: false,
    last_logged_in_at: null,
    current_logged_in_at: null,
    created_at: '2025-06-01T09:00:00Z',
    updated_at: '2025-06-01T09:00:00Z',
    role_id: null,
    role: null,
  },
];

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
    body: JSON.stringify({ success: 'true', data: MOCK_USERS }),
  });
}

function mockRoles(route: Route): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: MOCK_ROLES }),
  });
}

test.describe('Users — All Users page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate first: localStorage is unreachable on about:blank, so seeding the token
    // before any goto() throws a SecurityError.
    await page.goto('/login');
    await page.evaluate((token) => localStorage.setItem('token', token), MOCK_TOKEN);
    await page.route('**/api/auth/me', mockMe);
    await page.route('**/api/users', mockUsers);
    await page.route('**/api/roles', mockRoles);
  });

  // ── Renders ──────────────────────────────────────────────────────────────

  test('renders page heading', async ({ page }) => {
    await page.goto('/users/all');
    await expect(page.getByRole('heading', { name: 'All Users' })).toBeVisible();
  });

  test('renders a row for each user', async ({ page }) => {
    await page.goto('/users/all');
    await expect(page.locator('.users-table tbody tr')).toHaveCount(3);
  });

  test('displays user name and email', async ({ page }) => {
    await page.goto('/users/all');
    const firstRow = page.locator('.users-table tbody tr').first();
    await expect(firstRow).toContainText('Alice Smith');
    await expect(firstRow).toContainText('alice@imacals.com');
  });

  test('shows dash for null phone', async ({ page }) => {
    await page.goto('/users/all');
    const secondRow = page.locator('.users-table tbody tr').nth(1);
    await expect(secondRow).toContainText('—');
  });

  // ── Role column ───────────────────────────────────────────────────────────

  test('shows role title in Role column', async ({ page }) => {
    await page.goto('/users/all');
    const firstRow = page.locator('.users-table tbody tr').first();
    await expect(firstRow.locator('.badge')).toContainText('Admin');
  });

  test('shows Order Desk badge for second user', async ({ page }) => {
    await page.goto('/users/all');
    const secondRow = page.locator('.users-table tbody tr').nth(1);
    await expect(secondRow.locator('.badge')).toContainText('Order Desk');
  });

  test('shows dash when user has no role', async ({ page }) => {
    await page.goto('/users/all');
    const thirdRow = page.locator('.users-table tbody tr').nth(2);
    await expect(thirdRow.locator('.cell-muted').first()).toContainText('—');
  });

  // ── Role filter dropdown ──────────────────────────────────────────────────

  test('role filter dropdown is populated from /roles', async ({ page }) => {
    await page.goto('/users/all');
    const select = page.locator('.filter-select').first();
    await expect(select.locator('option', { hasText: 'Admin' })).toHaveCount(1);
    await expect(select.locator('option', { hasText: 'Order Desk' })).toHaveCount(1);
  });

  test('filtering by role hides non-matching users', async ({ page }) => {
    await page.goto('/users/all');
    await page.locator('.filter-select').first().selectOption('r-1'); // Admin
    await expect(page.locator('.users-table tbody tr')).toHaveCount(1);
    await expect(page.locator('.users-table tbody tr').first()).toContainText('Alice');
  });

  test('filtering by role shows only matching users', async ({ page }) => {
    await page.goto('/users/all');
    await page.locator('.filter-select').first().selectOption('r-2'); // Order Desk
    await expect(page.locator('.users-table tbody tr')).toHaveCount(1);
    await expect(page.locator('.users-table tbody tr').first()).toContainText('Bob');
  });

  // ── Search ───────────────────────────────────────────────────────────────

  test('search by name filters users', async ({ page }) => {
    await page.goto('/users/all');
    await page.locator('.search-input').fill('alice');
    await expect(page.locator('.users-table tbody tr')).toHaveCount(1);
    await expect(page.locator('.users-table tbody tr').first()).toContainText('Alice');
  });

  test('search by email filters users', async ({ page }) => {
    await page.goto('/users/all');
    await page.locator('.search-input').fill('bob@example');
    await expect(page.locator('.users-table tbody tr')).toHaveCount(1);
    await expect(page.locator('.users-table tbody tr').first()).toContainText('Bob');
  });

  // ── Clear filters ─────────────────────────────────────────────────────────

  test('clear button appears when a filter is active', async ({ page }) => {
    await page.goto('/users/all');
    await expect(page.locator('.clear-btn')).not.toBeVisible();
    await page.locator('.filter-select').first().selectOption('r-1');
    await expect(page.locator('.clear-btn')).toBeVisible();
  });

  test('clear button resets all filters', async ({ page }) => {
    await page.goto('/users/all');
    await page.locator('.search-input').fill('alice');
    await page.locator('.filter-select').first().selectOption('r-1');
    await page.locator('.clear-btn').click();
    await expect(page.locator('.users-table tbody tr')).toHaveCount(3);
    await expect(page.locator('.clear-btn')).not.toBeVisible();
  });

  // ── Add User modal ────────────────────────────────────────────────────────

  test('Add User button opens modal', async ({ page }) => {
    await page.goto('/users/all');
    await page.getByRole('button', { name: '+ Add User' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.locator('.modal-title')).toContainText('Add User');
  });

  test('modal role select is populated from /roles and defaults to the first', async ({ page }) => {
    await page.goto('/users/all');
    await page.getByRole('button', { name: '+ Add User' }).click();
    const roleSelect = page.locator('.modal select').last();
    await expect(roleSelect).toHaveValue('r-1');
    await expect(roleSelect.locator('option', { hasText: 'Admin' })).toHaveCount(1);
    await expect(roleSelect.locator('option', { hasText: 'Order Desk' })).toHaveCount(1);
  });

  test('Cancel button closes modal', async ({ page }) => {
    await page.goto('/users/all');
    await page.getByRole('button', { name: '+ Add User' }).click();
    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.getByRole('dialog')).not.toBeVisible();
  });

  test('submitting the form creates a user and closes modal', async ({ page }) => {
    await page.route('**/api/users', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            success: 'true',
            data: {
              user: {
                id: 'user-new', first_name: 'Jane', last_name: 'Doe',
                email: 'jane@example.com', phone: null,
                is_superuser: false, is_internal: false,
                last_logged_in_at: null, current_logged_in_at: null,
                created_at: '2026-05-11T00:00:00Z', updated_at: '2026-05-11T00:00:00Z',
              },
            },
          }),
        });
      } else {
        await mockUsers(route);
      }
    });

    await page.goto('/users/all');
    await page.getByRole('button', { name: '+ Add User' }).click();

    await page.locator('.modal input[type="text"]').first().fill('Jane');
    await page.locator('.modal input[type="text"]').nth(1).fill('Doe');
    await page.locator('.modal input[type="email"]').fill('jane@example.com');

    await page.getByRole('button', { name: 'Create User' }).click();
    await expect(page.getByRole('dialog')).not.toBeVisible();
    // New user appears at the top of the table
    await expect(page.locator('.users-table tbody tr').first()).toContainText('Jane Doe');
  });

  test('shows error message when create API fails', async ({ page }) => {
    await page.route('**/api/users', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 422,
          contentType: 'application/json',
          body: JSON.stringify({ success: 'false', code: 'EmailInUse', error: { message: 'Email already in use' } }),
        });
      } else {
        await mockUsers(route);
      }
    });

    await page.goto('/users/all');
    await page.getByRole('button', { name: '+ Add User' }).click();
    await page.locator('.modal input[type="text"]').first().fill('Jane');
    await page.locator('.modal input[type="text"]').nth(1).fill('Doe');
    await page.locator('.modal input[type="email"]').fill('existing@example.com');
    await page.getByRole('button', { name: 'Create User' }).click();
    await expect(page.locator('.modal-error')).toContainText('Email already in use');
    await expect(page.getByRole('dialog')).toBeVisible();
  });

  // ── Empty / error states ──────────────────────────────────────────────────

  test('shows no-match message when filters exclude all users', async ({ page }) => {
    await page.goto('/users/all');
    await page.locator('.search-input').fill('zzznomatch');
    await expect(page.locator('.empty-cell')).toContainText('No users match the current filters.');
  });

  test('shows empty state when API returns no users', async ({ page }) => {
    await page.route('**/api/users', (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: 'true', data: [] }),
      }),
    );
    await page.goto('/users/all');
    await expect(page.locator('.empty-cell')).toContainText('No users found.');
  });

  test('shows error message when users API fails', async ({ page }) => {
    await page.route('**/api/users', (route) =>
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ success: 'false', code: 'InternalServerError', error: { message: 'Unexpected error' } }),
      }),
    );
    await page.goto('/users/all');
    await expect(page.locator('.state-msg--error')).toBeVisible();
  });
});
