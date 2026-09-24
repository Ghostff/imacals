<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import { useRouter } from 'vue-router';
import { productService, formatNaira, type Product } from '@/services/product';
import { categoryService, type Category } from '@/services/category';
import { userService, type User } from '@/services/user';
import { integrationService, type Integration } from '@/services/integration';

const router = useRouter();

const products: Ref<Product[]>         = ref([]);
const categories: Ref<Category[]>     = ref([]);
const users: Ref<User[]>               = ref([]);
const integrations: Ref<Integration[]> = ref([]);
const loading: Ref<boolean>            = ref(true);
const error: Ref<string | null>        = ref(null);

onMounted(async () => {
  try {
    const [pList, cList, uList, iList] = await Promise.all([
      productService.index().catch(() => []),
      categoryService.index().catch(() => []),
      userService.index().catch(() => []),
      integrationService.index().catch(() => []),
    ]);
    products.value     = pList;
    categories.value   = cList;
    users.value        = uList;
    integrations.value = iList;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load dashboard data.';
  } finally {
    loading.value = false;
  }
});

const inStockCount: ComputedRef<number> = computed(() =>
  products.value.filter((p) => p.in_stock).length,
);

const recentProducts: ComputedRef<Product[]> = computed(() =>
  products.value.slice(0, 5),
);

const recentUsers: ComputedRef<User[]> = computed(() =>
  users.value.slice(0, 5),
);

const enabledIntegrationsCount: ComputedRef<number> = computed(() =>
  integrations.value.filter((i) => i.is_enabled).length,
);

function formatDate(iso: string | null | undefined): string {
  if (!iso) return '—';
  return new Date(iso).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}
</script>

<template>
  <div class="page">
    <!-- Header -->
    <div class="header-row">
      <div>
        <p class="page-label">Overview</p>
        <h1 class="page-title">Admin Dashboard</h1>
      </div>
      <div class="header-actions">
        <button class="btn-action-primary" type="button" @click="router.push('/products')">
          + Add Product
        </button>
        <button class="btn-action-secondary" type="button" @click="router.push({ path: '/users/all', query: { add: '1' } })">
          + Add User
        </button>
      </div>
    </div>

    <!-- Distribution Hub Banner -->
    <section class="hub-banner card" aria-label="Distribution Hub Status">
      <div class="hub-header">
        <div class="hub-badge-wrap">
          <span class="hub-status-dot"></span>
          <span class="hub-badge-text">LIVE DISTRIBUTION HUB</span>
        </div>
        <span class="hub-location">Aba, Abia State</span>
      </div>
      <div class="hub-content">
        <div>
          <h2 class="hub-title">Central Warehouse & Logistics Base</h2>
          <p class="hub-desc">
            Direct pick-and-pack warehouse operations from Aba. Same-day delivery inside Aba, with
            fleet dispatch across Southeast Nigeria. Online & phone orders unified into a single fulfillment flow.
          </p>
        </div>
        <div class="hub-pills">
          <div class="hub-pill">
            <span class="hub-pill-label">Storefront</span>
            <span class="hub-pill-val">imacals.com</span>
          </div>
          <div class="hub-pill">
            <span class="hub-pill-label">Phone Desk</span>
            <span class="hub-pill-val">Active</span>
          </div>
          <div class="hub-pill">
            <span class="hub-pill-label">Dispatch</span>
            <span class="hub-pill-val">Aba Fleet</span>
          </div>
        </div>
      </div>
    </section>

    <div v-if="loading" class="state-msg">Loading dashboard data…</div>
    <div v-else-if="error" class="state-msg state-msg--error">{{ error }}</div>

    <template v-else>
      <!-- Stats Grid -->
      <section class="stats-grid" aria-label="Key Metrics">
        <div class="stat-card card" @click="router.push('/products')">
          <div class="stat-top">
            <span class="stat-label">Catalogue Products</span>
            <span class="stat-icon" aria-hidden="true">📦</span>
          </div>
          <div class="stat-val">{{ products.length }}</div>
          <div class="stat-meta">
            <span class="stat-badge">{{ inStockCount }} in stock</span>
            <span class="stat-link">View catalogue →</span>
          </div>
        </div>

        <div class="stat-card card" @click="router.push('/products')">
          <div class="stat-top">
            <span class="stat-label">Product Categories</span>
            <span class="stat-icon" aria-hidden="true">🏷️</span>
          </div>
          <div class="stat-val">{{ categories.length }}</div>
          <div class="stat-meta">
            <span class="stat-sub">Organized product lines</span>
            <span class="stat-link">Manage →</span>
          </div>
        </div>

        <div class="stat-card card" @click="router.push('/users/all')">
          <div class="stat-top">
            <span class="stat-label">Total Users & Staff</span>
            <span class="stat-icon" aria-hidden="true">👥</span>
          </div>
          <div class="stat-val">{{ users.length }}</div>
          <div class="stat-meta">
            <span class="stat-sub">Staff & customer records</span>
            <span class="stat-link">View all →</span>
          </div>
        </div>

        <div class="stat-card card" @click="router.push('/integrations')">
          <div class="stat-top">
            <span class="stat-label">Integrations</span>
            <span class="stat-icon" aria-hidden="true">⚡</span>
          </div>
          <div class="stat-val">{{ integrations.length }}</div>
          <div class="stat-meta">
            <span class="stat-badge">{{ enabledIntegrationsCount }} active</span>
            <span class="stat-link">Configure →</span>
          </div>
        </div>
      </section>

      <!-- Quick Actions Grid -->
      <section class="section" aria-label="Quick Actions">
        <div class="section-header">
          <h2 class="section-title">Quick Actions</h2>
          <span class="section-sub">Immediate shortcuts for operations & administration</span>
        </div>
        <div class="actions-grid">
          <div class="action-card card" @click="router.push('/products')">
            <div class="action-icon-wrap">
              <span class="action-icon">➕</span>
            </div>
            <div class="action-body">
              <h3 class="action-title">Create Product</h3>
              <p class="action-desc">Add a sellable line with unit price (kobo), minimum order quantity, and stock.</p>
            </div>
          </div>

          <div class="action-card card" @click="router.push({ path: '/users/all', query: { add: '1' } })">
            <div class="action-icon-wrap">
              <span class="action-icon">👤</span>
            </div>
            <div class="action-body">
              <h3 class="action-title">Add Team / User</h3>
              <p class="action-desc">Create staff, order desk agents, warehouse pickers, or customer accounts.</p>
            </div>
          </div>

          <div class="action-card card" @click="router.push('/products')">
            <div class="action-icon-wrap">
              <span class="action-icon">📋</span>
            </div>
            <div class="action-body">
              <h3 class="action-title">Manage Catalogue</h3>
              <p class="action-desc">Update product pricing, stock availability, units, and categories.</p>
            </div>
          </div>

          <div class="action-card card" @click="router.push('/integrations')">
            <div class="action-icon-wrap">
              <span class="action-icon">🔌</span>
            </div>
            <div class="action-body">
              <h3 class="action-title">Integrations & API</h3>
              <p class="action-desc">Configure Paystack, SMS gateways, email delivery, and cloud storage.</p>
            </div>
          </div>

          <div class="action-card card" @click="router.push('/admin/map')">
            <div class="action-icon-wrap">
              <span class="action-icon">🗺️</span>
            </div>
            <div class="action-body">
              <h3 class="action-title">Delivery Zones & Map</h3>
              <p class="action-desc">Inspect Aba delivery polygons, zones, and regional distribution tariffs.</p>
            </div>
          </div>

          <div class="action-card card" @click="router.push('/models/domain')">
            <div class="action-icon-wrap">
              <span class="action-icon">🌐</span>
            </div>
            <div class="action-body">
              <h3 class="action-title">Operating Domains</h3>
              <p class="action-desc">Review location-scoped reference data, domains, and regional configurations.</p>
            </div>
          </div>
        </div>
      </section>

      <!-- Operational Channels -->
      <section class="section" aria-label="Operational Channels">
        <div class="section-header">
          <h2 class="section-title">Order Channels & Distribution</h2>
          <span class="section-sub">Two ordering channels, one central Aba warehouse</span>
        </div>
        <div class="channels-grid">
          <div class="channel-card card">
            <div class="channel-header">
              <div class="channel-title-wrap">
                <span class="channel-glyph">🏬</span>
                <span class="channel-name">Base Warehouse (Aba)</span>
              </div>
              <span class="channel-pill channel-pill--green">OPERATIONAL</span>
            </div>
            <p class="channel-desc">
              Central hub in Aba, Abia State. Direct pick-and-pack facility for all online and phone orders.
            </p>
            <div class="channel-stats">
              <div>
                <span class="channel-stat-lbl">Fulfilment</span>
                <span class="channel-stat-val">Same-Day Aba</span>
              </div>
              <div>
                <span class="channel-stat-lbl">Inventory</span>
                <span class="channel-stat-val">{{ inStockCount }} Lines Ready</span>
              </div>
            </div>
          </div>

          <div class="channel-card card">
            <div class="channel-header">
              <div class="channel-title-wrap">
                <span class="channel-glyph">📞</span>
                <span class="channel-name">Phone Order Desk</span>
              </div>
              <span class="channel-pill channel-pill--green">ACTIVE</span>
            </div>
            <p class="channel-desc">
              Staff take orders over the phone and enter them directly into the system on the customer's behalf.
            </p>
            <div class="channel-stats">
              <div>
                <span class="channel-stat-lbl">Record Parity</span>
                <span class="channel-stat-val">100% Unified</span>
              </div>
              <div>
                <span class="channel-stat-lbl">Role</span>
                <span class="channel-stat-val">Order Desk</span>
              </div>
            </div>
          </div>

          <div class="channel-card card">
            <div class="channel-header">
              <div class="channel-title-wrap">
                <span class="channel-glyph">🌐</span>
                <span class="channel-name">Online Storefront</span>
              </div>
              <span class="channel-pill channel-pill--green">CONNECTED</span>
            </div>
            <p class="channel-desc">
              Customer-facing web catalogue at <strong>imacals.com</strong> with direct self-checkout and tracking.
            </p>
            <div class="channel-stats">
              <div>
                <span class="channel-stat-lbl">Catalogue</span>
                <span class="channel-stat-val">Live Public</span>
              </div>
              <div>
                <span class="channel-stat-lbl">Coverage</span>
                <span class="channel-stat-val">Nigeria</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Tables Grid: Recent Products & Recent Users -->
      <section class="tables-grid" aria-label="Recent Records">
        <!-- Products Table Card -->
        <div class="overview-table-card card">
          <div class="overview-table-header">
            <div>
              <h3 class="overview-table-title">Recent Products</h3>
              <p class="overview-table-sub">Latest additions to the Aba warehouse catalogue</p>
            </div>
            <button class="btn-table-link" type="button" @click="router.push('/products')">
              View All ({{ products.length }}) →
            </button>
          </div>

          <div class="table-wrap">
            <table class="overview-table">
              <thead>
                <tr>
                  <th>Product</th>
                  <th>Unit</th>
                  <th>Price</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="recentProducts.length === 0">
                  <td colspan="4" class="empty-cell">No products added yet.</td>
                </tr>
                <tr v-for="p in recentProducts" :key="p.id" class="clickable-row" @click="router.push('/products')">
                  <td class="cell-name">{{ p.name }}</td>
                  <td class="cell-muted">{{ p.unit }}</td>
                  <td class="cell-price">{{ formatNaira(p.unit_price_kobo) }}</td>
                  <td>
                    <span :class="['badge', p.in_stock ? 'badge--green' : 'badge--red']">
                      {{ p.in_stock ? 'In Stock' : 'Out of Stock' }}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Users Table Card -->
        <div class="overview-table-card card">
          <div class="overview-table-header">
            <div>
              <h3 class="overview-table-title">Recent Users & Staff</h3>
              <p class="overview-table-sub">Registered team members and customer accounts</p>
            </div>
            <button class="btn-table-link" type="button" @click="router.push('/users/all')">
              View All ({{ users.length }}) →
            </button>
          </div>

          <div class="table-wrap">
            <table class="overview-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Job Title / Role</th>
                  <th>Organization</th>
                  <th>Joined</th>
                </tr>
              </thead>
              <tbody>
                <tr v-if="recentUsers.length === 0">
                  <td colspan="4" class="empty-cell">No users found.</td>
                </tr>
                <tr v-for="u in recentUsers" :key="u.id" class="clickable-row" @click="router.push(`/users/${u.id}`)">
                  <td class="cell-name">{{ u.first_name }} {{ u.last_name }}</td>
                  <td>
                    <span v-if="u.user_role" class="badge">{{ u.user_role.title }}</span>
                    <span v-else class="cell-muted">—</span>
                  </td>
                  <td>
                    <span v-if="u.organizations.length > 0" class="org-chip">{{ u.organizations[0].name }}</span>
                    <span v-else class="cell-muted">—</span>
                  </td>
                  <td class="cell-muted">{{ formatDate(u.created_at) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </template>
  </div>
</template>

<style scoped>
.page {
  padding: 40px var(--spacing-lg);
  max-width: 1280px;
  margin: 0 auto;
}

/* ── Header ── */
.header-row {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
  flex-wrap: wrap;
}

.page-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-sm);
}

.page-title {
  font-family: var(--font-display);
  font-size: 2.25rem;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--color-primary);
  line-height: 1.15;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.btn-action-primary {
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-on-primary);
  background-color: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 10px 20px;
  cursor: pointer;
  transition: opacity 0.15s, transform 0.15s;
}

.btn-action-primary:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn-action-secondary {
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-primary);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 9px 18px;
  cursor: pointer;
  transition: border-color 0.15s, transform 0.15s;
}

.btn-action-secondary:hover {
  border-color: var(--color-primary);
  transform: translateY(-1px);
}

/* ── Card Base ── */
.card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg, 12px);
  padding: 24px;
}

/* ── Distribution Hub Banner ── */
.hub-banner {
  margin-bottom: var(--spacing-lg);
  background: linear-gradient(135deg, var(--color-surface) 0%, color-mix(in srgb, var(--color-surface) 88%, var(--color-tertiary)) 100%);
  border: 1px solid color-mix(in srgb, var(--color-border) 70%, var(--color-tertiary));
  position: relative;
  overflow: hidden;
}

.hub-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.hub-badge-wrap {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: color-mix(in srgb, var(--color-neutral) 85%, transparent);
  border: 1px solid var(--color-border);
  padding: 4px 10px;
  border-radius: 999px;
}

.hub-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--color-tertiary);
  box-shadow: 0 0 8px var(--color-tertiary);
}

.hub-badge-text {
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.08em;
  font-weight: 600;
  color: var(--color-tertiary);
}

.hub-location {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.04em;
  color: var(--color-secondary);
  font-weight: 500;
}

.hub-content {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 24px;
  align-items: center;
}

.hub-title {
  font-family: var(--font-display);
  font-size: 1.35rem;
  font-weight: 600;
  color: var(--color-primary);
  margin-bottom: 6px;
}

.hub-desc {
  font-family: var(--font-body);
  font-size: 0.9rem;
  line-height: 1.5;
  color: var(--color-secondary);
  max-width: 680px;
}

.hub-pills {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.hub-pill {
  display: flex;
  flex-direction: column;
  background: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 8px 14px;
  min-width: 105px;
}

.hub-pill-label {
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.hub-pill-val {
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-primary);
}

/* ── Stats Grid ── */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}

.stat-card {
  padding: 20px;
  cursor: pointer;
  transition: transform 0.15s, border-color 0.15s;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.stat-card:hover {
  transform: translateY(-2px);
  border-color: var(--color-tertiary);
}

.stat-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.stat-label {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.stat-icon {
  font-size: 1.15rem;
  opacity: 0.8;
}

.stat-val {
  font-family: var(--font-display);
  font-size: 2rem;
  font-weight: 600;
  color: var(--color-primary);
  line-height: 1;
  margin-bottom: 12px;
}

.stat-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 0.75rem;
}

.stat-badge {
  font-family: var(--font-label);
  font-size: 0.65rem;
  padding: 2px 7px;
  border-radius: var(--rounded-sm);
  background: color-mix(in srgb, var(--color-tertiary) 15%, transparent);
  color: var(--color-tertiary);
  border: 1px solid color-mix(in srgb, var(--color-tertiary) 30%, transparent);
}

.stat-sub {
  color: var(--color-secondary);
}

.stat-link {
  color: var(--color-secondary);
  font-weight: 500;
  transition: color 0.15s;
}

.stat-card:hover .stat-link {
  color: var(--color-primary);
}

/* ── Section & Quick Actions ── */
.section {
  margin-bottom: 36px;
}

.section-header {
  margin-bottom: 16px;
}

.section-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-primary);
  margin-bottom: 4px;
}

.section-sub {
  font-family: var(--font-body);
  font-size: 0.85rem;
  color: var(--color-secondary);
}

.actions-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.action-card {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 18px;
  cursor: pointer;
  transition: transform 0.15s, border-color 0.15s;
}

.action-card:hover {
  transform: translateY(-2px);
  border-color: var(--color-primary);
}

.action-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--rounded-md);
  background: var(--color-neutral);
  border: 1px solid var(--color-border);
  flex-shrink: 0;
}

.action-icon {
  font-size: 1.15rem;
}

.action-body {
  flex: 1;
}

.action-title {
  font-family: var(--font-display);
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--color-primary);
  margin-bottom: 4px;
}

.action-desc {
  font-family: var(--font-body);
  font-size: 0.8rem;
  line-height: 1.45;
  color: var(--color-secondary);
}

/* ── Channels Grid ── */
.channels-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.channel-card {
  padding: 20px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.channel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.channel-title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.channel-glyph {
  font-size: 1.1rem;
}

.channel-name {
  font-family: var(--font-display);
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--color-primary);
}

.channel-pill {
  font-family: var(--font-label);
  font-size: 0.625rem;
  letter-spacing: 0.05em;
  padding: 2px 7px;
  border-radius: var(--rounded-sm);
  font-weight: 600;
}

.channel-pill--green {
  background: color-mix(in srgb, var(--color-tertiary) 15%, transparent);
  color: var(--color-tertiary);
  border: 1px solid color-mix(in srgb, var(--color-tertiary) 35%, transparent);
}

.channel-desc {
  font-family: var(--font-body);
  font-size: 0.825rem;
  color: var(--color-secondary);
  line-height: 1.45;
  margin-bottom: 16px;
}

.channel-stats {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--color-divider);
}

.channel-stat-lbl {
  display: block;
  font-family: var(--font-label);
  font-size: 0.625rem;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: 2px;
}

.channel-stat-val {
  display: block;
  font-family: var(--font-body);
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-primary);
}

/* ── Tables Grid ── */
.tables-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.overview-table-card {
  padding: 0;
  overflow: hidden;
}

.overview-table-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border);
}

.overview-table-title {
  font-family: var(--font-display);
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--color-primary);
  margin-bottom: 2px;
}

.overview-table-sub {
  font-family: var(--font-body);
  font-size: 0.775rem;
  color: var(--color-secondary);
}

.btn-table-link {
  font-family: var(--font-body);
  font-size: 0.8rem;
  color: var(--color-secondary);
  background: none;
  border: none;
  cursor: pointer;
  transition: color 0.15s;
  padding: 0;
}

.btn-table-link:hover {
  color: var(--color-primary);
  text-decoration: underline;
}

.table-wrap {
  overflow-x: auto;
}

.overview-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--font-body);
  font-size: 0.825rem;
}

.overview-table th {
  text-align: left;
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  padding: 10px 20px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-neutral);
}

.overview-table td {
  padding: 12px 20px;
  color: var(--color-primary);
  border-bottom: 1px solid var(--color-divider);
  vertical-align: middle;
}

.overview-table tbody tr:last-child td {
  border-bottom: none;
}

.clickable-row {
  cursor: pointer;
  transition: background-color 0.15s;
}

.clickable-row:hover td {
  background-color: var(--color-neutral);
}

.cell-name {
  font-weight: 500;
}

.cell-price {
  font-weight: 600;
  color: var(--color-tertiary);
}

.cell-muted {
  color: var(--color-secondary);
}

.empty-cell {
  text-align: center;
  color: var(--color-secondary);
  padding: 28px !important;
}

/* ── Badges ── */
.badge {
  display: inline-block;
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.04em;
  padding: 2px 7px;
  border-radius: var(--rounded-sm);
  border: 1px solid var(--color-border);
  color: var(--color-secondary);
  background: var(--color-neutral);
}

.badge--green {
  border-color: color-mix(in srgb, var(--color-tertiary) 40%, transparent);
  color: var(--color-tertiary);
  background: color-mix(in srgb, var(--color-tertiary) 10%, transparent);
}

.badge--red {
  border-color: rgba(220, 38, 38, 0.4);
  color: #ef4444;
  background: rgba(220, 38, 38, 0.1);
}

.org-chip {
  display: inline-block;
  font-family: var(--font-label);
  font-size: 0.65rem;
  padding: 2px 6px;
  border-radius: var(--rounded-sm);
  border: 1px solid var(--color-border);
  color: var(--color-secondary);
}

/* ── Loading / Error ── */
.state-msg {
  font-family: var(--font-body);
  color: var(--color-secondary);
  margin-top: 32px;
  font-size: 0.95rem;
}

.state-msg--error {
  color: var(--color-tertiary);
}

/* ── Responsive ── */
@media (max-width: 1024px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  .actions-grid, .channels-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  .tables-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .stats-grid, .actions-grid, .channels-grid {
    grid-template-columns: 1fr;
  }
  .hub-content {
    grid-template-columns: 1fr;
  }
}
</style>
