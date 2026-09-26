import { effectivePriceKobo, isTaxExempt, lineTaxKobo, type Product } from '@/services/catalog';

export type ShippingZone = 'aba_urban' | 'abia_regional' | 'south_east_near' | 'national';
export type ShippingMethod = 'standard' | 'express' | 'pickup';

export interface ShippingQuoteOption {
  id: ShippingMethod;
  name: string;
  fee_kobo: number;
  free_shipping_applied: boolean;
  description: string;
  estimated_delivery: string;
}

export interface OrderPricingBreakdown {
  items_subtotal_kobo: number;
  subtotal_kobo: number;
  original_items_subtotal_kobo: number;
  original_subtotal_kobo: number;
  promotional_savings_kobo: number;
  total_savings_kobo: number;
  taxable_subtotal_kobo: number;
  exempt_subtotal_kobo: number;
  tax_kobo: number;
  vat_rate_percent: number;
  shipping_zone: ShippingZone;
  shipping_zone_label: string;
  shipping_method: ShippingMethod;
  base_shipping_fee_kobo: number;
  shipping_fee_kobo: number;
  free_shipping_applied: boolean;
  is_free_shipping: boolean;
  free_shipping_threshold_kobo: number;
  amount_needed_for_free_shipping_kobo: number;
  grand_total_kobo: number;
  total_kobo: number;
  available_methods: ShippingQuoteOption[];
}

// Resolves dispatch zone from destination state and city in Nigeria.
export function resolveShippingZone(state: string, city: string): ShippingZone {
  const s = state.trim().toLowerCase();
  const c = city.trim().toLowerCase();

  if (s === 'abia' || s === 'abia state') {
    if (
      c.includes('aba')
      || c.includes('osisioma')
      || c.includes('ugwunagbo')
      || c.includes('obingwa')
      || c.includes('ariaria')
      || c.includes('faulks')
      || c.includes('ogbor')
    ) {
      return 'aba_urban';
    }
    return 'abia_regional';
  }

  const nearNeighbors = [
    'imo', 'imo state',
    'rivers', 'rivers state',
    'enugu', 'enugu state',
    'anambra', 'anambra state',
    'akwa ibom', 'akwa ibom state',
    'ebonyi', 'ebonyi state',
    'delta', 'delta state',
    'bayelsa', 'bayelsa state',
    'cross river', 'cross river state',
  ];

  if (nearNeighbors.includes(s)) {
    return 'south_east_near';
  }

  return 'national';
}

export function getShippingZoneLabel(zone: ShippingZone): string {
  switch (zone) {
    case 'aba_urban':
      return 'Aba Urban (Base Depot Area)';
    case 'abia_regional':
      return 'Abia State Regional (Umuahia / Ohafia / Arochukwu)';
    case 'south_east_near':
      return 'South-East / South-South Inter-state';
    case 'national':
      return 'National Nationwide Freight';
  }
}

export function getZoneBaseFeeKobo(zone: ShippingZone): number {
  switch (zone) {
    case 'aba_urban':
      return 250_000; // ₦2,500
    case 'abia_regional':
      return 400_000; // ₦4,000
    case 'south_east_near':
      return 650_000; // ₦6,500
    case 'national':
      return 1_000_000; // ₦10,000
  }
}

export function getZoneFreeThresholdKobo(zone: ShippingZone): number {
  switch (zone) {
    case 'aba_urban':
      return 30_000_000; // ₦300,000
    case 'abia_regional':
      return 40_000_000; // ₦400,000
    case 'south_east_near':
      return 50_000_000; // ₦500,000
    case 'national':
      return 75_000_000; // ₦750,000
  }
}

export function getAvailableShippingMethods(
  zone: ShippingZone,
  subtotalKobo: number,
): ShippingQuoteOption[] {
  const baseFee = getZoneBaseFeeKobo(zone);
  const threshold = getZoneFreeThresholdKobo(zone);
  const qualifiesForFree = subtotalKobo >= threshold;

  const standardFee = qualifiesForFree ? 0 : baseFee;
  const expressSurcharge = 250_000; // ₦2,500 priority handling fee
  const expressFee = qualifiesForFree ? expressSurcharge : baseFee + expressSurcharge;

  return [
    {
      id: 'standard',
      name: 'Standard Road Dispatch',
      fee_kobo: standardFee,
      free_shipping_applied: qualifiesForFree,
      description: 'Scheduled vehicle dispatch from Aba base warehouse.',
      estimated_delivery: zone === 'aba_urban' ? 'Same-day or next morning' : '1–3 business days',
    },
    {
      id: 'express',
      name: 'Priority Express Dispatch',
      fee_kobo: expressFee,
      free_shipping_applied: false,
      description: 'Dedicated priority loading and expedited vehicle routing.',
      estimated_delivery: zone === 'aba_urban' ? 'Within 3 hours' : 'Next-day priority delivery',
    },
    {
      id: 'pickup',
      name: 'Aba Warehouse Self-Pickup',
      fee_kobo: 0,
      free_shipping_applied: true,
      description: 'Collect ready pallet/cartons at Faulks Road central depot.',
      estimated_delivery: 'Ready in 1 hour during depot hours',
    },
  ];
}

// Complete order pricing calculation including effective discounts, 7.5% Nigerian VAT, and shipping.
export function computeOrderPricing(
  lines: Array<{ product: Product; quantity: number }>,
  state: string = 'Abia',
  city: string = 'Aba',
  method: ShippingMethod = 'standard',
): OrderPricingBreakdown {
  let items_subtotal_kobo = 0;
  let original_items_subtotal_kobo = 0;
  let taxable_subtotal_kobo = 0;
  let exempt_subtotal_kobo = 0;
  let tax_kobo = 0;

  for (const line of lines) {
    const qty = Math.max(0, line.quantity);
    const lineEffectivePrice = effectivePriceKobo(line.product);
    const lineEffectiveTotal = lineEffectivePrice * qty;
    const lineOriginalTotal = line.product.unit_price_kobo * qty;

    items_subtotal_kobo += lineEffectiveTotal;
    original_items_subtotal_kobo += lineOriginalTotal;

    if (isTaxExempt(line.product)) {
      exempt_subtotal_kobo += lineEffectiveTotal;
    } else {
      taxable_subtotal_kobo += lineEffectiveTotal;
      tax_kobo += lineTaxKobo(line.product, qty);
    }
  }

  const promotional_savings_kobo = Math.max(0, original_items_subtotal_kobo - items_subtotal_kobo);

  const shipping_zone = resolveShippingZone(state, city);
  const base_shipping_fee_kobo = getZoneBaseFeeKobo(shipping_zone);
  const free_shipping_threshold_kobo = getZoneFreeThresholdKobo(shipping_zone);
  const qualifiesForFree = items_subtotal_kobo >= free_shipping_threshold_kobo;

  let shipping_fee_kobo = 0;
  if (method === 'pickup') {
    shipping_fee_kobo = 0;
  } else if (method === 'standard') {
    shipping_fee_kobo = qualifiesForFree ? 0 : base_shipping_fee_kobo;
  } else if (method === 'express') {
    const expressSurcharge = 250_000;
    shipping_fee_kobo = qualifiesForFree ? expressSurcharge : base_shipping_fee_kobo + expressSurcharge;
  }

  const free_shipping_applied = qualifiesForFree && method !== 'pickup';
  const amount_needed_for_free_shipping_kobo = Math.max(0, free_shipping_threshold_kobo - items_subtotal_kobo);
  const grand_total_kobo = items_subtotal_kobo + tax_kobo + shipping_fee_kobo;
  const available_methods = getAvailableShippingMethods(shipping_zone, items_subtotal_kobo);

  return {
    items_subtotal_kobo,
    subtotal_kobo: items_subtotal_kobo,
    original_items_subtotal_kobo,
    original_subtotal_kobo: original_items_subtotal_kobo,
    promotional_savings_kobo,
    total_savings_kobo: promotional_savings_kobo,
    taxable_subtotal_kobo,
    exempt_subtotal_kobo,
    tax_kobo,
    vat_rate_percent: 7.5,
    shipping_zone,
    shipping_zone_label: getShippingZoneLabel(shipping_zone),
    shipping_method: method,
    base_shipping_fee_kobo,
    shipping_fee_kobo,
    free_shipping_applied,
    is_free_shipping: free_shipping_applied,
    free_shipping_threshold_kobo,
    amount_needed_for_free_shipping_kobo,
    grand_total_kobo,
    total_kobo: grand_total_kobo,
    available_methods,
  };
}
