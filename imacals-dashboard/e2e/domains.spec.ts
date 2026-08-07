import { test, expect, type Route } from '@playwright/test';

const MOCK_TOKEN = 'test-jwt-token';

const MOCK_ME = {
  id: 'u-1', first_name: 'Admin', last_name: 'User',
  email: 'admin@imacals.com', is_superuser: true, is_internal: true,
};

const MOCK_DOMAINS = [
  { id: 'd-1', name: 'Default US',  slug: 'default-us',  country_id: 'c-1', state_id: null,  city_id: null,  created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
  { id: 'd-2', name: 'Miami Metro', slug: 'miami-metro', country_id: 'c-1', state_id: 's-1', city_id: null,  created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' },
];

const MOCK_COUNTRIES = [
  { id: 'c-1', name: 'United States', iso2_code: 'US', iso3_code: 'USA' },
  { id: 'c-2', name: 'Canada',        iso2_code: 'CA', iso3_code: 'CAN' },
];

const MOCK_STATES = [
  { id: 's-1', country_id: 'c-1', name: 'Florida', code: 'FL', latitude: 27.66, longitude: -81.51 },
  { id: 's-2', country_id: 'c-1', name: 'Texas',   code: 'TX', latitude: 31.00, longitude: -100.00 },
];

const MOCK_CITIES = [
  { id: 'ci-1', state_id: 's-1', name: 'Miami',   latitude: 25.76, longitude: -80.19 },
  { id: 'ci-2', state_id: 's-1', name: 'Orlando', latitude: 28.53, longitude: -81.37 },
];

function mockMe(route: Route):        void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: { user: MOCK_ME } }) }); }
function mockDomains(route: Route):   void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: MOCK_DOMAINS }) }); }
function mockCountries(route: Route): void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: MOCK_COUNTRIES }) }); }
function mockStates(route: Route):    void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: MOCK_STATES }) }); }
function mockCities(route: Route):    void { route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: MOCK_CITIES }) }); }

test.describe('Domains page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate first: localStorage is unreachable on about:blank, so seeding the token
    // before any goto() throws a SecurityError.
    await page.goto('/login');
    await page.evaluate((t) => localStorage.setItem('token', t), MOCK_TOKEN);
    await page.route('**/api/auth/me',          mockMe);
    await page.route('**/api/domains',          mockDomains);
    await page.route('**/api/geo/countries',    mockCountries);
    await page.route('**/api/geo/countries/**', mockStates);
    await page.route('**/api/geo/states/**',    mockCities);
  });

  // ── Renders ─────────────────────────────────────────────────────────────

  test('renders page heading', async ({ page }) => {
    await page.goto('/models/domain');
    await expect(page.getByRole('heading', { name: 'Domains' })).toBeVisible();
  });

  test('renders a row for each domain', async ({ page }) => {
    await page.goto('/models/domain');
    await expect(page.locator('.data-table tbody tr')).toHaveCount(2);
  });

  // ── Country name resolution ──────────────────────────────────────────────

  test('shows country name instead of UUID', async ({ page }) => {
    await page.goto('/models/domain');
    const firstRow = page.locator('.data-table tbody tr').first();
    await expect(firstRow).toContainText('United States');
    await expect(firstRow).not.toContainText('c-1');
  });

  // ── Search ───────────────────────────────────────────────────────────────

  test('search by name filters domains', async ({ page }) => {
    await page.goto('/models/domain');
    await page.locator('.search-input').fill('miami');
    await expect(page.locator('.data-table tbody tr')).toHaveCount(1);
    await expect(page.locator('.data-table tbody tr').first()).toContainText('Miami Metro');
  });

  test('search by slug filters domains', async ({ page }) => {
    await page.goto('/models/domain');
    await page.locator('.search-input').fill('default-us');
    await expect(page.locator('.data-table tbody tr')).toHaveCount(1);
  });

  test('shows empty row when search matches nothing', async ({ page }) => {
    await page.goto('/models/domain');
    await page.locator('.search-input').fill('zzznomatch');
    await expect(page.locator('.empty-cell')).toContainText('No domains found.');
  });

  // ── Empty / error states ─────────────────────────────────────────────────

  test('shows empty state when API returns no domains', async ({ page }) => {
    await page.route('**/api/domains', (route) =>
      route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: [] }) }),
    );
    await page.goto('/models/domain');
    await expect(page.locator('.empty-cell')).toContainText('No domains found.');
  });

  test('shows error state when domains API fails', async ({ page }) => {
    await page.route('**/api/domains', (route) =>
      route.fulfill({ status: 500, contentType: 'application/json', body: JSON.stringify({ success: 'false', code: 'InternalServerError', error: { message: 'Unexpected error' } }) }),
    );
    await page.goto('/models/domain');
    await expect(page.locator('.state-msg--error')).toBeVisible();
  });

  // ── Add modal ────────────────────────────────────────────────────────────

  test('Add Domain button opens modal', async ({ page }) => {
    await page.goto('/models/domain');
    await page.getByRole('button', { name: '+ Add Domain' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.locator('.modal-title')).toContainText('Add Domain');
  });

  test('Cancel button closes Add modal', async ({ page }) => {
    await page.goto('/models/domain');
    await page.getByRole('button', { name: '+ Add Domain' }).click();
    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.getByRole('dialog')).not.toBeVisible();
  });

  test('slug is auto-generated from name input', async ({ page }) => {
    await page.goto('/models/domain');
    await page.getByRole('button', { name: '+ Add Domain' }).click();
    await page.locator('.modal input[type="text"]').first().fill('Houston Area');
    const slugInput = page.locator('.modal input[type="text"]').nth(1);
    await expect(slugInput).toHaveValue('houston-area');
  });

  test('submitting the Add form calls POST and appends the domain', async ({ page }) => {
    const newDomain = { id: 'd-new', name: 'Houston Area', slug: 'houston-area', country_id: 'c-1', state_id: null, city_id: null, created_at: '2026-05-11T00:00:00Z', updated_at: '2026-05-11T00:00:00Z' };
    await page.route('**/api/domains', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: newDomain }) });
      } else {
        await mockDomains(route);
      }
    });

    await page.goto('/models/domain');
    await page.getByRole('button', { name: '+ Add Domain' }).click();
    await page.locator('.modal input[type="text"]').first().fill('Houston Area');
    await page.locator('.modal select').first().selectOption('c-1');
    await page.getByRole('button', { name: 'Add Domain', exact: true }).click();
    await expect(page.getByRole('dialog')).not.toBeVisible();
    await expect(page.locator('.data-table tbody')).toContainText('Houston Area');
  });

  test('shows modal error when create API fails', async ({ page }) => {
    await page.route('**/api/domains', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({ status: 422, contentType: 'application/json', body: JSON.stringify({ success: 'false', code: 'Validation', error: { message: 'slug: Slug already taken' } }) });
      } else {
        await mockDomains(route);
      }
    });

    await page.goto('/models/domain');
    await page.getByRole('button', { name: '+ Add Domain' }).click();
    await page.locator('.modal input[type="text"]').first().fill('Default US');
    await page.locator('.modal select').first().selectOption('c-1');
    await page.getByRole('button', { name: 'Add Domain', exact: true }).click();
    await expect(page.locator('.modal-error')).toContainText('Slug already taken');
    await expect(page.getByRole('dialog')).toBeVisible();
  });

  // ── Edit modal ───────────────────────────────────────────────────────────

  test('Edit opens modal pre-filled with domain data', async ({ page }) => {
    await page.goto('/models/domain');
    await page.locator('.data-table tbody tr').first().hover();
    await page.locator('.data-table tbody tr').first().getByRole('button', { name: 'Edit' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.locator('.modal-title')).toContainText('Edit Domain');
    await expect(page.locator('.modal input[type="text"]').first()).toHaveValue('Default US');
  });

  test('editing a domain calls PUT and updates the table', async ({ page }) => {
    const updated = { ...MOCK_DOMAINS[0], name: 'Default USA' };
    await page.route('**/api/domains/d-1', (route) =>
      route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: updated }) }),
    );

    await page.goto('/models/domain');
    await page.locator('.data-table tbody tr').first().hover();
    await page.locator('.data-table tbody tr').first().getByRole('button', { name: 'Edit' }).click();
    await page.locator('.modal input[type="text"]').first().fill('Default USA');
    await page.getByRole('button', { name: 'Save Changes' }).click();
    await expect(page.getByRole('dialog')).not.toBeVisible();
    await expect(page.locator('.data-table tbody')).toContainText('Default USA');
  });

  // ── Delete confirmation ──────────────────────────────────────────────────

  test('Delete opens confirmation dialog', async ({ page }) => {
    await page.goto('/models/domain');
    await page.locator('.data-table tbody tr').first().hover();
    await page.locator('.data-table tbody tr').first().getByRole('button', { name: 'Delete' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.locator('.modal-title')).toContainText('Delete Domain');
    await expect(page.locator('.confirm-msg')).toContainText('Default US');
  });

  test('confirming delete removes domain from table', async ({ page }) => {
    await page.route('**/api/domains/d-1', (route) =>
      route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: 'true', data: { message: 'Domain deleted successfully' } }) }),
    );

    await page.goto('/models/domain');
    await page.locator('.data-table tbody tr').first().hover();
    await page.locator('.data-table tbody tr').first().getByRole('button', { name: 'Delete' }).click();
    await page.getByRole('button', { name: 'Delete' }).last().click();
    await expect(page.locator('.data-table tbody tr')).toHaveCount(1);
    await expect(page.locator('.data-table tbody')).not.toContainText('Default US');
  });
});
