import { ref, computed, type Ref, type ComputedRef } from 'vue';
import type { Product } from '@/services/catalog';

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
  add: (product: Product, quantity?: number) => void;
  setQuantity: (productId: string, quantity: number) => void;
  remove: (productId: string) => void;
  clear: () => void;
} {
  const itemCount: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) => sum + l.quantity, 0),
  );

  const subtotalKobo: ComputedRef<number> = computed<number>(() =>
    lines.value.reduce((sum, l) => sum + l.product.unit_price_kobo * l.quantity, 0),
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

  return { lines, itemCount, subtotalKobo, add, setQuantity, remove, clear };
}
