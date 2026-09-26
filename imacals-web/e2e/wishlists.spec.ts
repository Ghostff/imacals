import { test, expect, type Page, type Route } from '@playwright/test';

const MOCK_TOKEN = 'wishlist-test-token';
const MOCK_USER  = {
  id: 'user-cust-1',
  first_name: 'Chukwudi',
  last_name: 'Okonkwo',
  email: 'chukwudi@example.com',
  phone: '08031234567',
  is_superuser: false,
  is_internal: false,
};

const WISHLIST_LAGOS = {
  id: 'wl-1',
  organization_id: 'org-1',
  customer_id: 'cust-1',
  name: 'Stocking the Lagos branch',
  description: 'For the Q3 shop fit-out',
  created_at: '2026-09-20T09:00:00Z',
  updated_at: '2026-09-20T09:00:00Z',
  item_count: 2,
};

const WISHLIST_SAVED = {
  id: 'wl-2',
  organization_id: 'org-1',
  customer_id: 'cust-1',
  name: 'Saved items',
  description: null,
  created_at: '2026-09-22T09:00:00Z',
  updated_at: '2026-09-22T09:00:00Z',
  item_count: 0,
};

const RICE = {
  id: 'p-rice', slug: 'rice-50kg', name: 'Long Grain Rice — 50kg Bag',
  description: 'Parboiled long grain rice.',
  category_slug: 'foodstuff', category_name: 'Foodstuff',
  unit: 'bag (50kg)', unit_price_kobo: 8_950_000, min_order_quantity: 5,
  in_stock: true, image_url: null,
};

const DETERGENT = {
  id: 'p-det', slug: 'detergent-carton', name: 'Detergent Powder — Carton of 24',
  description: 'Carton of 24 sachets.',
  category_slug: 'household', category_name: 'Household',
  unit: 'carton (24)', unit_price_kobo: 3_120_000, min_order_quantity: 1,
  in_stock: true, image_url: null,
};

function ok(body: unknown) {
  return {
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: body }),
  };
}

async function seedAuth(page: Page): Promise<void> {
  // Navigate first so localStorage is reachable (it throws on about:blank).
  await page.goto('/login');
  await page.evaluate((t) => localStorage.setItem('token', t), MOCK_TOKEN);
  await page.route('**/api/auth/me', (route: Route) =>
    route.fulfill(ok({ user: MOCK_USER })),
  );
}

test.describe('Wishlists page', () => {
  test.beforeEach(async ({ page }) => {
    await seedAuth(page);
    await page.route('**/api/catalog/categories', (route: Route) =>
      route.fulfill(ok([])),
    );
    await page.route('**/api/catalog/products*', (route: Route) =>
      route.fulfill(ok([])),
    );
  });

  test('renders a card for each wishlist returned by the API', async ({ page }) => {
    await page.route('**/api/wishlists', (route: Route) =>
      route.fulfill(ok([WISHLIST_LAGOS, WISHLIST_SAVED])),
    );

    await page.goto('/wishlists');
    await expect(page.getByRole('heading', { name: 'Wishlists' })).toBeVisible();
    await expect(page.locator('.list-card')).toHaveCount(2);
    await expect(page.getByText('Stocking the Lagos branch')).toBeVisible();
    await expect(page.getByText('Saved items')).toBeVisible();
    await expect(page.getByText('2 items')).toBeVisible();
  });

  test('shows the empty state when the customer has no wishlists', async ({ page }) => {
    await page.route('**/api/wishlists', (route: Route) => route.fulfill(ok([])));

    await page.goto('/wishlists');
    await expect(page.locator('.state-msg')).toContainText('no wishlists yet');
  });

  test('shows an error state when the wishlists API fails', async ({ page }) => {
    await page.route('**/api/wishlists', (route: Route) =>
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({
          success: 'false',
          code: 'InternalServerError',
          error: { message: 'Unexpected error' },
        }),
      }),
    );

    await page.goto('/wishlists');
    await expect(page.locator('.state-msg--error')).toBeVisible();
  });

  test('creating a new list posts to the API and refreshes the grid', async ({ page }) => {
    let created = false;
    await page.route(/\/api\/wishlists$/, async (route: Route) => {
      if (route.request().method() === 'POST') {
        created = true;
        await route.fulfill(ok({
          ...WISHLIST_SAVED,
          name: 'Cement restock',
          description: 'For the new site',
        }));
        return;
      }
      // GET — return the freshly created list so refresh() renders it.
      await route.fulfill(ok([WISHLIST_LAGOS]));
    });

    await page.goto('/wishlists');
    await page.getByRole('button', { name: 'New list' }).click();

    await page.getByLabel('Name').fill('Cement restock');
    await page.getByLabel('Description (optional)').fill('For the new site');
    await page.getByRole('button', { name: 'Create list' }).click();

    expect(created).toBe(true);
    await expect(page.locator('.list-card')).toHaveCount(1);
  });
});

test.describe('Product page → save to wishlist', () => {
  test.beforeEach(async ({ page }) => {
    await seedAuth(page);
    await page.route('**/api/catalog/categories', (route: Route) =>
      route.fulfill(ok([])),
    );
    await page.route('**/api/catalog/products*', (route: Route) =>
      route.fulfill(ok([RICE, DETERGENT])),
    );
    await page.route('**/api/catalog/products/rice-50kg', (route: Route) =>
      route.fulfill(ok(RICE)),
    );
  });

  test('save-to-wishlist posts to the first list and shows confirmation', async ({ page }) => {
    let addItemPosted = false;

    await page.route(/\/api\/wishlists$/, (route: Route) => {
      const req = route.request();
      if (req.method() === 'GET') return route.fulfill(ok([WISHLIST_SAVED]));
      return route.fulfill(ok(WISHLIST_SAVED));
    });
    await page.route(/\/api\/wishlists\/wl-2\/items/, async (route: Route) => {
      addItemPosted = true;
      await route.fulfill(ok({
        ...WISHLIST_SAVED,
        items: [{
          id: 'item-1',
          wishlist_id: 'wl-2',
          notes: null,
          created_at: '2026-09-24T09:00:00Z',
          updated_at: '2026-09-24T09:00:00Z',
          product: RICE,
        }],
      }));
    });

    await page.goto('/product/rice-50kg');
    await page.getByRole('button', { name: 'Save to wishlist' }).click();

    // Wait for the UI to settle first — the create→refresh→add chain spans several requests.
    await expect(page.getByRole('status')).toContainText('Saved to');
    await expect(page.getByRole('status')).toContainText('Saved items');
    expect(addItemPosted).toBe(true);
  });

  test('creating a brand-new list on the product page goes through create then add-item', async ({ page }) => {
    let created = false;
    let added = false;

    await page.route(/\/api\/wishlists$/, (route: Route) => {
      const req = route.request();
      if (req.method() === 'GET') return route.fulfill(ok([]));
      created = true;
      return route.fulfill(ok(WISHLIST_SAVED));
    });
    await page.route(/\/api\/wishlists\/wl-2\/items/, async (route: Route) => {
      added = true;
      await route.fulfill(ok({
        ...WISHLIST_SAVED,
        items: [{
          id: 'item-1',
          wishlist_id: 'wl-2',
          notes: null,
          created_at: '2026-09-24T09:00:00Z',
          updated_at: '2026-09-24T09:00:00Z',
          product: RICE,
        }],
      }));
    });

    await page.goto('/product/rice-50kg');
    await page.getByRole('button', { name: 'Save to wishlist' }).click();

    // Wait for the status confirmation — asserting the flags synchronously races the
    // create→refresh→add-item chain, which spans three mocked requests.
    await expect(page.getByRole('status')).toContainText('Saved to');
    await expect(page.getByRole('status')).toContainText('Saved items');
    expect(created).toBe(true);
    expect(added).toBe(true);
  });
});

test.describe('Wishlist detail page', () => {
  test.beforeEach(async ({ page }) => {
    await seedAuth(page);
    await page.route('**/api/catalog/categories', (route: Route) =>
      route.fulfill(ok([])),
    );
    await page.route('**/api/catalog/products*', (route: Route) =>
      route.fulfill(ok([])),
    );
  });

  test('renders items as product cards', async ({ page }) => {
    await page.route('**/api/wishlists/wl-1', (route: Route) =>
      route.fulfill(ok({
        ...WISHLIST_LAGOS,
        items: [
          { id: 'i1', wishlist_id: 'wl-1', notes: null,
            created_at: '2026-09-22T09:00:00Z', updated_at: '2026-09-22T09:00:00Z',
            product: RICE },
          { id: 'i2', wishlist_id: 'wl-1', notes: null,
            created_at: '2026-09-22T09:00:00Z', updated_at: '2026-09-22T09:00:00Z',
            product: DETERGENT },
        ],
      })),
    );

    await page.goto('/wishlists/wl-1');
    await expect(page.getByRole('heading', { name: 'Stocking the Lagos branch' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Long Grain Rice — 50kg Bag' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Detergent Powder — Carton of 24' })).toBeVisible();
    await expect(page.getByText('2 items saved')).toBeVisible();
  });

  test('shows the empty state when the wishlist has no items', async ({ page }) => {
    await page.route('**/api/wishlists/wl-2', (route: Route) =>
      route.fulfill(ok({ ...WISHLIST_SAVED, items: [] })),
    );

    await page.goto('/wishlists/wl-2');
    await expect(page.locator('.state-msg')).toContainText('Nothing here yet');
  });
});

test.describe('Catalogue product card → save to wishlist', () => {
  test.beforeEach(async ({ page }) => {
    await seedAuth(page);
    await page.route('**/api/catalog/categories', (route: Route) =>
      route.fulfill(ok([])),
    );
    await page.route('**/api/catalog/products*', (route: Route) =>
      route.fulfill(ok([RICE, DETERGENT])),
    );
  });

  test('card heart posts add-item to the first list and flips to saved', async ({ page }) => {
    let addItemPosted = false;

    await page.route(/\/api\/wishlists$/, (route: Route) => {
      const req = route.request();
      if (req.method() === 'GET') return route.fulfill(ok([WISHLIST_SAVED]));
      return route.fulfill(ok(WISHLIST_SAVED));
    });
    await page.route(/\/api\/wishlists\/wl-2\/items/, async (route: Route) => {
      addItemPosted = true;
      await route.fulfill(ok({
        ...WISHLIST_SAVED,
        items: [{
          id: 'item-1',
          wishlist_id: 'wl-2',
          notes: null,
          created_at: '2026-09-24T09:00:00Z',
          updated_at: '2026-09-24T09:00:00Z',
          product: RICE,
        }],
      }));
    });

    await page.goto('/catalog');
    await page.getByRole('button', { name: 'Save Long Grain Rice — 50kg Bag to wishlist' }).click();

    await expect(
      page.getByRole('button', { name: 'Long Grain Rice — 50kg Bag saved to wishlist' }),
    ).toBeVisible();
    expect(addItemPosted).toBe(true);
  });

  test('card heart on a guest account routes to login', async ({ page }) => {
    // Overwrite the seeded token so the card sees a signed-out visitor.
    await page.goto('/catalog');
    await page.evaluate(() => localStorage.removeItem('token'));

    await page.goto('/catalog');
    await page
      .getByRole('button', { name: 'Save Long Grain Rice — 50kg Bag to wishlist' })
      .click();

    await expect(page).toHaveURL(/\/login/);
  });
});