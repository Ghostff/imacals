// Storefront contact and coverage details. Placeholders until the real numbers are supplied —
// these strings are customer-facing, so replace them before the site goes live.
export const SITE = {
  name: 'Imacals',
  tagline: 'Order online or by phone. We deliver.',

  // Base distribution warehouse. Every delivery route starts here.
  warehouse: {
    line1: 'Imacals Distribution Warehouse',
    city: 'Aba',
    state: 'Abia State',
    country: 'Nigeria',
  },

  // The phone-order desk. Half the order book comes in by voice, so this must be reachable from
  // every page, not buried in a contact form.
  orderLine: '+234 000 000 0000',
  // tel: href form — digits only, no spaces.
  orderLineHref: 'tel:+2340000000000',
  whatsapp: '+234 000 000 0000',

  hours: 'Monday – Saturday, 8:00am – 6:00pm',

  // Delivery coverage, nearest first. Same-day applies inside Aba only.
  coverage: [
    { area: 'Aba metropolis', eta: 'Same day' },
    { area: 'Abia State (outside Aba)', eta: '1 working day' },
    { area: 'South-East & South-South', eta: '1 – 2 working days' },
    { area: 'Rest of Nigeria', eta: '2 – 4 working days' },
  ],
} as const;
