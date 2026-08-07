import { test, expect, type Route, type Page } from '@playwright/test';

const MOCK_TOKEN = 'test-jwt-token';
const MOCK_USER = {
  id: 'u1',
  first_name: 'Test',
  last_name: 'User',
  email: 'test@imacals.com',
  is_superuser: true,
  is_internal: false,
};

const MOCK_DOMAINS = [{ id: 'd1', name: 'Default US', slug: 'default-us' }];

const MOCK_PROVIDER_TYPES = [
  {
    integration_type: 'smtp',
    integration_category: 'email',
    fields: [
      { name: 'SMTP_HOST', label: 'Host', type: 'text', is_encrypted: false, is_required: true },
      { name: 'SMTP_PORT', label: 'Port', type: 'text', is_encrypted: false, is_required: true },
      { name: 'SMTP_FROM_EMAIL', label: 'From Address', type: 'text', is_encrypted: false, is_required: true },
      { name: 'SMTP_PASSWORD', label: 'Password', type: 'password', is_encrypted: true, is_required: false },
    ],
  },
  {
    integration_type: 'log',
    integration_category: 'email',
    fields: [
      { name: 'LOG_FROM_EMAIL', label: 'From Address', type: 'text', is_encrypted: false, is_required: true },
    ],
  },
  {
    integration_type: 'mailgun',
    integration_category: 'email',
    fields: [
      { name: 'MAILGUN_API_KEY', label: 'API Key', type: 'password', is_encrypted: true, is_required: true },
      { name: 'MAILGUN_DOMAIN', label: 'Sending Domain', type: 'text', is_encrypted: false, is_required: true },
      { name: 'MAILGUN_FROM_EMAIL', label: 'From Address', type: 'text', is_encrypted: false, is_required: true },
    ],
  },
  {
    integration_type: 'zero-bounce',
    integration_category: 'email-validation',
    fields: [
      { name: 'ZEROBOUNCE_API_KEY', label: 'API Key', type: 'password', is_encrypted: true, is_required: true },
    ],
  },
  { integration_type: 'custom', integration_category: 'other', fields: [] },
];

const LOG_PROVIDER = {
  id: 'i1',
  organization_id: 'org1',
  domain_id: 'd1',
  created_by: 'u1',
  name: 'Log (writes to API log)',
  slug: 'log-mail',
  integration_type: 'log',
  integration_category: 'email',
  is_enabled: true,
  created_at: '2026-08-01T10:00:00Z',
  updated_at: '2026-08-01T10:00:00Z',
};

const MAILGUN_PROVIDER = {
  id: 'i2',
  organization_id: 'org1',
  domain_id: 'd1',
  created_by: 'u1',
  name: 'Mailgun',
  slug: 'mailgun',
  integration_type: 'mailgun',
  integration_category: 'email',
  is_enabled: false,
  created_at: '2026-08-01T11:00:00Z',
  updated_at: '2026-08-01T11:00:00Z',
};

const VERIFIER = {
  id: 'i3',
  organization_id: 'org1',
  domain_id: 'd1',
  created_by: 'u1',
  name: 'ZeroBounce',
  slug: 'zerobounce',
  integration_type: 'zero-bounce',
  integration_category: 'email-validation',
  is_enabled: true,
  created_at: '2026-08-01T12:00:00Z',
  updated_at: '2026-08-01T12:00:00Z',
};

// The API withholds encrypted values, so `value` is null for secrets — the page has to cope.
const MAILGUN_ATTRIBUTES = [
  {
    id: 'a1', created_by: 'u1', attributeable_type: 'integrations', attributeable_id: 'i2',
    name: 'MAILGUN_API_KEY', value: null, type: 'password', is_encrypted: true,
    created_at: '2026-08-01T11:00:00Z', updated_at: '2026-08-01T11:00:00Z',
  },
  {
    id: 'a2', created_by: 'u1', attributeable_type: 'integrations', attributeable_id: 'i2',
    name: 'MAILGUN_DOMAIN', value: 'mg.imacals.com', type: 'text', is_encrypted: false,
    created_at: '2026-08-01T11:00:00Z', updated_at: '2026-08-01T11:00:00Z',
  },
  {
    id: 'a3', created_by: 'u1', attributeable_type: 'integrations', attributeable_id: 'i2',
    name: 'MAILGUN_FROM_EMAIL', value: 'campaigns@imacals.com', type: 'text', is_encrypted: false,
    created_at: '2026-08-01T11:00:00Z', updated_at: '2026-08-01T11:00:00Z',
  },
];

function json(route: Route, data: unknown): void {
  route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data }),
  });
}

function fail(route: Route): void {
  route.fulfill({
    status: 500,
    contentType: 'application/json',
    body: JSON.stringify({
      success: 'false',
      code: 'InternalServerError',
      error: { message: 'Unexpected error' },
    }),
  });
}

function mockMe(route: Route): void {
  json(route, { user: MOCK_USER });
}

async function authenticate(page: Page): Promise<void> {
  await page.goto('/login');
  await page.evaluate((t) => localStorage.setItem('token', t), MOCK_TOKEN);
  await page.route('**/api/auth/me', mockMe);
}

async function mockBase(page: Page, integrations: unknown[]): Promise<void> {
  await page.route('**/api/integrations/provider-types', (route) => json(route, MOCK_PROVIDER_TYPES));
  await page.route('**/api/domains', (route) => json(route, MOCK_DOMAINS));
  // Matches /integrations and /integrations?category=… but not the sub-resources routed above.
  await page.route(/\/api\/integrations(\?[^/]*)?$/, (route) => {
    if (route.request().method() === 'GET') return json(route, integrations);
    return route.fallback();
  });
}

test.describe('Integrations page', () => {
  test.beforeEach(async ({ page }) => {
    await authenticate(page);
  });

  test('renders the page heading and intro', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER]);
    await page.goto('/integrations');
    await expect(page.getByRole('heading', { name: 'Integrations' })).toBeVisible();
    await expect(
      page.getByText('Credentials live here, not in environment files', { exact: false }),
    ).toBeVisible();
  });

  test('names the live sending provider', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER, MAILGUN_PROVIDER]);
    await page.goto('/integrations');
    await expect(page.locator('.live-value')).toContainText('Log (writes to API log)');
  });

  test('warns when nothing is live', async ({ page }) => {
    await mockBase(page, [MAILGUN_PROVIDER]);
    await page.goto('/integrations');
    await expect(page.locator('.live-value--none')).toContainText('campaigns cannot send');
  });

  test('groups providers into sending and verification sections', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER, MAILGUN_PROVIDER, VERIFIER]);
    await page.goto('/integrations');

    await expect(page.getByRole('heading', { name: 'Sending Providers' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Address Verification' })).toBeVisible();

    const sections = page.locator('.section');
    await expect(sections.nth(0).locator('tbody tr')).toHaveCount(2);
    await expect(sections.nth(1).locator('tbody tr')).toHaveCount(1);
  });

  test('marks each row Live or Off', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER, MAILGUN_PROVIDER]);
    await page.goto('/integrations');

    const rows = page.locator('.section').first().locator('tbody tr');
    await expect(rows.nth(0).locator('.badge--live')).toHaveText('Live');
    await expect(rows.nth(1).locator('.badge--off')).toHaveText('Off');
  });

  test('switching providers sends the enable call and re-reads the list', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER, MAILGUN_PROVIDER]);

    let enableBody: unknown = null;
    await page.route('**/api/integrations/i2/enabled', (route) => {
      enableBody = route.request().postDataJSON();
      json(route, { ...MAILGUN_PROVIDER, is_enabled: true });
    });

    await page.goto('/integrations');
    await expect(page.locator('.live-value')).toContainText('Log');

    // The page re-reads the list after switching, so serve the post-switch state.
    await page.unroute(/\/api\/integrations(\?[^/]*)?$/);
    await page.route(/\/api\/integrations(\?[^/]*)?$/, (route) =>
      json(route, [
        { ...LOG_PROVIDER, is_enabled: false },
        { ...MAILGUN_PROVIDER, is_enabled: true },
      ]),
    );

    await page.getByRole('button', { name: 'Make live' }).click();

    await expect(page.locator('.live-value')).toContainText('Mailgun');
    expect(enableBody).toEqual({ is_enabled: true });
  });

  test('credentials dialog shows stored plaintext but never a secret', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER, MAILGUN_PROVIDER]);
    await page.route('**/api/integrations/i2/attributes', (route) => json(route, MAILGUN_ATTRIBUTES));

    await page.goto('/integrations');
    await page
      .locator('tbody tr', { hasText: 'Mailgun' })
      .getByRole('button', { name: 'Credentials' })
      .click();

    await expect(page.getByRole('dialog')).toBeVisible();
    // Plaintext config is prefilled…
    await expect(page.locator('#edit-MAILGUN_DOMAIN')).toHaveValue('mg.imacals.com');
    // …the secret is not, and is flagged as already stored.
    await expect(page.locator('#edit-MAILGUN_API_KEY')).toHaveValue('');
    await expect(page.locator('#edit-MAILGUN_API_KEY')).toHaveAttribute('type', 'password');
    await expect(page.locator('.stored-tag').first()).toHaveText('stored');
  });

  test('only retyped credentials are saved', async ({ page }) => {
    await mockBase(page, [MAILGUN_PROVIDER]);
    await page.route('**/api/integrations/i2/attributes', (route) => json(route, MAILGUN_ATTRIBUTES));

    const updated: string[] = [];
    await page.route('**/api/attributes/*', (route) => {
      updated.push(route.request().url().split('/').pop() as string);
      json(route, MAILGUN_ATTRIBUTES[0]);
    });

    await page.goto('/integrations');
    await page
      .locator('tbody tr', { hasText: 'Mailgun' })
      .getByRole('button', { name: 'Credentials' })
      .click();
    await page.locator('#edit-MAILGUN_API_KEY').fill('new-key');
    await page.getByRole('button', { name: 'Save changes' }).click();

    await expect(page.getByText('The next send uses these values.')).toBeVisible();
    // Only the API key was touched — the untouched fields must not be rewritten.
    expect(updated).toEqual(['a1']);
  });

  test('the add form renders the chosen provider fields', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER]);
    await page.goto('/integrations');
    await page.getByRole('button', { name: '+ Add Provider' }).click();

    // SMTP is the default selection.
    await expect(page.locator('#cred-SMTP_HOST')).toBeVisible();
    await expect(page.locator('#cred-SMTP_PASSWORD')).toHaveAttribute('type', 'password');

    // Switching provider swaps the whole credential set.
    await page.locator('#int-type').selectOption('mailgun');
    await expect(page.locator('#cred-MAILGUN_API_KEY')).toBeVisible();
    await expect(page.locator('#cred-SMTP_HOST')).toHaveCount(0);
  });

  test('creating a provider posts the typed credentials', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER]);

    let posted: Record<string, unknown> | null = null;
    await page.route(/\/api\/integrations$/, (route) => {
      if (route.request().method() !== 'POST') return route.fallback();
      posted = route.request().postDataJSON();
      json(route, { ...MAILGUN_PROVIDER, id: 'new' });
    });

    await page.goto('/integrations');
    await page.getByRole('button', { name: '+ Add Provider' }).click();
    await page.locator('#int-name').fill('Campaign Relay');
    await page.locator('#cred-SMTP_HOST').fill('imacals-mail');
    await page.locator('#cred-SMTP_PORT').fill('1025');
    await page.locator('#cred-SMTP_FROM_EMAIL').fill('no-reply@imacals.local');
    await page.getByRole('button', { name: 'Add Provider', exact: true }).click();

    await expect.poll(() => posted).not.toBeNull();
    expect(posted).toMatchObject({
      name: 'Campaign Relay',
      slug: 'campaign-relay',
      integration_type: 'smtp',
      domain_id: 'd1',
    });
    // The blank optional password must not be stored as an empty attribute.
    const names = (posted as unknown as { attributes: { name: string }[] }).attributes.map(
      (a) => a.name,
    );
    expect(names).toEqual(['SMTP_HOST', 'SMTP_PORT', 'SMTP_FROM_EMAIL']);
  });

  test('deleting the live provider warns first', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER]);
    await page.goto('/integrations');
    await page.locator('tbody tr').first().getByRole('button', { name: 'Delete' }).click();
    await expect(
      page.getByText('deleting it leaves nothing sending', { exact: false }),
    ).toBeVisible();
  });

  test('shows the empty state when nothing is configured', async ({ page }) => {
    await mockBase(page, []);
    await page.goto('/integrations');
    await expect(page.getByText('No integrations configured yet.')).toBeVisible();
  });

  test('shows error state when the API fails', async ({ page }) => {
    await page.route('**/api/integrations/provider-types', fail);
    await page.route(/\/api\/integrations(\?[^/]*)?$/, fail);
    await page.route('**/api/domains', (route) => json(route, MOCK_DOMAINS));

    await page.goto('/integrations');
    await expect(page.locator('.state-msg--error')).toBeVisible();
  });

  test('the old /models/integrations path still lands here', async ({ page }) => {
    await mockBase(page, [LOG_PROVIDER]);
    await page.goto('/models/integrations');
    await expect(page).toHaveURL(/\/integrations$/);
    await expect(page.getByRole('heading', { name: 'Integrations' })).toBeVisible();
  });
});
