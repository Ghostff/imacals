import { api } from '@/services/api';
import { usingPreviewCatalog } from '@/services/catalog';

// A line as the API expects it — product plus quantity. Prices are re-resolved server side so a
// tampered client cannot dictate what it pays.
export interface OrderLineInput {
  product_id: string;
  quantity: number;
}

export interface PlaceOrderInput {
  customer_name: string;
  phone: string;
  email?: string;
  delivery_address: string;
  city: string;
  state: string;
  // Free-text note the dispatch desk reads — landmarks, preferred delivery window.
  note?: string;
  lines: OrderLineInput[];
}

export interface PlacedOrder {
  id: string;
  reference: string;
  status: string;
  total_kobo: number;
  delivery_fee_kobo: number;
  placed_at: string;
}

export interface OrderStatusEvent {
  status: string;
  note: string | null;
  occurred_at: string;
}

export interface TrackedOrder {
  reference: string;
  status: string;
  placed_at: string;
  total_kobo: number;
  history: OrderStatusEvent[];
}

export const orderService = {
  async place(input: PlaceOrderInput): Promise<PlacedOrder> {
    if (usingPreviewCatalog) {
      throw new Error(
        'Checkout is not connected yet — the orders API has not been built. ' +
        'Call the order line on +234 000 000 0000 to place this order today.',
      );
    }
    return api.post<PlacedOrder>('/orders', input);
  },

  async track(reference: string): Promise<TrackedOrder> {
    if (usingPreviewCatalog) {
      throw new Error('Order tracking is not connected yet — the orders API has not been built.');
    }
    return api.get<TrackedOrder>(`/orders/${encodeURIComponent(reference)}/track`);
  },
};
