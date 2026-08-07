import { test, expect, type Route } from '@playwright/test';

const MOCK_TOKEN = 'test-jwt-token';

const MOCK_ME = {
  id: 'u-me', first_name: 'Admin', last_name: 'User',
  email: 'admin@imacals.com', is_superuser: true, is_internal: true,
};

const MOCK_USERS = [
  { id: 'u-1', first_name: 'Jane', last_name: 'Smith', email: 'jane@test.com', phone: null,
    date_of_birth: null, is_superuser: false, is_internal: false, last_logged_in_at: null,
    current_logged_in_at: null, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z',
    organizations: [], role: null, user_role: null },
  { id: 'u-2', first_name: 'Bob',  last_name: 'Jones',  email: 'bob@test.com',  phone: null,
    date_of_birth: null, is_superuser: false, is_internal: false, last_logged_in_at: null,
    current_logged_in_at: null, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z',
    organizations: [], role: null, user_role: null },
];

const MOCK_DOMAINS = [
  { id: 'd-1', name: 'Default US',  slug: 'default-us',  country_id: 'c-1', state_id: null,  city_id: null,  created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
  { id: 'd-2', name: 'Miami Metro', slug: 'miami-metro', country_id: 'c-1', state_id: 's-1', city_id: 'ci-1', created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
];

const MOCK_ELIGIBLE_ROLES = [
  { id: 'r-1', name: 'broker',    title: 'Broker',    description: '', organization_id: null, system_user_eligible: true, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
  { id: 'r-2', name: 'realtor',   title: 'Realtor',   description: '', organization_id: null, system_user_eligible: true, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
  { id: 'r-3', name: 'hml',       title: 'HML',       description: '', organization_id: null, system_user_eligible: true, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
  { id: 'r-4', name: 'insurance', title: 'Insurance', description: '', organization_id: null, system_user_eligible: true, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
];

const MOCK_ASSIGNMENTS = [
  { id: 'a-1', domain_id: 'd-1', domain_name: 'Default US', user_id: 'u-1',
    user_first_name: 'Jane', user_last_name: 'Smith', user_email: 'jane@test.com',
    user_role_id: 'r-1', role_name: 'broker', role_title: 'Broker',
    created_by: 'u-me', created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
  { id: 'a-2', domain_id: 'd-1', domain_name: 'Default US', user_id: 'u-2',
    user_first_name: 'Bob', user_last_name: 'Jones', user_email: 'bob@test.com',
    user_role_id: 'r-2', role_name: 'realtor', role_title: 'Realtor',
    created_by: 'u-me', created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
];

function mockMe(route: Route):            void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: { user: MOCK_ME } }) }); }
function mockDomains(route: Route):       void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: MOCK_DOMAINS }) }); }
function mockUsers(route: Route):         void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: MOCK_USERS }) }); }
function mockEligibleRoles(route: Route): void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: MOCK_ELIGIBLE_ROLES }) }); }
function mockAssignments(route: Route):   void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: MOCK_ASSIGNMENTS }) }); }

test.describe('System Users page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate first: localStorage is unreachable on about:blank, so seeding the token
    // before any goto() throws a SecurityError.
    await page.goto('/login');
    await page.evaluate((t) => localStorage.setItem('token', t), MOCK_TOKEN);
    await page.route('**/api/auth/me',                              mockMe);
    await page.route('**/api/domains',                              mockDomains);
    await page.route('**/api/users',                                mockUsers);
    await page.route('**/api/domain-system-users/eligible-roles',   mockEligibleRoles);
    await page.route('**/api/domain-system-users',                  mockAssignments);
  });

  // ── Renders ────────────────────────────────────────────────────────────────

  test('renders page heading', async ({ page }) => {
    await page.goto('/users/system');
    await expect(page.getByRole('heading', { name: 'System Users' })).toBeVisible();
  });

  test('renders one row per assignment', async ({ page }) => {
    await page.goto('/users/system');
    await expect(page.locator('.data-table tbody tr')).toHaveCount(2);
  });

  test('shows role title in badge', async ({ page }) => {
    await page.goto('/users/system');
    await expect(page.locator('.badge').first()).toHaveText('Broker');
  });

  test('shows user name in table row', async ({ page }) => {
    await page.goto('/users/system');
    await expect(page.getByText('Jane Smith')).toBeVisible();
  });

  // ── Empty state ────────────────────────────────────────────────────────────

  test('shows empty state when no assignments exist', async ({ page }) => {
    await page.route('**/api/domain-system-users', (route) => {
      if (route.request().method() === 'GET' && !route.request().url().includes('eligible')) {
        route.fulfill({ status: 200, contentType: 'application/json',
          body: JSON.stringify({ success: 'true', data: [] }) });
      } else {
        mockAssignments(route);
      }
    });
    await page.goto('/users/system');
    await expect(page.getByText('No system users configured.')).toBeVisible();
  });

  // ── Error state ────────────────────────────────────────────────────────────

  test('shows error state when API fails', async ({ page }) => {
    await page.route('**/api/domain-system-users', (route) => {
      if (!route.request().url().includes('eligible')) {
        route.fulfill({ status: 500, contentType: 'application/json',
          body: JSON.stringify({ success: 'false', code: 'InternalServerError',
            error: { message: 'Unexpected error' } }) });
      } else {
        mockEligibleRoles(route);
      }
    });
    await page.goto('/users/system');
    await expect(page.locator('.state-msg--error')).toBeVisible();
  });

  // ── Set modal ──────────────────────────────────────────────────────────────

  test('opens set modal when clicking Set System User', async ({ page }) => {
    await page.goto('/users/system');
    await page.getByRole('button', { name: '+ Set System User' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Set System User' })).toBeVisible();
  });

  test('role dropdown shows only eligible roles', async ({ page }) => {
    await page.goto('/users/system');
    await page.getByRole('button', { name: '+ Set System User' }).click();
    const roleSelect = page.locator('select').nth(1);
    await expect(roleSelect.locator('option[value="r-1"]')).toHaveText('Broker');
    await expect(roleSelect.locator('option[value="r-3"]')).toHaveText('HML');
  });

  test('submit button disabled until all fields filled', async ({ page }) => {
    await page.goto('/users/system');
    await page.getByRole('button', { name: '+ Set System User' }).click();
    await expect(page.getByRole('button', { name: 'Set User' })).toBeDisabled();
  });

  test('saves assignment and closes modal on success', async ({ page }) => {
    const saved = { ...MOCK_ASSIGNMENTS[0], id: 'a-new', user_id: 'u-2',
      user_first_name: 'Bob', user_last_name: 'Jones', user_email: 'bob@test.com' };

    await page.route('**/api/domain-system-users', (route) => {
      if (route.request().method() === 'POST') {
        route.fulfill({ status: 200, contentType: 'application/json',
          body: JSON.stringify({ success: 'true', data: saved }) });
      } else {
        mockAssignments(route);
      }
    });

    await page.goto('/users/system');
    await page.getByRole('button', { name: '+ Set System User' }).click();

    await page.locator('select').nth(0).selectOption('d-1');
    await page.locator('select').nth(1).selectOption('r-1');
    await page.locator('.user-option').first().click();

    await page.getByRole('button', { name: 'Set User' }).click();
    await expect(page.getByRole('dialog')).not.toBeVisible();
  });

  // ── Remove modal ───────────────────────────────────────────────────────────

  test('opens remove confirmation when hovering and clicking Remove', async ({ page }) => {
    await page.goto('/users/system');
    const row = page.locator('.data-table tbody tr').first();
    await row.hover();
    await row.getByRole('button', { name: 'Remove' }).click();
    await expect(page.getByRole('heading', { name: 'Remove System User' })).toBeVisible();
  });

  test('removes assignment on confirm', async ({ page }) => {
    await page.route('**/api/domain-system-users/a-1', (route) =>
      route.fulfill({ status: 200, contentType: 'application/json',
        body: JSON.stringify({ success: 'true', data: { message: 'System user removed successfully' } }) }));

    await page.goto('/users/system');
    const row = page.locator('.data-table tbody tr').first();
    await row.hover();
    await row.getByRole('button', { name: 'Remove' }).click();
    await page.getByRole('button', { name: 'Remove' }).last().click();
    await expect(page.getByRole('dialog')).not.toBeVisible();
    await expect(page.locator('.data-table tbody tr')).toHaveCount(1);
  });
});
