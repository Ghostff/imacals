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

async function mockCommon(page: Page): Promise<void> {
  await page.route('**/api/catalog/categories', (route: Route) => route.fulfill(ok([])));
  await page.route('**/api/catalog/products*', (route: Route) => route.fulfill(ok([])));
  await page.route('**/api/auth/me', (route: Route) => route.fulfill(ok({ user: MOCK_USER })));
  await page.route('**/api/wishlists', (route: Route) =>
    route.fulfill(
      ok([
        {
          id: 'wl-1',
          customer_id: 'cust-1',
          name: 'Monthly Provisions',
          description: 'Aba store restock',
          item_count: 2,
          created_at: '2026-06-01T10:00:00Z',
          updated_at: '2026-06-01T10:00:00Z',
        },
      ]),
    ),
  );
}

test.describe('Customer Dashboard (/account)', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to /login first to establish domain before touching localStorage
    await page.goto('/login');
    await page.evaluate((token) => {
      localStorage.clear();
      localStorage.setItem('token', token);
    }, MOCK_TOKEN);
    await mockCommon(page);
  });

  test('renders customer dashboard header with customer greeting and tabs', async ({ page }) => {
    await page.goto('/account');
    await expect(page.getByRole('heading', { name: 'Hello, Chukwudi' })).toBeVisible();
    await expect(page.getByText('Customer Dashboard')).toBeVisible();
    await expect(page.getByText('Aba Warehouse: Dispatches Active')).toBeVisible();

    // Verify Tab buttons
    const nav = page.getByRole('navigation', { name: 'Dashboard sections' });
    await expect(nav.getByRole('button', { name: 'Overview' })).toBeVisible();
    await expect(nav.getByRole('button', { name: /Orders/ })).toBeVisible();
    await expect(nav.getByRole('button', { name: /Delivery Addresses/ })).toBeVisible();
    await expect(nav.getByRole('button', { name: /Saved Wishlists/ })).toBeVisible();
    await expect(nav.getByRole('button', { name: 'Personal Details' })).toBeVisible();
  });

  test('overview tab renders metric cards and active delivery spotlight', async ({ page }) => {
    await page.goto('/account');

    // Metrics
    await expect(page.locator('.metric-label', { hasText: 'In-Flight Orders' })).toBeVisible();
    await expect(page.locator('.metric-label', { hasText: 'Total Orders' })).toBeVisible();
    await expect(page.locator('.metric-label', { hasText: 'Delivery Addresses' })).toBeVisible();
    await expect(page.locator('.metric-label', { hasText: 'Wishlist Items' })).toBeVisible();

    // Spotlight active order
    await expect(page.getByText('Active Fulfilment')).toBeVisible();
    await expect(page.getByRole('heading', { name: 'IMC-849201' })).toBeVisible();
    await expect(page.getByText('Origin: Aba Central Depot')).toBeVisible();

    // Aba depot facts
    await expect(page.getByRole('heading', { name: 'Base Warehouse' })).toBeVisible();
    await expect(page.getByText('Factory Road, Aba, Abia State')).toBeVisible();
  });

  test('switching to orders tab lists orders and displays order channel badges', async ({ page }) => {
    await page.goto('/account');
    await page.getByRole('button', { name: /Orders/ }).click();

    await expect(page.getByRole('heading', { name: 'Order Book & Dispatch History' })).toBeVisible();
    await expect(page.getByText('IMC-849201')).toBeVisible();
    await expect(page.getByText('IMC-723140')).toBeVisible();

    // Verify phone order badge is present
    await expect(page.getByText('📞 Phone Order').first()).toBeVisible();

    // Filter by phone orders
    await page.getByRole('button', { name: /Phone Orders/ }).click();
    await expect(page.getByText('IMC-723140')).toBeVisible();
    await expect(page.getByText('IMC-849201')).not.toBeVisible();
  });

  test('clicking Details & Receipt opens the Order Details modal', async ({ page }) => {
    await page.goto('/account?tab=orders');

    await page.getByRole('button', { name: 'Details & Receipt' }).first().click();

    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole('heading', { name: /Order IMC-/ })).toBeVisible();
    await expect(dialog.getByText('Status Progression')).toBeVisible();
    await expect(dialog.getByText('Grand Total')).toBeVisible();

    // Close modal
    await dialog.getByRole('button', { name: 'Close', exact: true }).click();
    await expect(dialog).not.toBeVisible();
  });

  test('delivery addresses tab allows adding a new address', async ({ page }) => {
    await page.goto('/account');
    await page.getByRole('button', { name: /Delivery Addresses/ }).click();

    await expect(page.getByRole('heading', { name: 'Saved Delivery Addresses' })).toBeVisible();
    await expect(page.getByText('Main Commercial Store')).toBeVisible();

    // Open Add Address modal
    await page.getByRole('button', { name: '+ Add New Address' }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();

    await page.locator('#addr_label').fill('New Market Branch');
    await page.locator('#addr_name').fill('Emeka Eze');
    await page.locator('#addr_phone').fill('0802 999 8888');
    await page.locator('#addr_street').fill('25 Market Road');
    await page.locator('#addr_landmark').fill('Opposite Enyimba Mall');
    await page.locator('#addr_city').fill('Aba');

    await dialog.getByRole('button', { name: 'Save Address' }).click();

    // Verify new address appears in the list
    await expect(page.getByText('New Market Branch')).toBeVisible();
    await expect(page.getByText('25 Market Road')).toBeVisible();
  });

  test('wishlists tab displays saved wishlists', async ({ page }) => {
    await page.goto('/account?tab=wishlists');

    await expect(page.getByRole('heading', { name: 'Saved Wishlists & Recurring Stock' })).toBeVisible();
    await expect(page.getByText('Monthly Provisions')).toBeVisible();
    await expect(page.getByText('Aba store restock')).toBeVisible();
    await expect(page.getByRole('link', { name: 'Open List' })).toBeVisible();
  });

  test('personal details tab allows editing profile and persists updates', async ({ page }) => {
    await page.route('**/api/users/user-cust-1', (route: Route) => {
      if (route.request().method() === 'PUT') {
        return route.fulfill(
          ok({
            ...MOCK_USER,
            first_name: 'Chukwudi Edited',
            phone: '0809 111 2222',
          }),
        );
      }
      return route.fulfill(ok(MOCK_USER));
    });

    await page.goto('/account?tab=profile');

    await expect(page.getByRole('heading', { name: 'Personal Details' })).toBeVisible();
    await expect(page.getByText('chukwudi@example.com')).toBeVisible();

    // Click edit
    await page.getByRole('button', { name: 'Edit' }).click();
    await page.locator('#edit_fname').fill('Chukwudi Edited');
    await page.locator('#edit_phone').fill('0809 111 2222');

    await page.getByRole('button', { name: 'Save Changes' }).click();

    await expect(page.getByText('Profile updated successfully')).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Hello, Chukwudi Edited' })).toBeVisible();
  });

  test('re-order button adds past items into cart and redirects to /cart', async ({ page }) => {
    await page.goto('/account?tab=orders');

    await page.getByRole('button', { name: 'Re-order All' }).first().click();

    await expect(page).toHaveURL(/\/cart/);
  });

  test('editing live address on dashboard immediately updates primary location and personal details', async ({ page }) => {
    await page.goto('/account');

    // Click Edit Address on Primary Delivery Location card
    await page.getByRole('button', { name: 'Edit Address' }).click();

    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();

    await page.locator('#addr_street').fill('84 Azikiwe Road');
    await page.locator('#addr_landmark').fill('Near CKC Cathedral');
    await page.locator('#addr_city').fill('Aba');

    await dialog.getByRole('button', { name: 'Update Address' }).click();

    await expect(dialog).not.toBeVisible();

    // Verify Primary Delivery Location card updates live
    await expect(page.locator('.live-val', { hasText: '84 Azikiwe Road' })).toBeVisible();
    await expect(page.locator('.live-landmark', { hasText: 'Near CKC Cathedral' })).toBeVisible();

    // Verify Personal Details reflects the updated live address
    await page.getByRole('button', { name: 'Personal Details' }).click();
    await expect(page.locator('.detail-val', { hasText: '84 Azikiwe Road' })).toBeVisible();
  });

  test('setting delivery address during checkout updates live address on customer dashboard', async ({ page }) => {
    await page.route('**/api/catalog/products*', (route: Route) =>
      route.fulfill(
        ok([
          {
            id: 'p1',
            slug: 'rice-50kg',
            name: 'Long Grain Rice — 50kg Bag',
            description: 'Parboiled rice',
            category_slug: 'foodstuff',
            category_name: 'Foodstuff',
            unit: 'bag (50kg)',
            unit_price_kobo: 8_950_000,
            min_order_quantity: 5,
            in_stock: true,
            image_url: null,
          },
        ]),
      ),
    );
    await page.route('**/api/orders', (route: Route) =>
      route.fulfill(
        ok({
          id: 'ord-new-chk',
          reference: 'IMC-999888',
          status: 'pending',
          total_kobo: 45_000_000,
          delivery_fee_kobo: 250_000,
          placed_at: '2026-09-26T10:00:00Z',
        }),
      ),
    );

    await page.evaluate(() => {
      const prod = {
        id: 'p1',
        slug: 'rice-50kg',
        name: 'Long Grain Rice — 50kg Bag',
        description: 'Parboiled rice',
        category_slug: 'foodstuff',
        category_name: 'Foodstuff',
        unit: 'bag (50kg)',
        unit_price_kobo: 8_950_000,
        min_order_quantity: 5,
        in_stock: true,
        image_url: null,
      };
      localStorage.setItem('cart', JSON.stringify([{ product: prod, quantity: 5 }]));
    });

    await page.goto('/checkout');

    await page.getByLabel('Delivery address').fill('55 Pound Road');
    await page.getByLabel('Town / city').fill('Aba');
    await page.locator('#note').fill('Offload near main gate');

    await page.getByRole('button', { name: 'Place order' }).click();

    await expect(page.getByRole('heading', { name: 'Reference IMC-999888' })).toBeVisible();

    // Click link to Customer Dashboard
    await page.getByRole('link', { name: 'Customer Dashboard' }).click();

    // Verify order reference and delivery address in the order book
    await expect(page.getByText('IMC-999888')).toBeVisible();
    await expect(page.getByText('55 Pound Road, Aba')).toBeVisible();

    // Switch to Overview tab to check Primary Delivery Location
    await page.getByRole('button', { name: 'Overview' }).click();
    await expect(page.locator('.live-val', { hasText: '55 Pound Road' })).toBeVisible();
  });
});
