use serde::{Deserialize, Serialize};

// Delivery dispatch zone relative to Aba, Abia State central warehouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShippingZone {
    AbaUrban,
    AbiaRegional,
    SouthEastNear,
    National,
}

impl ShippingZone {
    // Resolves dispatch zone from destination state and city in Nigeria.
    pub fn resolve(state: &str, city: &str) -> Self {
        let state_clean = state.trim().to_lowercase();
        let city_clean = city.trim().to_lowercase();

        if state_clean == "abia" || state_clean == "abia state" {
            if city_clean.contains("aba")
                || city_clean.contains("osisioma")
                || city_clean.contains("ugwunagbo")
                || city_clean.contains("obingwa")
                || city_clean.contains("ariaria")
                || city_clean.contains("faulks")
            {
                ShippingZone::AbaUrban
            } else {
                ShippingZone::AbiaRegional
            }
        } else if matches!(
            state_clean.as_str(),
            "imo"
                | "imo state"
                | "rivers"
                | "rivers state"
                | "enugu"
                | "enugu state"
                | "anambra"
                | "anambra state"
                | "akwa ibom"
                | "akwa ibom state"
                | "ebonyi"
                | "ebonyi state"
                | "delta"
                | "delta state"
                | "bayelsa"
                | "bayelsa state"
                | "cross river"
                | "cross river state"
        ) {
            ShippingZone::SouthEastNear
        } else {
            ShippingZone::National
        }
    }

    // Base shipping tariff in kobo before speed surcharge or free tier deductions.
    pub fn base_fee_kobo(&self) -> i64 {
        match self {
            ShippingZone::AbaUrban => 250_000,      // ₦2,500
            ShippingZone::AbiaRegional => 400_000,  // ₦4,000
            ShippingZone::SouthEastNear => 650_000, // ₦6,500
            ShippingZone::National => 1_000_000,    // ₦10,000
        }
    }

    // Order subtotal in kobo above which standard shipping is waived (free wholesale tier).
    pub fn free_shipping_threshold_kobo(&self) -> i64 {
        match self {
            ShippingZone::AbaUrban => 30_000_000,      // ₦300,000
            ShippingZone::AbiaRegional => 40_000_000,  // ₦400,000
            ShippingZone::SouthEastNear => 50_000_000, // ₦500,000
            ShippingZone::National => 75_000_000,    // ₦750,000
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShippingMethod {
    Standard,
    Express,
    WarehousePickup,
}

impl Default for ShippingMethod {
    fn default() -> Self {
        ShippingMethod::Standard
    }
}

// Line calculation input for pricing engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingLineInput {
    pub unit_price_kobo: i64,
    pub discount_price_kobo: Option<i64>,
    pub quantity: i32,
    pub is_tax_exempt: bool,
    pub tax_rate_basis_points: i32,
}

// Itemized tax calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxBreakdown {
    pub taxable_subtotal_kobo: i64,
    pub exempt_subtotal_kobo: i64,
    pub tax_kobo: i64,
    pub statutory_vat_rate_percent: f64,
}

// Complete order pricing summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPricingSummary {
    pub items_subtotal_kobo: i64,
    pub original_items_subtotal_kobo: i64,
    pub promotional_savings_kobo: i64,
    pub tax: TaxBreakdown,
    pub shipping_zone: ShippingZone,
    pub shipping_method: ShippingMethod,
    pub base_shipping_fee_kobo: i64,
    pub shipping_fee_kobo: i64,
    pub free_shipping_applied: bool,
    pub grand_total_kobo: i64,
}

pub struct OrderPricingCalculator;

impl OrderPricingCalculator {
    pub fn calculate(
        lines: &[PricingLineInput],
        state: &str,
        city: &str,
        method: ShippingMethod,
    ) -> OrderPricingSummary {
        let mut items_subtotal_kobo: i64 = 0;
        let mut original_items_subtotal_kobo: i64 = 0;
        let mut taxable_subtotal_kobo: i64 = 0;
        let mut exempt_subtotal_kobo: i64 = 0;
        let mut total_tax_kobo: i64 = 0;

        for line in lines {
            let effective_price = line.discount_price_kobo.unwrap_or(line.unit_price_kobo);
            let line_qty = line.quantity.max(0) as i64;
            let line_effective_total = effective_price * line_qty;
            let line_original_total = line.unit_price_kobo * line_qty;

            items_subtotal_kobo += line_effective_total;
            original_items_subtotal_kobo += line_original_total;

            if line.is_tax_exempt || line.tax_rate_basis_points <= 0 {
                exempt_subtotal_kobo += line_effective_total;
            } else {
                taxable_subtotal_kobo += line_effective_total;
                // Integer kobo calculation with rounding: (line_total * bps + 5000) / 10000
                let line_tax = (line_effective_total * (line.tax_rate_basis_points as i64) + 5000) / 10000;
                total_tax_kobo += line_tax;
            }
        }

        let promotional_savings_kobo = (original_items_subtotal_kobo - items_subtotal_kobo).max(0);

        let zone = ShippingZone::resolve(state, city);
        let base_fee = zone.base_fee_kobo();
        let qualifies_for_free = items_subtotal_kobo >= zone.free_shipping_threshold_kobo();

        let shipping_fee_kobo = match method {
            ShippingMethod::WarehousePickup => 0,
            ShippingMethod::Standard => {
                if qualifies_for_free {
                    0
                } else {
                    base_fee
                }
            }
            ShippingMethod::Express => {
                let express_surcharge = 250_000; // ₦2,500 priority dispatch fee
                if qualifies_for_free {
                    express_surcharge
                } else {
                    base_fee + express_surcharge
                }
            }
        };

        let free_shipping_applied = qualifies_for_free && method != ShippingMethod::WarehousePickup;
        let grand_total_kobo = items_subtotal_kobo + total_tax_kobo + shipping_fee_kobo;

        OrderPricingSummary {
            items_subtotal_kobo,
            original_items_subtotal_kobo,
            promotional_savings_kobo,
            tax: TaxBreakdown {
                taxable_subtotal_kobo,
                exempt_subtotal_kobo,
                tax_kobo: total_tax_kobo,
                statutory_vat_rate_percent: 7.5,
            },
            shipping_zone: zone,
            shipping_method: method,
            base_shipping_fee_kobo: base_fee,
            shipping_fee_kobo,
            free_shipping_applied,
            grand_total_kobo,
        }
    }
}
