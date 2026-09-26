<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, type Ref, type ComputedRef } from 'vue';
import { useRouter, useRoute, RouterLink } from 'vue-router';
import { useAuth, ApiException } from '@/composables/useAuth';
import { useCart } from '@/composables/useCart';
import { useWishlist } from '@/composables/useWishlist';
import { formatNaira, type Product } from '@/services/catalog';
import { customerAddressService, type CustomerAddress, type CreateAddressInput } from '@/services/customerAddress';
import { customerOrderService, type CustomerOrder, type OrderStatus } from '@/services/customerOrder';
import { SITE } from '@/site';

const router = useRouter();
const route  = useRoute();
const { user, isAuthenticated, logout, updateProfile, fetchMe } = useAuth();
const { add: addToCart } = useCart();
const { lists: wishlists, totalCount: totalWishlistCount, refresh: fetchWishlists } = useWishlist();

type TabKey = 'overview' | 'orders' | 'addresses' | 'wishlists' | 'profile';
const activeTab: Ref<TabKey> = ref('overview');

// Data state
const loadingOrders: Ref<boolean>       = ref(false);
const orders: Ref<CustomerOrder[]>      = ref([]);
const orderFilter: Ref<string>          = ref('all');
const orderSearch: Ref<string>          = ref('');
const selectedOrder: Ref<CustomerOrder | null> = ref(null);

const loadingAddresses: Ref<boolean>    = ref(false);
const addresses: Ref<CustomerAddress[]> = ref([]);
const addressModalOpen: Ref<boolean>    = ref(false);
const editingAddressId: Ref<string | null> = ref(null);
const addressForm: Ref<CreateAddressInput> = ref({
  label: 'Main Store',
  recipient_name: '',
  phone: '',
  street: '',
  landmark: '',
  city: 'Aba',
  state: 'Abia State',
  is_default: false,
  instructions: '',
});

// Profile edit state
const editingProfile: Ref<boolean>       = ref(false);
const savingProfile: Ref<boolean>        = ref(false);
const profileError: Ref<string | null>   = ref(null);
const profileSuccess: Ref<boolean>       = ref(false);
const toastMessage: Ref<string | null>   = ref(null);

const profileForm = ref({
  first_name: '',
  last_name: '',
  email: '',
  phone: '',
  street: '',
  landmark: '',
  city: 'Aba',
  state: 'Abia State',
  instructions: '',
});

function showToast(msg: string): void {
  toastMessage.value = msg;
  setTimeout(() => {
    if (toastMessage.value === msg) toastMessage.value = null;
  }, 3500);
}

// Live computed addresses
const defaultAddress: ComputedRef<CustomerAddress | null> = computed(() => {
  return addresses.value.find((a) => a.is_default) || addresses.value[0] || null;
});

const liveAddressText: ComputedRef<string> = computed(() => {
  if (!defaultAddress.value) return '';
  const landmarkPart = defaultAddress.value.landmark ? ` (${defaultAddress.value.landmark})` : '';
  return `${defaultAddress.value.street}${landmarkPart}, ${defaultAddress.value.city}, ${defaultAddress.value.state}`;
});

// Stats computed
const activeOrders: ComputedRef<CustomerOrder[]> = computed(() =>
  orders.value.filter((o) => ['pending', 'confirmed', 'picked', 'dispatched'].includes(o.status)),
);

const spotlightOrder: ComputedRef<CustomerOrder | null> = computed(() => {
  return activeOrders.value[0] || null;
});

const completedOrdersCount: ComputedRef<number> = computed(() =>
  orders.value.filter((o) => o.status === 'delivered').length,
);

const filteredOrders: ComputedRef<CustomerOrder[]> = computed(() => {
  return orders.value.filter((order) => {
    if (orderFilter.value === 'active' && !['pending', 'confirmed', 'picked', 'dispatched'].includes(order.status)) {
      return false;
    }
    if (orderFilter.value === 'delivered' && order.status !== 'delivered') {
      return false;
    }
    if (orderFilter.value === 'phone' && order.channel !== 'phone') {
      return false;
    }
    if (orderSearch.value.trim()) {
      const q = orderSearch.value.trim().toLowerCase();
      const refMatch = order.reference.toLowerCase().includes(q);
      const itemMatch = order.items.some((i) => i.name.toLowerCase().includes(q));
      const cityMatch = order.city.toLowerCase().includes(q);
      return refMatch || itemMatch || cityMatch;
    }
    return true;
  });
});

function initProfileForm(): void {
  if (user.value) {
    profileForm.value = {
      first_name:   user.value.first_name ?? '',
      last_name:    user.value.last_name ?? '',
      email:        user.value.email ?? '',
      phone:        user.value.phone ?? '',
      street:       defaultAddress.value?.street ?? '',
      landmark:     defaultAddress.value?.landmark ?? '',
      city:         defaultAddress.value?.city ?? 'Aba',
      state:        defaultAddress.value?.state ?? 'Abia State',
      instructions: defaultAddress.value?.instructions ?? '',
    };
  }
}

async function loadData(): Promise<void> {
  const userId = user.value?.id;
  const fullName = user.value ? `${user.value.first_name} ${user.value.last_name}`.trim() : undefined;
  const userPhone = user.value?.phone || undefined;
  const userEmail = user.value?.email || undefined;

  loadingAddresses.value = true;
  loadingOrders.value = true;

  try {
    addresses.value = await customerAddressService.listAddresses(userId, {
      name: fullName,
      phone: userPhone,
    });
  } catch (err) {
    console.error('Failed to load addresses:', err);
  } finally {
    loadingAddresses.value = false;
  }

  try {
    orders.value = await customerOrderService.listOrders(userId, {
      name: fullName,
      phone: userPhone,
      email: userEmail,
      address: liveAddressText.value || undefined,
      city: defaultAddress.value?.city,
      state: defaultAddress.value?.state,
    });
  } catch (err) {
    console.error('Failed to load orders:', err);
  } finally {
    loadingOrders.value = false;
  }

  try {
    await fetchWishlists();
  } catch (err) {
    console.error('Failed to load wishlists:', err);
  }
}

// Tab handling
function setTab(tab: TabKey): void {
  activeTab.value = tab;
  router.replace({ query: { ...route.query, tab } });
}

// Profile update with live address sync
async function saveProfile(): Promise<void> {
  profileError.value   = null;
  profileSuccess.value = false;
  savingProfile.value  = true;

  try {
    const updatedFname = profileForm.value.first_name.trim();
    const updatedLname = profileForm.value.last_name.trim();
    const updatedEmail = profileForm.value.email.trim();
    const updatedPhone = profileForm.value.phone.trim() || undefined;

    await updateProfile({
      first_name: updatedFname,
      last_name:  updatedLname,
      email:      updatedEmail,
      phone:      updatedPhone,
    });

    profileSuccess.value = true;
    editingProfile.value = false;

    // Sync live default address recipient details & address fields with updated profile
    const userId = user.value?.id;
    const fullName = `${updatedFname} ${updatedLname}`.trim();

    if (userId && profileForm.value.street.trim()) {
      await customerAddressService.saveLivePrimaryAddress(
        {
          recipient_name: fullName,
          phone: updatedPhone || defaultAddress.value?.phone || '',
          street: profileForm.value.street.trim(),
          landmark: profileForm.value.landmark.trim() || undefined,
          city: profileForm.value.city.trim() || 'Aba',
          state: profileForm.value.state.trim() || 'Abia State',
          instructions: profileForm.value.instructions.trim() || undefined,
        },
        userId,
      );
      addresses.value = await customerAddressService.listAddresses(userId);
    } else if (defaultAddress.value && userId) {
      await customerAddressService.updateAddress(
        defaultAddress.value.id,
        {
          recipient_name: fullName,
          phone: updatedPhone || defaultAddress.value.phone,
        },
        userId,
      );
      addresses.value = await customerAddressService.listAddresses(userId);
    }

    orders.value = await customerOrderService.listOrders(userId, {
      name: fullName,
      phone: updatedPhone,
      email: updatedEmail,
      address: liveAddressText.value,
      city: defaultAddress.value?.city,
      state: defaultAddress.value?.state,
    });

    showToast('Personal details and live address updated successfully');
    initProfileForm();
    setTimeout(() => { profileSuccess.value = false; }, 3000);
  } catch (e: unknown) {
    profileError.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Failed to update profile.';
  } finally {
    savingProfile.value = false;
  }
}

// Addresses
function openAddAddressModal(): void {
  editingAddressId.value = null;
  const fullName = user.value ? `${user.value.first_name} ${user.value.last_name}`.trim() : '';
  addressForm.value = {
    label: addresses.value.length === 0 ? 'Main Store' : 'Warehouse / Shop',
    recipient_name: fullName,
    phone: user.value?.phone || '',
    street: '',
    landmark: '',
    city: 'Aba',
    state: 'Abia State',
    is_default: addresses.value.length === 0,
    instructions: '',
  };
  addressModalOpen.value = true;
}

function openEditAddressModal(addr: CustomerAddress): void {
  editingAddressId.value = addr.id;
  addressForm.value = {
    label: addr.label,
    recipient_name: addr.recipient_name,
    phone: addr.phone,
    street: addr.street,
    landmark: addr.landmark || '',
    city: addr.city,
    state: addr.state,
    is_default: addr.is_default,
    instructions: addr.instructions || '',
  };
  addressModalOpen.value = true;
}

function closeAddressModal(): void {
  addressModalOpen.value = false;
  editingAddressId.value = null;
}

async function saveAddress(): Promise<void> {
  if (!addressForm.value.recipient_name.trim() || !addressForm.value.phone.trim() || !addressForm.value.street.trim()) {
    return;
  }

  const userId = user.value?.id;
  if (editingAddressId.value) {
    await customerAddressService.updateAddress(editingAddressId.value, addressForm.value, userId);
    showToast('Delivery address updated');
  } else {
    await customerAddressService.createAddress(addressForm.value, userId);
    showToast('Delivery address saved as live destination');
  }
  closeAddressModal();
  addresses.value = await customerAddressService.listAddresses(userId);

  // Keep live orders in sync with new default address
  const fullName = user.value ? `${user.value.first_name} ${user.value.last_name}`.trim() : undefined;
  orders.value = await customerOrderService.listOrders(userId, {
    name: fullName,
    phone: user.value?.phone || undefined,
    address: liveAddressText.value,
    city: defaultAddress.value?.city,
    state: defaultAddress.value?.state,
  });
  initProfileForm();
}

async function setDefaultAddress(id: string): Promise<void> {
  const userId = user.value?.id;
  await customerAddressService.setDefaultAddress(id, userId);
  addresses.value = await customerAddressService.listAddresses(userId);

  // Sync orders with new live default address
  orders.value = await customerOrderService.listOrders(userId, {
    address: liveAddressText.value,
    city: defaultAddress.value?.city,
    state: defaultAddress.value?.state,
  });
  initProfileForm();
  showToast('Default delivery address updated');
}

async function deleteAddress(id: string): Promise<void> {
  const userId = user.value?.id;
  await customerAddressService.deleteAddress(id, userId);
  addresses.value = await customerAddressService.listAddresses(userId);
  initProfileForm();
  showToast('Address removed');
}

// Re-order past order lines
function orderPromotionalSavingsKobo(order?: CustomerOrder | null): number {
  if (!order?.items) return 0;
  return order.items.reduce((acc, item) => {
    if (item.original_unit_price_kobo && item.original_unit_price_kobo > item.unit_price_kobo) {
      return acc + (item.original_unit_price_kobo - item.unit_price_kobo) * item.quantity;
    }
    return acc;
  }, 0);
}

async function reorder(order: CustomerOrder): Promise<void> {
  let count = 0;
  for (const item of order.items) {
    try {
      const isDiscountedLine = !!(item.original_unit_price_kobo && item.original_unit_price_kobo > item.unit_price_kobo);
      const prod: Product = {
        id: item.product_id,
        slug: item.slug || 'product',
        name: item.name,
        description: '',
        category_slug: 'foodstuff',
        category_name: 'Catalogue Item',
        unit: item.unit,
        unit_price_kobo: isDiscountedLine ? (item.original_unit_price_kobo as number) : item.unit_price_kobo,
        discount_price_kobo: isDiscountedLine ? item.unit_price_kobo : undefined,
        min_order_quantity: 1,
        in_stock: true,
        image_url: null,
      };
      addToCart(prod, item.quantity);
      count += 1;
    } catch (err) {
      console.warn('Could not re-add item:', item.name, err);
    }
  }
  showToast(`Re-added ${count} item${count === 1 ? '' : 's'} to your cart`);
  router.push('/cart');
}

function onSignOut(): void {
  logout();
  router.push('/');
}

// Helper for status styling & step calculation
function getStatusStep(status: OrderStatus): number {
  switch (status) {
    case 'pending': return 1;
    case 'confirmed': return 1;
    case 'picked': return 2;
    case 'dispatched': return 3;
    case 'delivered': return 4;
    default: return 0;
  }
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return new Intl.DateTimeFormat('en-NG', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(d);
  } catch {
    return iso;
  }
}

watch(
  () => route.query.tab,
  (newTab) => {
    if (newTab && ['overview', 'orders', 'addresses', 'wishlists', 'profile'].includes(newTab as string)) {
      activeTab.value = newTab as TabKey;
    }
  },
  { immediate: true },
);

function handleExternalAddressOrUserChange(): void {
  loadData().then(() => initProfileForm());
}

onMounted(async () => {
  if (!isAuthenticated.value) {
    router.push('/login?redirect=/account');
    return;
  }
  await fetchMe();
  await loadData();
  initProfileForm();

  if (typeof window !== 'undefined') {
    window.addEventListener('imacals:address-changed', handleExternalAddressOrUserChange);
    window.addEventListener('imacals:user-updated', handleExternalAddressOrUserChange);
  }
});

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('imacals:address-changed', handleExternalAddressOrUserChange);
    window.removeEventListener('imacals:user-updated', handleExternalAddressOrUserChange);
  }
});
</script>

<template>
  <div class="page customer-dashboard">
    <!-- Feedback Toast -->
    <transition name="fade">
      <div v-if="toastMessage" class="toast-banner" role="status" aria-live="polite">
        <span class="toast-indicator">●</span>
        <span>{{ toastMessage }}</span>
      </div>
    </transition>

    <!-- Top Dashboard Header -->
    <header class="dash-head">
      <div class="dash-head-meta">
        <div class="customer-badge-row">
          <p class="eyebrow">Customer Dashboard</p>
          <span class="hub-pill">
            <span class="live-dot" aria-hidden="true"></span>
            Aba Warehouse: Dispatches Active
          </span>
        </div>
        <!-- Keep exact H1 text for auth and e2e test compatibility -->
        <h1 class="section-title">Hello, {{ user?.first_name || 'Customer' }}</h1>
        <p class="dash-sub">
          Account Ref: <span class="mono-code">IMC-CUST-{{ user?.id ? user.id.slice(0, 8).toUpperCase() : '001' }}</span>
          <span class="pipe-sep">|</span>
          Base Warehouse: <span class="location-tag">Aba, Abia State</span>
        </p>
      </div>

      <div class="dash-head-actions">
        <a class="phone-action-btn" :href="SITE.orderLineHref">
          <span class="phone-action-icon">📞</span>
          <div class="phone-action-text">
            <span class="phone-action-label">Aba Order Desk</span>
            <span class="phone-action-number">{{ SITE.orderLine }}</span>
          </div>
        </a>
        <button class="btn-secondary sign-out-btn" type="button" @click="onSignOut">Sign out</button>
      </div>
    </header>

    <!-- Tab Navigation -->
    <nav class="dash-tabs" aria-label="Dashboard sections">
      <button
        class="tab-btn"
        :class="{ 'tab-btn--active': activeTab === 'overview' }"
        type="button"
        @click="setTab('overview')"
      >
        <span class="tab-icon">📊</span>
        <span>Overview</span>
      </button>

      <button
        class="tab-btn"
        :class="{ 'tab-btn--active': activeTab === 'orders' }"
        type="button"
        @click="setTab('orders')"
      >
        <span class="tab-icon">📦</span>
        <span>Orders</span>
        <span v-if="orders.length" class="tab-counter">{{ orders.length }}</span>
      </button>

      <button
        class="tab-btn"
        :class="{ 'tab-btn--active': activeTab === 'addresses' }"
        type="button"
        @click="setTab('addresses')"
      >
        <span class="tab-icon">📍</span>
        <span>Delivery Addresses</span>
        <span v-if="addresses.length" class="tab-counter">{{ addresses.length }}</span>
      </button>

      <button
        class="tab-btn"
        :class="{ 'tab-btn--active': activeTab === 'wishlists' }"
        type="button"
        @click="setTab('wishlists')"
      >
        <span class="tab-icon">♡</span>
        <span>Saved Wishlists</span>
        <span v-if="totalWishlistCount" class="tab-counter">{{ totalWishlistCount }}</span>
      </button>

      <button
        class="tab-btn"
        :class="{ 'tab-btn--active': activeTab === 'profile' }"
        type="button"
        @click="setTab('profile')"
      >
        <span class="tab-icon">👤</span>
        <span>Personal Details</span>
      </button>
    </nav>

    <!-- TAB 1: OVERVIEW -->
    <div v-if="activeTab === 'overview'" class="tab-pane">
      <!-- Quick Metric Cards -->
      <section class="metrics-grid">
        <div class="metric-card" @click="setTab('orders')">
          <div class="metric-header">
            <span class="metric-label">In-Flight Orders</span>
            <span class="metric-icon">🚚</span>
          </div>
          <p class="metric-value">{{ activeOrders.length }}</p>
          <p class="metric-desc">
            {{ activeOrders.length > 0 ? 'Orders in picking or dispatch' : 'No active deliveries currently' }}
          </p>
        </div>

        <div class="metric-card" @click="setTab('orders')">
          <div class="metric-header">
            <span class="metric-label">Total Orders</span>
            <span class="metric-icon">📦</span>
          </div>
          <p class="metric-value">{{ orders.length }}</p>
          <p class="metric-desc">{{ completedOrdersCount }} successfully delivered</p>
        </div>

        <div class="metric-card" @click="setTab('addresses')">
          <div class="metric-header">
            <span class="metric-label">Delivery Addresses</span>
            <span class="metric-icon">📍</span>
          </div>
          <p class="metric-value">{{ addresses.length }}</p>
          <p class="metric-desc">
            {{ defaultAddress ? `Live: ${defaultAddress.label} (${defaultAddress.city})` : 'No address set' }}
          </p>
        </div>

        <div class="metric-card" @click="setTab('wishlists')">
          <div class="metric-header">
            <span class="metric-label">Wishlist Items</span>
            <span class="metric-icon">♡</span>
          </div>
          <p class="metric-value">{{ totalWishlistCount }}</p>
          <p class="metric-desc">Across {{ wishlists.length }} saved lists</p>
        </div>
      </section>

      <!-- Active Delivery Spotlight (if an active order exists) -->
      <section v-if="spotlightOrder" class="card spotlight-card">
        <div class="spotlight-head">
          <div class="spotlight-badge-col">
            <span class="eyebrow">Active Fulfilment</span>
            <h2 class="spotlight-ref">{{ spotlightOrder.reference }}</h2>
          </div>
          <div class="spotlight-status-col">
            <span class="status-pill status-pill--dispatched">
              {{ spotlightOrder.status.toUpperCase() }}
            </span>
            <span class="spotlight-eta">Origin: Aba Central Depot (Factory Rd)</span>
          </div>
        </div>

        <!-- 4-step Tracker progress bar -->
        <div class="stepper" aria-label="Delivery progress">
          <div class="step-item" :class="{ 'step-item--done': getStatusStep(spotlightOrder.status) >= 1 }">
            <div class="step-dot">1</div>
            <span class="step-label">Confirmed</span>
          </div>
          <div class="step-line" :class="{ 'step-line--done': getStatusStep(spotlightOrder.status) >= 2 }"></div>
          <div class="step-item" :class="{ 'step-item--done': getStatusStep(spotlightOrder.status) >= 2 }">
            <div class="step-dot">2</div>
            <span class="step-label">Picked in Aba</span>
          </div>
          <div class="step-line" :class="{ 'step-line--done': getStatusStep(spotlightOrder.status) >= 3 }"></div>
          <div class="step-item" :class="{ 'step-item--done': getStatusStep(spotlightOrder.status) >= 3, 'step-item--active': spotlightOrder.status === 'dispatched' }">
            <div class="step-dot">3</div>
            <span class="step-label">Dispatched</span>
          </div>
          <div class="step-line" :class="{ 'step-line--done': getStatusStep(spotlightOrder.status) >= 4 }"></div>
          <div class="step-item" :class="{ 'step-item--done': getStatusStep(spotlightOrder.status) >= 4 }">
            <div class="step-dot">4</div>
            <span class="step-label">Delivered</span>
          </div>
        </div>

        <div class="spotlight-details">
          <div class="spotlight-info-col">
            <span class="info-label">Destination Address</span>
            <p class="info-val">
              {{ defaultAddress ? liveAddressText : (spotlightOrder.delivery_address + ', ' + spotlightOrder.city) }}
            </p>
          </div>
          <div class="spotlight-info-col">
            <span class="info-label">Order Items</span>
            <p class="info-val">
              {{ spotlightOrder.items.map((i) => `${i.quantity} × ${i.name}`).join(', ') }}
            </p>
          </div>
          <div class="spotlight-info-col">
            <span class="info-label">Total Amount</span>
            <p class="info-val total-val">{{ formatNaira(spotlightOrder.total_kobo) }}</p>
          </div>
        </div>

        <div class="spotlight-actions">
          <RouterLink class="btn-primary" to="/track">Track Live Timeline</RouterLink>
          <button class="btn-secondary" type="button" @click="selectedOrder = spotlightOrder">
            View Order Receipt
          </button>
        </div>
      </section>

      <!-- Live Primary Delivery Address Card -->
      <section class="card live-address-card">
        <div class="card-head">
          <div>
            <div class="live-addr-badge-row">
              <span class="eyebrow">Primary Delivery Location</span>
              <span v-if="defaultAddress" class="default-badge">LIVE DEFAULT</span>
            </div>
            <h2 class="card-title">{{ defaultAddress ? defaultAddress.label : 'No Delivery Address Set' }}</h2>
            <p class="card-subtitle">
              {{ defaultAddress ? 'What you set here is your active live dispatch destination across the store' : 'Add your address to enable direct deliveries from our Aba warehouse' }}
            </p>
          </div>
          <button v-if="defaultAddress" class="btn-edit" type="button" @click="openEditAddressModal(defaultAddress)">
            Edit Address
          </button>
          <button v-else class="btn-primary" type="button" @click="openAddAddressModal">
            + Add Delivery Address
          </button>
        </div>

        <div v-if="defaultAddress" class="live-address-details">
          <div class="live-address-grid">
            <div class="live-col">
              <span class="detail-label">Recipient & Receiving Phone</span>
              <p class="live-val"><strong>{{ defaultAddress.recipient_name }}</strong></p>
              <p class="live-phone mono-code">{{ defaultAddress.phone || user?.phone || 'No phone set' }}</p>
            </div>
            <div class="live-col live-col--wide">
              <span class="detail-label">Street Address & Landmark</span>
              <p class="live-val">{{ defaultAddress.street }}</p>
              <p v-if="defaultAddress.landmark" class="live-landmark">
                <span class="landmark-tag">Landmark:</span> {{ defaultAddress.landmark }}
              </p>
              <p class="live-city-state">{{ defaultAddress.city }}, {{ defaultAddress.state }}</p>
            </div>
            <div v-if="defaultAddress.instructions" class="live-col">
              <span class="detail-label">Special Offloading Instructions</span>
              <p class="live-instructions"><em>"{{ defaultAddress.instructions }}"</em></p>
            </div>
          </div>
        </div>

        <div v-else class="live-address-empty">
          <p class="empty-note">
            You do not have a default delivery address set yet. What you set as your address will immediately display here and autofill checkout.
          </p>
        </div>
      </section>

      <!-- Two-column Section: Recent Orders & Order Desk Quick Channel -->
      <div class="dash-split-grid">
        <!-- Recent Orders Summary -->
        <section class="card recent-orders-card">
          <div class="card-head">
            <div>
              <h2 class="card-title">Recent Orders</h2>
              <p class="card-subtitle">Both online and Aba desk phone orders are tracked here</p>
            </div>
            <button class="btn-link" type="button" @click="setTab('orders')">
              View All ({{ orders.length }}) →
            </button>
          </div>

          <div v-if="orders.length === 0" class="empty-state">
            <p class="empty-text">You have not placed any orders yet.</p>
            <RouterLink class="btn-secondary" to="/catalog">Browse Catalogue</RouterLink>
          </div>

          <div v-else class="recent-list">
            <div
              v-for="order in orders.slice(0, 3)"
              :key="order.id"
              class="recent-row"
              @click="selectedOrder = order"
            >
              <div class="recent-main">
                <div class="recent-meta-top">
                  <span class="mono-code">{{ order.reference }}</span>
                  <span class="channel-badge" :class="order.channel === 'phone' ? 'channel-badge--phone' : ''">
                    {{ order.channel === 'phone' ? '📞 Phone Order' : 'Online' }}
                  </span>
                  <span class="status-pill" :class="`status-pill--${order.status}`">
                    {{ order.status }}
                  </span>
                </div>
                <p class="recent-items-summary">
                  {{ order.items.map((i) => `${i.quantity} × ${i.name}`).join('; ') }}
                </p>
                <p class="recent-date">{{ formatDate(order.placed_at) }}</p>
              </div>

              <div class="recent-side">
                <span class="recent-price">{{ formatNaira(order.total_kobo) }}</span>
                <span class="recent-view-link">Details →</span>
              </div>
            </div>
          </div>
        </section>

        <!-- Warehouse & Support Quick Channel -->
        <aside class="card desk-card">
          <div class="desk-head">
            <span class="eyebrow">Aba Central Fulfilment</span>
            <h2 class="card-title">Base Warehouse</h2>
          </div>
          <p class="desk-copy">
            Imacals picks, packs, and dispatches all orders directly out of our base distribution warehouse in Aba.
          </p>

          <div class="desk-facts">
            <div class="fact-item">
              <span class="fact-label">Depot Location</span>
              <span class="fact-val">Factory Road, Aba, Abia State</span>
            </div>
            <div class="fact-item">
              <span class="fact-label">Local Aba Deliveries</span>
              <span class="fact-val">Same-day delivery (orders placed before 1:00 PM)</span>
            </div>
            <div class="fact-item">
              <span class="fact-label">Regional Transit</span>
              <span class="fact-val">24h – 48h to Umuahia, Owerri, Port Harcourt, and Environs</span>
            </div>
            <div class="fact-item">
              <span class="fact-label">Operating Hours</span>
              <span class="fact-val">Monday – Saturday: 7:30 AM – 6:00 PM</span>
            </div>
          </div>

          <div class="desk-phone-box">
            <span class="desk-phone-label">Need urgent wholesale assistance?</span>
            <a class="desk-phone-num" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>
            <span class="desk-phone-note">Call the desk to order or update dispatch instructions</span>
          </div>

          <div class="desk-actions">
            <RouterLink class="action-btn-link" to="/catalog">
              <span>Browse wholesale catalogue</span>
              <span aria-hidden="true">→</span>
            </RouterLink>
            <RouterLink class="action-btn-link" to="/track">
              <span>Track an order reference</span>
              <span aria-hidden="true">→</span>
            </RouterLink>
            <RouterLink class="action-btn-link" to="/cart">
              <span>Review active cart</span>
              <span aria-hidden="true">→</span>
            </RouterLink>
          </div>
        </aside>
      </div>
    </div>

    <!-- TAB 2: ORDERS & HISTORY -->
    <div v-else-if="activeTab === 'orders'" class="tab-pane">
      <div class="card orders-pane-card">
        <div class="card-head">
          <div>
            <h2 class="card-title">Order Book & Dispatch History</h2>
            <p class="card-subtitle">
              Every order — whether placed via online self-checkout or by phone with our Aba desk — is recorded here.
            </p>
          </div>
          <RouterLink class="btn-secondary" to="/catalog">New Order</RouterLink>
        </div>

        <!-- Filter and Search Row -->
        <div class="orders-toolbar">
          <div class="filter-pills" role="tablist" aria-label="Order status filter">
            <button
              class="filter-pill"
              :class="{ 'filter-pill--active': orderFilter === 'all' }"
              type="button"
              @click="orderFilter = 'all'"
            >
              All Orders ({{ orders.length }})
            </button>
            <button
              class="filter-pill"
              :class="{ 'filter-pill--active': orderFilter === 'active' }"
              type="button"
              @click="orderFilter = 'active'"
            >
              Active / In Transit ({{ activeOrders.length }})
            </button>
            <button
              class="filter-pill"
              :class="{ 'filter-pill--active': orderFilter === 'delivered' }"
              type="button"
              @click="orderFilter = 'delivered'"
            >
              Delivered ({{ completedOrdersCount }})
            </button>
            <button
              class="filter-pill"
              :class="{ 'filter-pill--active': orderFilter === 'phone' }"
              type="button"
              @click="orderFilter = 'phone'"
            >
              Phone Orders ({{ orders.filter((o) => o.channel === 'phone').length }})
            </button>
          </div>

          <div class="search-box">
            <input
              v-model="orderSearch"
              class="field-input search-input"
              type="search"
              placeholder="Search reference or product…"
            />
          </div>
        </div>

        <!-- Orders Listing -->
        <div v-if="filteredOrders.length === 0" class="empty-state">
          <p class="empty-text">No orders found matching this filter.</p>
          <button v-if="orderFilter !== 'all' || orderSearch" class="btn-secondary" type="button" @click="orderFilter = 'all'; orderSearch = ''">
            Clear Filters
          </button>
          <RouterLink v-else class="btn-secondary" to="/catalog">Browse Catalogue</RouterLink>
        </div>

        <div v-else class="orders-list">
          <article
            v-for="order in filteredOrders"
            :key="order.id"
            class="order-card"
          >
            <div class="order-card-header">
              <div class="order-ref-group">
                <span class="mono-code order-ref">{{ order.reference }}</span>
                <span class="channel-badge" :class="order.channel === 'phone' ? 'channel-badge--phone' : ''">
                  {{ order.channel === 'phone' ? '📞 Phone Order' : 'Online Order' }}
                </span>
                <span class="status-pill" :class="`status-pill--${order.status}`">
                  {{ order.status }}
                </span>
              </div>
              <span class="order-date">{{ formatDate(order.placed_at) }}</span>
            </div>

            <div class="order-card-body">
              <div class="order-items-col">
                <div v-for="item in order.items" :key="item.product_id" class="order-line-item">
                  <span class="item-qty">{{ item.quantity }} ×</span>
                  <span class="item-name">{{ item.name }}</span>
                  <span class="item-unit">({{ item.unit }})</span>
                  <span class="item-price">{{ formatNaira(item.unit_price_kobo * item.quantity) }}</span>
                </div>
              </div>

              <div class="order-dest-col">
                <span class="dest-label">Delivery Destination</span>
                <p class="dest-val">{{ order.delivery_address }}, {{ order.city }}</p>
                <p v-if="order.phone" class="dest-phone">Recipient phone: {{ order.phone }}</p>
              </div>
            </div>

            <div class="order-card-footer">
              <div class="order-total-block">
                <span class="total-label">Total Amount</span>
                <span class="total-amount">{{ formatNaira(order.total_kobo) }}</span>
                <span v-if="orderPromotionalSavingsKobo(order) > 0" class="order-promo-saved">
                  Saved {{ formatNaira(orderPromotionalSavingsKobo(order)) }}
                </span>
              </div>

              <div class="order-actions-group">
                <button class="btn-secondary btn-sm" type="button" @click="selectedOrder = order">
                  Details & Receipt
                </button>
                <RouterLink class="btn-secondary btn-sm" :to="`/track`">
                  Track Delivery
                </RouterLink>
                <button class="btn-secondary btn-sm" type="button" @click="reorder(order)">
                  Re-order All
                </button>
              </div>
            </div>
          </article>
        </div>

        <!-- Phone order sync note -->
        <div class="phone-sync-notice">
          <span class="notice-icon">💡</span>
          <p class="notice-text">
            <strong>Phone Order Sync:</strong> Orders placed over the phone with the Aba order desk are linked automatically
            to your customer account using your phone number (<strong>{{ user?.phone || 'on file' }}</strong>).
            Need to link past orders? Call <a class="inline-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>.
          </p>
        </div>
      </div>
    </div>

    <!-- TAB 3: SAVED DELIVERY ADDRESSES -->
    <div v-else-if="activeTab === 'addresses'" class="tab-pane">
      <div class="card addresses-pane-card">
        <div class="card-head">
          <div>
            <h2 class="card-title">Saved Delivery Addresses</h2>
            <p class="card-subtitle">
              Manage warehouses, retail stores, and job-site locations across Aba, Abia State, and Nigeria for fast dispatch.
            </p>
          </div>
          <button class="btn-primary" type="button" @click="openAddAddressModal">
            + Add New Address
          </button>
        </div>

        <div v-if="addresses.length === 0" class="empty-state">
          <p class="empty-text">No delivery addresses saved yet.</p>
          <button class="btn-primary" type="button" @click="openAddAddressModal">
            Add Your First Address
          </button>
        </div>

        <div v-else class="addresses-grid">
          <div
            v-for="addr in addresses"
            :key="addr.id"
            class="address-card"
            :class="{ 'address-card--default': addr.is_default }"
          >
            <div class="address-card-top">
              <div class="address-title-row">
                <h3 class="address-label">{{ addr.label }}</h3>
                <span v-if="addr.is_default" class="default-badge">DEFAULT</span>
              </div>
              <p class="address-recipient"><strong>{{ addr.recipient_name }}</strong></p>
              <p class="address-phone">{{ addr.phone }}</p>
            </div>

            <div class="address-card-body">
              <p class="address-street">{{ addr.street }}</p>
              <p v-if="addr.landmark" class="address-landmark">
                <span class="landmark-tag">Landmark:</span> {{ addr.landmark }}
              </p>
              <p class="address-city-state">{{ addr.city }}, {{ addr.state }}</p>
              <p v-if="addr.instructions" class="address-notes">
                <em>"{{ addr.instructions }}"</em>
              </p>
            </div>

            <div class="address-card-actions">
              <button
                v-if="!addr.is_default"
                class="address-btn-subtle"
                type="button"
                @click="setDefaultAddress(addr.id)"
              >
                Set as Default
              </button>
              <button class="address-btn-subtle" type="button" @click="openEditAddressModal(addr)">
                Edit
              </button>
              <button class="address-btn-subtle address-btn-subtle--danger" type="button" @click="deleteAddress(addr.id)">
                Delete
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 4: SAVED WISHLISTS -->
    <div v-else-if="activeTab === 'wishlists'" class="tab-pane">
      <div class="card wishlists-pane-card">
        <div class="card-head">
          <div>
            <h2 class="card-title">Saved Wishlists & Recurring Stock</h2>
            <p class="card-subtitle">
              Quick access to products you monitor or reorder regularly.
            </p>
          </div>
          <RouterLink class="btn-secondary" to="/wishlists">
            Manage Wishlists →
          </RouterLink>
        </div>

        <div v-if="wishlists.length === 0" class="empty-state">
          <p class="empty-text">You have no saved wishlists yet.</p>
          <RouterLink class="btn-primary" to="/catalog">Browse Catalogue</RouterLink>
        </div>

        <div v-else class="wishlists-grid">
          <div v-for="wl in wishlists" :key="wl.id" class="wishlist-summary-card">
            <div class="wl-card-head">
              <h3 class="wl-name">{{ wl.name }}</h3>
              <span class="wl-count-pill">{{ wl.item_count }} item{{ wl.item_count === 1 ? '' : 's' }}</span>
            </div>
            <p v-if="wl.description" class="wl-desc">{{ wl.description }}</p>

            <div class="wl-footer">
              <RouterLink class="btn-secondary btn-sm" :to="`/wishlists/${wl.id}`">
                Open List
              </RouterLink>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 5: PROFILE & PERSONAL DETAILS -->
    <div v-else-if="activeTab === 'profile'" class="tab-pane">
      <div class="dash-split-grid">
        <!-- Personal Details Edit Form -->
        <section class="card profile-card">
          <div class="card-head">
            <h2 class="card-title">Personal Details</h2>
            <button v-if="!editingProfile" class="btn-edit" type="button" @click="editingProfile = true; initProfileForm()">
              Edit
            </button>
          </div>

          <div v-if="profileSuccess" class="alert alert--success">Profile updated successfully.</div>
          <div v-if="profileError" class="alert alert--error">{{ profileError }}</div>

          <form v-if="editingProfile" class="edit-form" @submit.prevent="saveProfile">
            <div class="field-grid">
              <div class="field">
                <label class="field-label" for="edit_fname">First name</label>
                <input id="edit_fname" v-model="profileForm.first_name" class="field-input" type="text" required />
              </div>

              <div class="field">
                <label class="field-label" for="edit_lname">Last name</label>
                <input id="edit_lname" v-model="profileForm.last_name" class="field-input" type="text" required />
              </div>
            </div>

            <div class="field">
              <label class="field-label" for="edit_email">Email address</label>
              <input id="edit_email" v-model="profileForm.email" class="field-input" type="email" required />
            </div>

            <div class="field">
              <label class="field-label" for="edit_phone">Primary phone number</label>
              <input id="edit_phone" v-model="profileForm.phone" class="field-input" type="tel" placeholder="0800 000 0000" />
              <span class="field-hint">Primary number Aba warehouse drivers call before delivery.</span>
            </div>

            <div class="field-divider">
              <span class="eyebrow">Live Primary Delivery Address</span>
            </div>

            <div class="field">
              <label class="field-label" for="edit_street">Street address</label>
              <input id="edit_street" v-model="profileForm.street" class="field-input" type="text" placeholder="e.g. 14 Faulks Road" />
            </div>

            <div class="field">
              <label class="field-label" for="edit_landmark">Landmark / Directions</label>
              <input id="edit_landmark" v-model="profileForm.landmark" class="field-input" type="text" placeholder="e.g. Opposite Ariaria Market Gate 2" />
            </div>

            <div class="field-grid">
              <div class="field">
                <label class="field-label" for="edit_city">City</label>
                <input id="edit_city" v-model="profileForm.city" class="field-input" type="text" placeholder="Aba" />
              </div>
              <div class="field">
                <label class="field-label" for="edit_state">State</label>
                <input id="edit_state" v-model="profileForm.state" class="field-input" type="text" placeholder="Abia State" />
              </div>
            </div>

            <div class="field">
              <label class="field-label" for="edit_instructions">Offloading / delivery instructions</label>
              <textarea id="edit_instructions" v-model="profileForm.instructions" class="field-input" rows="2" placeholder="e.g. Call on arrival before offloading"></textarea>
            </div>

            <div class="form-actions">
              <button class="btn-secondary" type="button" :disabled="savingProfile" @click="editingProfile = false">
                Cancel
              </button>
              <button class="btn-primary" type="submit" :disabled="savingProfile">
                {{ savingProfile ? 'Saving…' : 'Save Changes' }}
              </button>
            </div>
          </form>

          <div v-else class="details-list">
            <div class="detail-row">
              <span class="detail-label">Full Name</span>
              <span class="detail-val">{{ user?.first_name }} {{ user?.last_name }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Email Address</span>
              <span class="detail-val">{{ user?.email }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Phone Number</span>
              <span class="detail-val">{{ user?.phone || 'Not provided' }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Customer ID</span>
              <span class="detail-val mono-code">{{ user?.id || '—' }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Primary Delivery Address</span>
              <div class="detail-val">
                <template v-if="defaultAddress">
                  <p class="live-addr-line"><strong>{{ defaultAddress.street }}</strong></p>
                  <p v-if="defaultAddress.landmark" class="sub-detail">Landmark: {{ defaultAddress.landmark }}</p>
                  <p class="sub-detail">{{ defaultAddress.city }}, {{ defaultAddress.state }}</p>
                  <p v-if="defaultAddress.instructions" class="sub-detail notes-italics">"{{ defaultAddress.instructions }}"</p>
                </template>
                <span v-else class="text-muted">No address set yet</span>
              </div>
            </div>
          </div>
        </section>

        <!-- Account Security / Channel Rules Overview -->
        <aside class="card side-summary-card">
          <h2 class="card-title">Order Channels & Security</h2>
          <p class="sidebar-copy">
            Your Imacals account provides unified tracking across both online and phone ordering channels.
          </p>

          <div class="rules-list">
            <div class="rule-box">
              <span class="rule-icon">📞</span>
              <div>
                <strong>Phone Orders Sync</strong>
                <p>Calling {{ SITE.orderLine }} automatically pulls your order history using your phone number.</p>
              </div>
            </div>
            <div class="rule-box">
              <span class="rule-icon">🏭</span>
              <div>
                <strong>Aba Distribution Warehouse</strong>
                <p>Dispatched directly from Factory Road, Aba. Same-day delivery inside Aba.</p>
              </div>
            </div>
            <div class="rule-box">
              <span class="rule-icon">🔒</span>
              <div>
                <strong>Data Privacy</strong>
                <p>Tenant-scoped security ensures your wishlists and addresses stay private.</p>
              </div>
            </div>
          </div>
        </aside>
      </div>
    </div>

    <!-- MODAL: ORDER DETAILS / RECEIPT -->
    <div v-if="selectedOrder" class="modal-scrim" @click.self="selectedOrder = null">
      <div class="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-order-title">
        <div class="modal-head">
          <div>
            <p class="eyebrow">{{ selectedOrder.channel === 'phone' ? 'Phone Order' : 'Online Checkout' }}</p>
            <h2 id="modal-order-title" class="modal-title">Order {{ selectedOrder.reference }}</h2>
          </div>
          <button class="modal-close" type="button" aria-label="Close modal" @click="selectedOrder = null">✕</button>
        </div>

        <div class="modal-body">
          <div class="modal-meta-row">
            <div>
              <span class="detail-label">Status</span>
              <span class="status-pill" :class="`status-pill--${selectedOrder.status}`">
                {{ selectedOrder.status }}
              </span>
            </div>
            <div>
              <span class="detail-label">Placed At</span>
              <span class="modal-meta-val">{{ formatDate(selectedOrder.placed_at) }}</span>
            </div>
            <div>
              <span class="detail-label">Recipient</span>
              <span class="modal-meta-val">{{ selectedOrder.customer_name }}</span>
            </div>
          </div>

          <!-- Items Table -->
          <div class="modal-items-table">
            <div class="table-head-row">
              <span>Item / Description</span>
              <span class="text-right">Qty</span>
              <span class="text-right">Unit Price</span>
              <span class="text-right">Total</span>
            </div>
            <div v-for="item in selectedOrder.items" :key="item.product_id" class="table-body-row">
              <div class="item-name-cell">
                <strong>{{ item.name }}</strong>
                <span class="item-unit-tag">Unit: {{ item.unit }}</span>
                <span v-if="item.original_unit_price_kobo && item.original_unit_price_kobo > item.unit_price_kobo" class="modal-promo-tag">
                  Promo deal
                </span>
              </div>
              <span class="text-right">{{ item.quantity }}</span>
              <span class="text-right mono-num">
                <template v-if="item.original_unit_price_kobo && item.original_unit_price_kobo > item.unit_price_kobo">
                  <span class="item-strikethrough">{{ formatNaira(item.original_unit_price_kobo) }}</span>
                  <br />
                </template>
                {{ formatNaira(item.unit_price_kobo) }}
              </span>
              <span class="text-right mono-num">{{ formatNaira(item.unit_price_kobo * item.quantity) }}</span>
            </div>

            <div class="table-summary-row">
              <span>Items Subtotal</span>
              <span class="text-right mono-num">
                {{ formatNaira(selectedOrder.items.reduce((acc, it) => acc + it.unit_price_kobo * it.quantity, 0)) }}
              </span>
            </div>
            <div v-if="orderPromotionalSavingsKobo(selectedOrder) > 0" class="table-summary-row promo-savings-row">
              <span>Promotional Savings</span>
              <span class="text-right mono-num promo-savings-val">-{{ formatNaira(orderPromotionalSavingsKobo(selectedOrder)) }}</span>
            </div>
            <div class="table-summary-row">
              <span>VAT (7.5% Nigerian Statutory)</span>
              <span class="text-right mono-num">
                {{ selectedOrder.tax_kobo != null && selectedOrder.tax_kobo > 0 ? formatNaira(selectedOrder.tax_kobo) : '₦0 (Exempt)' }}
              </span>
            </div>
            <div class="table-summary-row">
              <span>
                Delivery Fee ({{ selectedOrder.shipping_method === 'express' ? 'Priority Express Dispatch' : selectedOrder.shipping_method === 'pickup' ? 'Faulks Rd Depot Pickup' : 'Standard Road Dispatch' }})
              </span>
              <span class="text-right mono-num">
                {{ selectedOrder.delivery_fee_kobo > 0 ? formatNaira(selectedOrder.delivery_fee_kobo) : '₦0 (Free Wholesale)' }}
              </span>
            </div>
            <div class="table-summary-row table-summary-row--grand">
              <span>Grand Total</span>
              <span class="text-right mono-num grand-total">{{ formatNaira(selectedOrder.total_kobo) }}</span>
            </div>
          </div>

          <!-- Destination details -->
          <div class="modal-dest-box">
            <span class="detail-label">Delivery Destination & Instructions</span>
            <p><strong>{{ selectedOrder.delivery_address }}, {{ selectedOrder.city }}, {{ selectedOrder.state }}</strong></p>
            <p v-if="selectedOrder.phone">Contact Phone: {{ selectedOrder.phone }}</p>
            <p v-if="selectedOrder.note" class="modal-note"><em>Note: {{ selectedOrder.note }}</em></p>
          </div>

          <!-- Timeline -->
          <div v-if="selectedOrder.history && selectedOrder.history.length" class="modal-timeline">
            <span class="detail-label">Status Progression</span>
            <ol class="timeline-list">
              <li v-for="h in selectedOrder.history" :key="h.occurred_at" class="timeline-item">
                <span class="timeline-status">{{ h.status.toUpperCase() }}</span>
                <span class="timeline-time">{{ formatDate(h.occurred_at) }}</span>
                <p v-if="h.note" class="timeline-note">{{ h.note }}</p>
              </li>
            </ol>
          </div>
        </div>

        <div class="modal-actions">
          <button class="btn-secondary" type="button" @click="selectedOrder = null">Close</button>
          <button class="btn-primary" type="button" @click="reorder(selectedOrder); selectedOrder = null">
            Reorder These Items
          </button>
        </div>
      </div>
    </div>

    <!-- MODAL: ADD / EDIT ADDRESS -->
    <div v-if="addressModalOpen" class="modal-scrim" @click.self="closeAddressModal">
      <div class="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-addr-title">
        <div class="modal-head">
          <h2 id="modal-addr-title" class="modal-title">
            {{ editingAddressId ? 'Edit Delivery Address' : 'Add Delivery Address' }}
          </h2>
          <button class="modal-close" type="button" aria-label="Close modal" @click="closeAddressModal">✕</button>
        </div>

        <form class="modal-body modal-form" @submit.prevent="saveAddress">
          <div class="field">
            <label class="field-label" for="addr_label">Address Label</label>
            <input
              id="addr_label"
              v-model="addressForm.label"
              class="field-input"
              type="text"
              placeholder="e.g. Main Aba Store, Depot Annex, Home"
              required
            />
          </div>

          <div class="field-grid">
            <div class="field">
              <label class="field-label" for="addr_name">Recipient Name</label>
              <input
                id="addr_name"
                v-model="addressForm.recipient_name"
                class="field-input"
                type="text"
                required
              />
            </div>
            <div class="field">
              <label class="field-label" for="addr_phone">Receiving Phone Number</label>
              <input
                id="addr_phone"
                v-model="addressForm.phone"
                class="field-input"
                type="tel"
                placeholder="0800 000 0000"
                required
              />
            </div>
          </div>

          <div class="field">
            <label class="field-label" for="addr_street">Street Address</label>
            <input
              id="addr_street"
              v-model="addressForm.street"
              class="field-input"
              type="text"
              placeholder="e.g. 14 Faulks Road"
              required
            />
          </div>

          <div class="field">
            <label class="field-label" for="addr_landmark">Landmark / Directions (Crucial for drivers)</label>
            <input
              id="addr_landmark"
              v-model="addressForm.landmark"
              class="field-input"
              type="text"
              placeholder="e.g. Opposite Ariaria Market Gate 2, near Brass Junction"
            />
          </div>

          <div class="field-grid">
            <div class="field">
              <label class="field-label" for="addr_city">Town / City</label>
              <input
                id="addr_city"
                v-model="addressForm.city"
                class="field-input"
                type="text"
                required
              />
            </div>
            <div class="field">
              <label class="field-label" for="addr_state">State</label>
              <input
                id="addr_state"
                v-model="addressForm.state"
                class="field-input"
                type="text"
                required
              />
            </div>
          </div>

          <div class="field">
            <label class="field-label" for="addr_inst">Special Delivery Instructions (Optional)</label>
            <textarea
              id="addr_inst"
              v-model="addressForm.instructions"
              class="field-input"
              rows="2"
              placeholder="e.g. Offload at back bay, phone driver on entry"
            ></textarea>
          </div>

          <div class="checkbox-row">
            <label class="checkbox-label">
              <input v-model="addressForm.is_default" type="checkbox" />
              <span>Set as default delivery address</span>
            </label>
          </div>

          <div class="modal-actions">
            <button class="btn-secondary" type="button" @click="closeAddressModal">Cancel</button>
            <button class="btn-primary" type="submit">
              {{ editingAddressId ? 'Update Address' : 'Save Address' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<style scoped>
.customer-dashboard {
  padding-bottom: 64px;
}

/* Toast */
.toast-banner {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 1000;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  background-color: var(--color-surface);
  color: var(--color-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 12px 18px;
  box-shadow: 0 8px 24px var(--color-overlay);
  font-family: var(--font-label);
  font-size: 0.85rem;
}

.toast-indicator {
  color: var(--color-tertiary);
  font-size: 0.75rem;
}

/* Dashboard Head */
.dash-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--spacing-lg);
  margin-bottom: var(--spacing-lg);
  flex-wrap: wrap;
}

.customer-badge-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: 4px;
}

.hub-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  padding: 3px 8px;
  border-radius: var(--rounded-sm);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  color: var(--color-secondary);
}

.live-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: var(--color-tertiary);
}

.dash-sub {
  font-size: 0.85rem;
  color: var(--color-secondary);
  margin-top: 4px;
}

.mono-code {
  font-family: var(--font-label);
  letter-spacing: 0.02em;
  color: var(--color-primary);
}

.pipe-sep {
  margin: 0 var(--spacing-sm);
  color: var(--color-border);
}

.location-tag {
  color: var(--color-primary);
  font-weight: 500;
}

.dash-head-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  flex-wrap: wrap;
}

.phone-action-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 8px 14px;
  text-decoration: none;
  transition: border-color 0.15s, background-color 0.15s;
}

.phone-action-btn:hover {
  background-color: var(--color-neutral);
  border-color: var(--color-primary);
}

.phone-action-icon {
  font-size: 1.1rem;
}

.phone-action-text {
  display: flex;
  flex-direction: column;
}

.phone-action-label {
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.phone-action-number {
  font-family: var(--font-label);
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--color-primary);
}

/* Tabs */
.dash-tabs {
  display: flex;
  gap: 6px;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: var(--spacing-lg);
  overflow-x: auto;
  white-space: nowrap;
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--color-secondary);
  font-family: var(--font-label);
  font-size: 0.8rem;
  letter-spacing: 0.02em;
  padding: 12px 16px;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}

.tab-btn:hover {
  color: var(--color-primary);
}

.tab-btn--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-tertiary);
}

.tab-counter {
  display: inline-block;
  min-width: 18px;
  padding: 1px 6px;
  border-radius: 9px;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  font-size: 0.7rem;
  text-align: center;
}

/* Metric Cards */
.metrics-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

@media (max-width: 860px) {
  .metrics-grid { grid-template-columns: repeat(2, 1fr); }
}

@media (max-width: 480px) {
  .metrics-grid { grid-template-columns: 1fr; }
}

.metric-card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-md);
  cursor: pointer;
  transition: border-color 0.15s, transform 0.1s;
}

.metric-card:hover {
  border-color: var(--color-primary);
}

.metric-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.metric-label {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.metric-icon {
  font-size: 1rem;
}

.metric-value {
  font-family: var(--font-label);
  font-size: 1.75rem;
  font-weight: 600;
  color: var(--color-primary);
  line-height: 1.1;
  margin-bottom: 4px;
}

.metric-desc {
  font-size: 0.75rem;
  color: var(--color-secondary);
}

/* Spotlight Card */
.spotlight-card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
  margin-bottom: var(--spacing-lg);
}

.spotlight-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
  flex-wrap: wrap;
}

.spotlight-ref {
  font-family: var(--font-display);
  font-size: 1.5rem;
  font-weight: 500;
  margin-top: 2px;
}

.spotlight-status-col {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}

.spotlight-eta {
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
}

/* Stepper */
.stepper {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--spacing-lg);
  padding: 0 var(--spacing-sm);
}

.step-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  position: relative;
  z-index: 1;
}

.step-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-label);
  font-size: 0.75rem;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  color: var(--color-secondary);
  transition: all 0.2s;
}

.step-label {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  text-transform: uppercase;
}

.step-item--done .step-dot {
  background-color: var(--color-neutral);
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.step-item--active .step-dot {
  background-color: var(--color-tertiary);
  border-color: var(--color-tertiary);
  color: var(--color-on-primary);
}

.step-item--done .step-label,
.step-item--active .step-label {
  color: var(--color-primary);
  font-weight: 500;
}

.step-line {
  flex: 1;
  height: 2px;
  background-color: var(--color-border);
  margin: 0 4px;
  margin-bottom: 22px;
}

.step-line--done {
  background-color: var(--color-primary);
}

.spotlight-details {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-md);
  padding: var(--spacing-md) 0;
  border-top: 1px solid var(--color-divider);
  border-bottom: 1px solid var(--color-divider);
  margin-bottom: var(--spacing-md);
}

@media (max-width: 720px) {
  .spotlight-details { grid-template-columns: 1fr; }
}

.info-label {
  display: block;
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: 4px;
}

.info-val {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--color-primary);
}

.total-val {
  font-family: var(--font-label);
  font-weight: 600;
  font-size: 1.05rem;
}

.spotlight-actions {
  display: flex;
  gap: var(--spacing-sm);
  justify-content: flex-end;
}

/* Live Primary Address Card */
.live-address-card {
  margin-bottom: var(--spacing-lg);
  border-color: var(--color-border);
}

.live-addr-badge-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: 4px;
}

.live-address-details {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: var(--spacing-md);
  margin-top: var(--spacing-sm);
}

.live-address-grid {
  display: grid;
  grid-template-columns: 1fr 1.5fr 1fr;
  gap: var(--spacing-md);
}

@media (max-width: 768px) {
  .live-address-grid { grid-template-columns: 1fr; }
}

.live-col {
  display: flex;
  flex-direction: column;
}

.live-val {
  font-family: var(--font-body);
  font-size: 0.95rem;
  color: var(--color-primary);
}

.live-phone {
  font-size: 0.85rem;
  color: var(--color-secondary);
  margin-top: 2px;
}

.live-landmark {
  font-size: 0.8rem;
  color: var(--color-secondary);
  margin-top: 2px;
}

.live-city-state {
  font-size: 0.85rem;
  color: var(--color-primary);
  font-weight: 500;
  margin-top: 2px;
}

.live-instructions {
  font-size: 0.85rem;
  color: var(--color-secondary);
}

.live-address-empty {
  background-color: var(--color-neutral);
  border-radius: var(--rounded-md);
  padding: var(--spacing-md);
  margin-top: var(--spacing-sm);
}

.empty-note {
  font-size: 0.85rem;
  color: var(--color-secondary);
  line-height: 1.45;
}

/* Split layout */
.dash-split-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
  gap: var(--spacing-lg);
  align-items: start;
}

@media (max-width: 860px) {
  .dash-split-grid { grid-template-columns: 1fr; }
}

.card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
}

.card-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-md);
  gap: var(--spacing-md);
  flex-wrap: wrap;
}

.card-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 500;
}

.card-subtitle {
  font-size: 0.85rem;
  color: var(--color-secondary);
  margin-top: 2px;
}

.btn-link {
  background: none;
  border: none;
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  cursor: pointer;
  padding: 4px 0;
  border-bottom: 1px solid transparent;
}

.btn-link:hover {
  border-bottom-color: var(--color-primary);
}

/* Recent list */
.recent-list {
  display: flex;
  flex-direction: column;
}

.recent-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 0;
  border-bottom: 1px solid var(--color-divider);
  cursor: pointer;
  transition: background-color 0.1s;
}

.recent-row:last-child {
  border-bottom: none;
}

.recent-row:hover .recent-view-link {
  color: var(--color-primary);
}

.recent-meta-top {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: 4px;
}

.recent-items-summary {
  font-size: 0.85rem;
  color: var(--color-primary);
  margin-bottom: 2px;
}

.recent-date {
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.recent-side {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
}

.recent-price {
  font-family: var(--font-label);
  font-size: 0.95rem;
  font-weight: 500;
}

.recent-view-link {
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
  transition: color 0.15s;
}

/* Status pills */
.status-pill {
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  padding: 2px 7px;
  border-radius: var(--rounded-sm);
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  color: var(--color-secondary);
}

.status-pill--dispatched,
.status-pill--confirmed {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.status-pill--delivered {
  background-color: var(--color-surface);
  border-color: var(--color-tertiary);
  color: var(--color-tertiary);
}

.channel-badge {
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.02em;
  padding: 2px 6px;
  border-radius: var(--rounded-sm);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  color: var(--color-secondary);
}

.channel-badge--phone {
  border-color: var(--color-secondary);
  color: var(--color-primary);
}

/* Desk Card */
.desk-copy {
  font-size: 0.85rem;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-md);
  line-height: 1.45;
}

.desk-facts {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: var(--spacing-md);
}

.fact-item {
  display: flex;
  flex-direction: column;
}

.fact-label {
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.fact-val {
  font-size: 0.85rem;
  color: var(--color-primary);
}

.desk-phone-box {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 12px 14px;
  margin-bottom: var(--spacing-md);
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.desk-phone-label {
  font-family: var(--font-label);
  font-size: 0.7rem;
  color: var(--color-secondary);
}

.desk-phone-num {
  font-family: var(--font-label);
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-primary);
  text-decoration: none;
}

.desk-phone-note {
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.desk-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.action-btn-link {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: var(--font-label);
  font-size: 0.8rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  text-decoration: none;
  padding: 9px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  transition: border-color 0.15s, background-color 0.15s;
}

.action-btn-link:hover {
  background-color: var(--color-neutral);
  border-color: var(--color-primary);
}

/* Orders Pane Toolbar */
.orders-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
  flex-wrap: wrap;
}

.filter-pills {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.filter-pill {
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  padding: 6px 12px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
}

.filter-pill:hover {
  color: var(--color-primary);
  border-color: var(--color-secondary);
}

.filter-pill--active {
  background-color: var(--color-neutral);
  border-color: var(--color-primary);
  color: var(--color-primary);
  font-weight: 500;
}

.search-input {
  max-width: 260px;
}

/* Order Cards */
.orders-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.order-card {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: var(--spacing-md);
}

.order-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--color-divider);
  margin-bottom: 12px;
  flex-wrap: wrap;
  gap: var(--spacing-sm);
}

.order-ref-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.order-ref {
  font-size: 0.95rem;
  font-weight: 600;
}

.order-date {
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.order-card-body {
  display: grid;
  grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
  gap: var(--spacing-md);
  margin-bottom: 12px;
}

@media (max-width: 640px) {
  .order-card-body { grid-template-columns: 1fr; }
}

.order-items-col {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.order-line-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-size: 0.85rem;
}

.item-qty {
  font-family: var(--font-label);
  color: var(--color-secondary);
}

.item-name {
  color: var(--color-primary);
}

.item-unit {
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.item-price {
  margin-left: auto;
  font-family: var(--font-label);
  font-size: 0.8rem;
  color: var(--color-secondary);
}

.order-dest-col {
  display: flex;
  flex-direction: column;
}

.dest-label {
  font-family: var(--font-label);
  font-size: 0.65rem;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: 2px;
}

.dest-val {
  font-size: 0.85rem;
  color: var(--color-primary);
}

.dest-phone {
  font-size: 0.75rem;
  color: var(--color-secondary);
  margin-top: 2px;
}

.order-card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 10px;
  border-top: 1px solid var(--color-divider);
  flex-wrap: wrap;
  gap: var(--spacing-sm);
}

.order-total-block {
  display: flex;
  flex-direction: column;
}

.total-label {
  font-family: var(--font-label);
  font-size: 0.65rem;
  color: var(--color-secondary);
  text-transform: uppercase;
}

.total-amount {
  font-family: var(--font-label);
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--color-primary);
}

.order-promo-saved {
  font-family: var(--font-label);
  font-size: 0.7rem;
  font-weight: 600;
  color: #1a4d1a;
  background-color: var(--color-tertiary);
  padding: 1px 6px;
  border-radius: var(--rounded-sm);
  display: inline-block;
  margin-top: 2px;
  width: fit-content;
}

.order-actions-group {
  display: flex;
  gap: 6px;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 0.75rem;
}

.phone-sync-notice {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 12px 16px;
  display: flex;
  align-items: flex-start;
  gap: 12px;
  font-size: 0.85rem;
  color: var(--color-secondary);
  line-height: 1.45;
}

.notice-icon {
  font-size: 1.1rem;
}

.inline-link {
  color: var(--color-primary);
  text-decoration: underline;
}

/* Addresses Grid */
.addresses-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: var(--spacing-md);
}

.address-card {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: var(--spacing-md);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.address-card--default {
  border-color: var(--color-primary);
}

.address-title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.address-label {
  font-family: var(--font-display);
  font-size: 1.05rem;
  font-weight: 500;
}

.default-badge {
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.02em;
  background-color: var(--color-surface);
  border: 1px solid var(--color-primary);
  color: var(--color-primary);
  padding: 2px 6px;
  border-radius: var(--rounded-sm);
}

.address-recipient {
  font-size: 0.9rem;
}

.address-phone {
  font-family: var(--font-label);
  font-size: 0.8rem;
  color: var(--color-secondary);
  margin-bottom: 10px;
}

.address-card-body {
  margin-bottom: var(--spacing-md);
  font-size: 0.85rem;
  line-height: 1.4;
}

.address-landmark {
  color: var(--color-secondary);
  font-size: 0.8rem;
  margin-top: 2px;
}

.landmark-tag {
  font-family: var(--font-label);
  font-size: 0.7rem;
  text-transform: uppercase;
}

.address-notes {
  font-size: 0.8rem;
  color: var(--color-secondary);
  margin-top: 6px;
}

.address-card-actions {
  display: flex;
  gap: 8px;
  border-top: 1px solid var(--color-divider);
  padding-top: 10px;
}

.address-btn-subtle {
  background: none;
  border: none;
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
  cursor: pointer;
  padding: 4px 0;
  margin-right: 8px;
  transition: color 0.15s;
}

.address-btn-subtle:hover {
  color: var(--color-primary);
}

.address-btn-subtle--danger:hover {
  color: var(--color-tertiary);
}

/* Wishlists Grid */
.wishlists-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-md);
}

.wishlist-summary-card {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: var(--spacing-md);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.wl-card-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 6px;
}

.wl-name {
  font-family: var(--font-display);
  font-size: 1.1rem;
  font-weight: 500;
}

.wl-count-pill {
  font-family: var(--font-label);
  font-size: 0.7rem;
  color: var(--color-secondary);
}

.wl-desc {
  font-size: 0.85rem;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-md);
}

.wl-footer {
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid var(--color-divider);
  padding-top: 10px;
}

/* Profile Tab & Forms */
.btn-edit {
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  padding: 5px 12px;
  cursor: pointer;
}

.btn-edit:hover {
  border-color: var(--color-primary);
}

.details-list {
  display: flex;
  flex-direction: column;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: 12px 0;
  border-bottom: 1px solid var(--color-divider);
}

.detail-row:last-child {
  border-bottom: none;
}

.detail-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  text-transform: uppercase;
}

.detail-val {
  font-family: var(--font-body);
  font-size: 0.95rem;
  color: var(--color-primary);
}

.edit-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.field-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md);
}

@media (max-width: 540px) {
  .field-grid { grid-template-columns: 1fr; }
}

.field {
  display: flex;
  flex-direction: column;
}

.field-hint {
  font-size: 0.75rem;
  color: var(--color-secondary);
  margin-top: 4px;
}

.field-divider {
  border-top: 1px solid var(--color-divider);
  padding-top: var(--spacing-sm);
  margin-top: 4px;
}

.live-addr-line {
  margin-bottom: 2px;
}

.sub-detail {
  font-size: 0.85rem;
  color: var(--color-secondary);
  line-height: 1.4;
}

.notes-italics {
  font-style: italic;
  margin-top: 2px;
}

.text-muted {
  color: var(--color-secondary);
  font-size: 0.85rem;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-sm);
}

.alert {
  padding: 10px 14px;
  border-radius: var(--rounded-md);
  font-size: 0.85rem;
  margin-bottom: var(--spacing-md);
}

.alert--success {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  color: var(--color-primary);
}

.alert--error {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-tertiary);
  color: var(--color-tertiary);
}

.rules-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
  margin-top: var(--spacing-md);
}

.rule-box {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  font-size: 0.85rem;
  color: var(--color-secondary);
}

.rule-box strong {
  color: var(--color-primary);
  display: block;
  font-family: var(--font-label);
  font-size: 0.8rem;
  margin-bottom: 2px;
}

.rule-icon {
  font-size: 1.1rem;
}

/* Modals */
.modal-scrim {
  position: fixed;
  inset: 0;
  z-index: 999;
  background-color: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-md);
}

.modal-dialog {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  max-width: 640px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 12px 36px var(--color-overlay);
}

.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: var(--spacing-lg);
  border-bottom: 1px solid var(--color-divider);
}

.modal-title {
  font-family: var(--font-display);
  font-size: 1.35rem;
  font-weight: 500;
}

.modal-close {
  background: none;
  border: none;
  font-size: 1.1rem;
  color: var(--color-secondary);
  cursor: pointer;
  padding: 4px 8px;
}

.modal-close:hover {
  color: var(--color-primary);
}

.modal-body {
  padding: var(--spacing-lg);
}

.modal-meta-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-divider);
}

.modal-meta-val {
  font-size: 0.9rem;
  color: var(--color-primary);
  display: block;
}

.modal-items-table {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: var(--spacing-lg);
}

.table-head-row,
.table-body-row,
.table-summary-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1.5fr 1.5fr;
  align-items: center;
  font-size: 0.85rem;
  gap: var(--spacing-sm);
}

.table-head-row {
  font-family: var(--font-label);
  font-size: 0.7rem;
  text-transform: uppercase;
  color: var(--color-secondary);
  padding-bottom: 6px;
  border-bottom: 1px solid var(--color-divider);
}

.table-body-row {
  padding: 8px 0;
  border-bottom: 1px solid var(--color-divider);
}

.item-name-cell {
  display: flex;
  flex-direction: column;
}

.item-unit-tag {
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.text-right {
  text-align: right;
}

.mono-num {
  font-family: var(--font-label);
}

.table-summary-row {
  padding: 4px 0;
  color: var(--color-secondary);
}

.table-summary-row--grand {
  border-top: 1px solid var(--color-border);
  padding-top: 8px;
  color: var(--color-primary);
  font-weight: 600;
}

.grand-total {
  font-size: 1.05rem;
}

.modal-promo-tag {
  font-family: var(--font-label);
  font-size: 0.65rem;
  font-weight: 600;
  text-transform: uppercase;
  color: #1a4d1a;
  background-color: var(--color-tertiary);
  padding: 1px 5px;
  border-radius: var(--rounded-sm);
  width: fit-content;
  margin-top: 2px;
}

.item-strikethrough {
  text-decoration: line-through;
  color: var(--color-secondary);
  font-size: 0.75rem;
}

.promo-savings-row {
  color: #1a4d1a;
}

.promo-savings-val {
  color: #1a4d1a;
  font-weight: 600;
}

.modal-dest-box {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 12px 16px;
  margin-bottom: var(--spacing-lg);
  font-size: 0.85rem;
}

.modal-note {
  color: var(--color-secondary);
  margin-top: 4px;
}

.modal-timeline {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.timeline-list {
  list-style: none;
  padding-left: var(--spacing-md);
  border-left: 1px solid var(--color-border);
}

.timeline-item {
  padding: 8px 0;
}

.timeline-status {
  font-family: var(--font-label);
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-primary);
  margin-right: 8px;
}

.timeline-time {
  font-family: var(--font-label);
  font-size: 0.7rem;
  color: var(--color-secondary);
}

.timeline-note {
  font-size: 0.8rem;
  color: var(--color-secondary);
  margin-top: 2px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  padding: var(--spacing-lg);
  border-top: 1px solid var(--color-divider);
}

.checkbox-row {
  margin: var(--spacing-sm) 0;
}

.checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 0.85rem;
  color: var(--color-primary);
  cursor: pointer;
}

/* Empty State */
.empty-state {
  text-align: center;
  padding: var(--spacing-lg);
  background-color: var(--color-neutral);
  border-radius: var(--rounded-md);
  margin: var(--spacing-md) 0;
}

.empty-text {
  font-size: 0.9rem;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-md);
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
