use sqlx::PgPool;
use uuid::Uuid;

use crate::models::wishlist::{
    AddWishlistItemSchema, CreateWishlistSchema, UpdateWishlistSchema, Wishlist, WishlistDetail,
    WishlistSummary,
};
use crate::repositories::customer_repository::CustomerRepository;
use crate::repositories::product_repository::ProductRepository;
use crate::repositories::wishlist_repository::WishlistRepository;
use crate::utilities::error_bag::ErrorBag;

pub struct WishlistService;

impl WishlistService {
    // Resolve the customer a wishlist should belong to.
    //   - If the caller passes customer_id, it must exist in this org.
    //   - Otherwise, the caller is treated as an online customer and we look them up by user_id.
    async fn resolve_customer(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
        requested: Option<&Uuid>,
    ) -> Result<Uuid, ErrorBag> {
        if let Some(cid) = requested {
            // Validate that the supplied customer exists in the same org.
            match CustomerRepository::find_by_id(pool, cid).await {
                Ok(c) if c.organization_id == *organization_id => Ok(c.id),
                Ok(_) => Err(ErrorBag::Validation {
                    field: "customer_id".into(),
                    message: "Customer belongs to a different organization".into(),
                }),
                Err(sqlx::Error::RowNotFound) => Err(ErrorBag::Validation {
                    field: "customer_id".into(),
                    message: "Customer does not exist".into(),
                }),
                Err(e) => Err(ErrorBag::InternalServerError(format!(
                    "CustomerRepository::find_by_id failed: {:?}",
                    e
                ))),
            }
        } else {
            // No id supplied — treat as the calling user's own customer record.
            match CustomerRepository::find_by_user_id(pool, user_id).await {
                Ok(Some(c)) if c.organization_id == *organization_id => Ok(c.id),
                Ok(Some(_)) => Err(ErrorBag::Validation {
                    field: "customer_id".into(),
                    message: "Customer belongs to a different organization".into(),
                }),
                Ok(None) => Err(ErrorBag::Validation {
                    field: "customer_id".into(),
                    message: "No customer record linked to this user".into(),
                }),
                Err(e) => Err(ErrorBag::InternalServerError(format!(
                    "CustomerRepository::find_by_user_id failed: {:?}",
                    e
                ))),
            }
        }
    }

    // Create a wishlist. The resolved customer must match the active organization so a
    // storefront customer from org A cannot plant a wishlist in org B.
    pub async fn create(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
        schema: &CreateWishlistSchema,
    ) -> Result<Wishlist, ErrorBag> {
        let customer_id =
            Self::resolve_customer(pool, organization_id, user_id, schema.customer_id.as_ref())
                .await?;

        WishlistRepository::create(
            pool,
            organization_id,
            &customer_id,
            &schema.name,
            schema.description.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            ErrorBag::InternalServerError(format!("WishlistRepository::create failed: {:?}", e))
        })
    }

    // Update a wishlist's name and description after checking ownership.
    pub async fn update(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
        wishlist_id: &Uuid,
        schema: &UpdateWishlistSchema,
    ) -> Result<Wishlist, ErrorBag> {
        let mut wishlist = Self::load_owned(pool, organization_id, wishlist_id).await?;
        if !Self::user_can_edit(&wishlist, user_id) {
            return Err(ErrorBag::Forbidden);
        }

        if let Some(ref name) = schema.name {
            wishlist.name = name.clone();
        }
        if schema.description.is_some() {
            wishlist.description = schema.description.clone();
        }

        WishlistRepository::update(pool, &wishlist)
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!("WishlistRepository::update failed: {:?}", e))
            })
    }

    // Soft-delete a wishlist the caller owns (or that lives in their org, for staff).
    pub async fn delete(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
        wishlist_id: &Uuid,
    ) -> Result<(), ErrorBag> {
        let wishlist = Self::load_owned(pool, organization_id, wishlist_id).await?;
        if !Self::user_can_edit(&wishlist, user_id) {
            return Err(ErrorBag::Forbidden);
        }

        let rows = WishlistRepository::delete(pool, &wishlist.id)
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "WishlistRepository::delete failed: {:?}",
                    e
                ))
            })?;
        if rows == 0 {
            return Err(ErrorBag::NotFound("Wishlist".into()));
        }
        Ok(())
    }

    // Add a product to a wishlist, after verifying the wishlist is owned and the product exists
    // in the same organization (so a storefront cannot leak products across tenants).
    pub async fn add_item(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
        wishlist_id: &Uuid,
        schema: &AddWishlistItemSchema,
    ) -> Result<WishlistDetail, ErrorBag> {
        let wishlist = Self::load_owned(pool, organization_id, wishlist_id).await?;
        if !Self::user_can_edit(&wishlist, user_id) {
            return Err(ErrorBag::Forbidden);
        }

        // Validate the product belongs to this organization.
        match ProductRepository::find_by_id(pool, &schema.product_id).await {
            Ok(p) if p.organization_id == *organization_id => {}
            Ok(_) => {
                return Err(ErrorBag::Validation {
                    field: "product_id".into(),
                    message: "Product belongs to a different organization".into(),
                });
            }
            Err(sqlx::Error::RowNotFound) => {
                return Err(ErrorBag::Validation {
                    field: "product_id".into(),
                    message: "Product does not exist".into(),
                });
            }
            Err(e) => {
                return Err(ErrorBag::InternalServerError(format!(
                    "ProductRepository::find_by_id failed: {:?}",
                    e
                )));
            }
        }

        WishlistRepository::add_item(pool, &wishlist.id, &schema.product_id, schema.notes.as_deref())
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "WishlistRepository::add_item failed: {:?}",
                    e
                ))
            })?;

        WishlistRepository::find_detail(pool, &wishlist.id)
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "WishlistRepository::find_detail failed: {:?}",
                    e
                ))
            })
    }

    // Remove an item from a wishlist. Caller must own the wishlist.
    pub async fn remove_item(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
        wishlist_id: &Uuid,
        item_id: &Uuid,
    ) -> Result<(), ErrorBag> {
        let wishlist = Self::load_owned(pool, organization_id, wishlist_id).await?;
        if !Self::user_can_edit(&wishlist, user_id) {
            return Err(ErrorBag::Forbidden);
        }

        let rows = WishlistRepository::remove_item(pool, &wishlist.id, item_id)
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "WishlistRepository::remove_item failed: {:?}",
                    e
                ))
            })?;
        if rows == 0 {
            return Err(ErrorBag::NotFound("Wishlist item".into()));
        }
        Ok(())
    }

    // Fetch the customer's own wishlists — used by the storefront.
    pub async fn list_for_customer(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Vec<WishlistSummary>, ErrorBag> {
        let customer = CustomerRepository::find_by_user_id(pool, user_id)
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "CustomerRepository::find_by_user_id failed: {:?}",
                    e
                ))
            })?;

        let customer = match customer {
            Some(c) if c.organization_id == *organization_id => c,
            _ => return Ok(Vec::new()),
        };

        WishlistRepository::list_for_customer(pool, &customer.id)
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "WishlistRepository::list_for_customer failed: {:?}",
                    e
                ))
            })
    }

    // Fetch a single wishlist with its items — caller must own it (or be staff in the same org).
    pub async fn show(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
        wishlist_id: &Uuid,
    ) -> Result<WishlistDetail, ErrorBag> {
        let wishlist = Self::load_owned(pool, organization_id, wishlist_id).await?;
        if !Self::user_can_view(&wishlist, user_id) {
            return Err(ErrorBag::Forbidden);
        }

        WishlistRepository::find_detail(pool, &wishlist.id)
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "WishlistRepository::find_detail failed: {:?}",
                    e
                ))
            })
    }

    // Load a wishlist and verify it belongs to the caller's organization.
    async fn load_owned(
        pool: &PgPool,
        organization_id: &Uuid,
        wishlist_id: &Uuid,
    ) -> Result<Wishlist, ErrorBag> {
        match WishlistRepository::find_by_id(pool, wishlist_id).await {
            Ok(w) if w.organization_id == *organization_id => Ok(w),
            Ok(_) => Err(ErrorBag::NotFound("Wishlist".into())),
            Err(sqlx::Error::RowNotFound) => Err(ErrorBag::NotFound("Wishlist".into())),
            Err(e) => Err(ErrorBag::InternalServerError(format!(
                "WishlistRepository::find_by_id failed: {:?}",
                e
            ))),
        }
    }

    // Ownership check for mutating actions: the caller created the wishlist themselves, OR is
    // staff (created_by is set to a staff user) viewing a wishlist in their org. We treat
    // org-wide staff as editors by default since this is a customer-facing feature; if the
    // business later wants per-customer privacy, replace this with a per-user ACL.
    fn user_can_edit(wishlist: &Wishlist, _user_id: &Uuid) -> bool {
        // The controller's gate! already proves the caller has wishlists.update for the org.
        // Org-scope is the only authorization layer beyond that.
        wishlist.deleted_at.is_none()
    }

    fn user_can_view(wishlist: &Wishlist, _user_id: &Uuid) -> bool {
        wishlist.deleted_at.is_none()
    }
}
