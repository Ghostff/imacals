use actix_web::{web, HttpResponse};
use actix_web::web::{Data, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::repositories::product_repository::ProductRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;
use crate::utilities::order_pricing::{
    OrderPricingCalculator, OrderPricingSummary, PricingLineInput, ShippingMethod, ShippingZone,
};

#[derive(Debug, Deserialize, Validate)]
pub struct CalculateOrderItemPayload {
    pub product_id: Uuid,
    #[validate(range(min = 1, message = "Quantity must be at least 1"))]
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CalculateOrderPricingPayload {
    #[validate(length(min = 1, message = "State must be specified"))]
    pub state: String,
    #[validate(length(min = 1, message = "City must be specified"))]
    pub city: String,
    pub shipping_method: Option<ShippingMethod>,
    pub items: Vec<CalculateOrderItemPayload>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ShippingQuotePayload {
    pub state: String,
    pub city: String,
    pub subtotal_kobo: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ShippingMethodQuote {
    pub method: ShippingMethod,
    pub name: String,
    pub fee_kobo: i64,
    pub free_shipping_applied: bool,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ShippingQuoteResponse {
    pub zone: ShippingZone,
    pub base_fee_kobo: i64,
    pub free_shipping_threshold_kobo: i64,
    pub methods: Vec<ShippingMethodQuote>,
}

// Computes complete pricing, tax (7.5% Nigerian VAT), and delivery tariffs from server-verified products.
pub async fn calculate(
    app: Data<AppState>,
    payload: Json<CalculateOrderPricingPayload>,
) -> HttpResponse {
    if let Err(e) = payload.validate() {
        return JsonResponse::error(ErrorBag::Validation {
            field: "pricing".into(),
            message: format!("Invalid pricing request: {:?}", e),
        });
    }

    let method = payload.shipping_method.unwrap_or_default();
    let mut pricing_lines: Vec<PricingLineInput> = Vec::with_capacity(payload.items.len());

    for item in &payload.items {
        match ProductRepository::find_by_id(&app.pool, &item.product_id).await {
            Ok(product) => {
                pricing_lines.push(PricingLineInput {
                    unit_price_kobo: product.unit_price_kobo,
                    discount_price_kobo: product.discount_price_kobo,
                    quantity: item.quantity,
                    is_tax_exempt: product.is_tax_exempt,
                    tax_rate_basis_points: product.tax_rate_basis_points,
                });
            }
            Err(sqlx::Error::RowNotFound) => {
                return JsonResponse::error(ErrorBag::NotFound(format!("Product {}", item.product_id)));
            }
            Err(e) => {
                return JsonResponse::fatal(e, "pricing_controller.calculate product lookup failed");
            }
        }
    }

    let summary: OrderPricingSummary = OrderPricingCalculator::calculate(
        &pricing_lines,
        &payload.state,
        &payload.city,
        method,
    );

    JsonResponse::success(summary)
}

// Returns available shipping methods and tariffs for a destination.
pub async fn shipping_quote(payload: Json<ShippingQuotePayload>) -> HttpResponse {
    let zone = ShippingZone::resolve(&payload.state, &payload.city);
    let subtotal = payload.subtotal_kobo.unwrap_or(0);
    let qualifies_for_free = subtotal >= zone.free_shipping_threshold_kobo();

    let standard_fee = if qualifies_for_free { 0 } else { zone.base_fee_kobo() };
    let express_surcharge = 250_000;
    let express_fee = if qualifies_for_free { express_surcharge } else { zone.base_fee_kobo() + express_surcharge };

    let methods = vec![
        ShippingMethodQuote {
            method: ShippingMethod::Standard,
            name: "Standard Road Dispatch".into(),
            fee_kobo: standard_fee,
            free_shipping_applied: qualifies_for_free,
            description: "Direct dispatch from Aba central depot to destination address".into(),
        },
        ShippingMethodQuote {
            method: ShippingMethod::Express,
            name: "Priority Express Dispatch".into(),
            fee_kobo: express_fee,
            free_shipping_applied: false,
            description: "Same-day priority loading and expedited vehicle dispatch".into(),
        },
        ShippingMethodQuote {
            method: ShippingMethod::WarehousePickup,
            name: "Warehouse Depot Pickup".into(),
            fee_kobo: 0,
            free_shipping_applied: true,
            description: "Self-pickup at Aba base warehouse (Faulks / Ariaria axis), free of charge".into(),
        },
    ];

    JsonResponse::success(ShippingQuoteResponse {
        zone,
        base_fee_kobo: zone.base_fee_kobo(),
        free_shipping_threshold_kobo: zone.free_shipping_threshold_kobo(),
        methods,
    })
}
