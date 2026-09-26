import { ref, computed, type Ref, type ComputedRef } from 'vue';
import {
  effectivePriceKobo,
  discountSavingsKobo,
  lineTaxKobo,
  isTaxExempt,
  type Product,
} from '@/services/catalog';

export interface CartLine {
  product: Product;
  quantity: number;
}

const STORAGE_KEY: string = 'cart';

// Module-level singleton: the header badge and the cart page read one source of truth.
const lines: Ref<CartLine[]> = ref([]);

function persist(): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(lines.value));
}

// Call once at boot. A malformed or hand-edited entry must not break the whole app, so a bad
// parse resets the cart rather than throwing on mount.
export function initCart(): void {
  const raw: string | null = localStorage.getItem(STORAGE_KEY);
  if (!raw) return;
  try {
    const parsed: unknown = JSON.parse(raw);
    lines.value = Array.isArray(parsed) ? (parsed as CartLine[]) : [];
  } catch {
    lines.value = [];
  }
}

export function useCart(): {
  lines: Ref<CartLine[]>;
  itemCount: ComputedRef<number>;
  subtotalKobo: ComputedRef<number>;
  originalSubtotalKobo: ComputedRef<number>;
  totalSavingsKobo: ComputedRef<number>;
  taxKobo: ComputedRef<number>;
  taxableSubtotalKobo: ComputedRef<number>;
  exemptSubtotalKobo: ComputedRef<number>;
  add: (product: Product, quantity?: number) => void;
  setQuantity: (productId: string, quantity: number) => void;
  remove: (productId: string) => void;
  clear: () => void;
} {
  const itemCount: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) => sum + l.quantity, 0),
  );

  // Subtotal is always summed over the effective selling price (promotional discount price if set).
  const subtotalKobo: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) => sum + effectivePriceKobo(l.product) * l.quantity, 0),
  );

  const originalSubtotalKobo: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) => sum + l.product.unit_price_kobo * l.quantity, 0),
  );

  const totalSavingsKobo: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) => sum + discountSavingsKobo(l.product) * l.quantity, 0),
  );

  // Statutory Nigerian VAT (7.5%) applied to taxable non-exempt lines.
  const taxKobo: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) => sum + lineTaxKobo(l.product, l.quantity), 0),
  );

  const taxableSubtotalKobo: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) =>
      isTaxExempt(l.product) ? sum : sum + effectivePriceKobo(l.product) * l.quantity,
    0),
  );

  const exemptSubtotalKobo: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) =>
      isTaxExempt(l.product) ? sum + effectivePriceKobo(l.product) * l.quantity : sum,
    0),
  );

  // Adding an item already in the cart tops up its quantity instead of duplicating the line.
  function add(product: Product, quantity: number = product.min_order_quantity): void {
    const existing = lines.value.find((l) => l.product.id === product.id);
    if (existing) {
      existing.quantity += quantity;
    } else {
      lines.value = [...lines.value, { product, quantity }];
    }
    persist();
  }

  // Quantity is clamped to the product's minimum — dropping below it removes the line entirely,
  // because a wholesale line below its MOQ is not an order the warehouse can pick.
  function setQuantity(productId: string, quantity: number): void {
    const line = lines.value.find((l) => l.product.id === productId);
    if (!line) return;
    if (quantity < line.product.min_order_quantity) {
      remove(productId);
      return;
    }
    line.quantity = quantity;
    persist();
  }

  function remove(productId: string): void {
    lines.value = lines.value.filter((l) => l.product.id !== productId);
    persist();
  }

  function clear(): void {
    lines.value = [];
    persist();
  }

  return {
    lines,
    itemCount,
    subtotalKobo,
    originalSubtotalKobo,
    totalSavingsKobo,
    taxKobo,
    taxableSubtotalKobo,
    exemptSubtotalKobo,
    add,
    setQuantity,
    remove,
    clear,
  };
}
