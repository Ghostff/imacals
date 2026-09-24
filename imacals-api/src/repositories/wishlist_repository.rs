use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::models::product::CatalogProduct;
use crate::models::wishlist::{Wishlist, WishlistDetail, WishlistItem, WishlistItemWithProduct, WishlistSummary};

pub struct WishlistRepository;

// Flat row used by the JOIN that hydrates each wishlist item with its product snapshot.
#[derive(sqlx::FromRow)]
struct WishlistItemRow {
    pub id: Uuid,
    pub wishlist_id: Uuid,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub product_id_text: String,
    pub product_slug: String,
    pub product_name: String,
    pub product_description: String,
    pub product_unit: String,
    pub product_unit_price_kobo: i64,
    pub product_min_order_quantity: i32,
    pub product_in_stock: bool,
    pub product_image_url: Option<String>,
    pub category_slug: String,
    pub category_name: String,
}

impl From<WishlistItemRow> for WishlistItemWithProduct {
    fn from(r: WishlistItemRow) -> Self {
        Self {
            id: r.id,
            wishlist_id: r.wishlist_id,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
            product: CatalogProduct {
                id: r.product_id_text,
                slug: r.product_slug,
                name: r.product_name,
                description: r.product_description,
                category_slug: r.category_slug,
                category_name: r.category_name,
                unit: r.product_unit,
                unit_price_kobo: r.product_unit_price_kobo,
                min_order_quantity: r.product_min_order_quantity,
                in_stock: r.product_in_stock,
                image_url: r.product_image_url,
                images: Vec::new(),
            },
        }
    }
}

// Flat row used by the list query — Wishlist fields plus the live item count.
#[derive(sqlx::FromRow)]
struct WishlistSummaryRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub customer_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub item_count: i64,
}

impl From<WishlistSummaryRow> for WishlistSummary {
    fn from(r: WishlistSummaryRow) -> Self {
        Self {
            wishlist: Wishlist {
                id: r.id,
                organization_id: r.organization_id,
                customer_id: r.customer_id,
                name: r.name,
                description: r.description,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
            },
            item_count: r.item_count,
        }
    }
}

impl WishlistRepository {
    // Find an active wishlist by id.
    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Wishlist, Error> {
        sqlx::query_as!(
            Wishlist,
            r#"SELECT id, organization_id, customer_id, name, description,
                      created_by, created_at, updated_at, deleted_at
               FROM wishlists
               WHERE id = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            id
        )
        .fetch_one(pool)
        .await
    }

    // List active wishlists for a customer, newest first, with the live item count.
    pub async fn list_for_customer(
        pool: &PgPool,
        customer_id: &Uuid,
    ) -> Result<Vec<WishlistSummary>, Error> {
        let rows: Vec<WishlistSummaryRow> = sqlx::query_as!(
            WishlistSummaryRow,
            r#"SELECT w.id, w.organization_id, w.customer_id, w.name, w.description,
                      w.created_by, w.created_at, w.updated_at, w.deleted_at,
                      (SELECT COUNT(*) FROM wishlist_items wi
                       WHERE wi.wishlist_id = w.id AND wi.deleted_at IS NULL) AS "item_count!"
               FROM wishlists w
               WHERE w.customer_id = $1 AND w.deleted_at IS NULL
               ORDER BY w.created_at DESC"#,
            customer_id
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    // List every wishlist in an organization. Used by staff for an org-wide overview.
    pub async fn list_for_organization(
        pool: &PgPool,
        organization_id: &Uuid,
    ) -> Result<Vec<WishlistSummary>, Error> {
        let rows: Vec<WishlistSummaryRow> = sqlx::query_as!(
            WishlistSummaryRow,
            r#"SELECT w.id, w.organization_id, w.customer_id, w.name, w.description,
                      w.created_by, w.created_at, w.updated_at, w.deleted_at,
                      (SELECT COUNT(*) FROM wishlist_items wi
                       WHERE wi.wishlist_id = w.id AND wi.deleted_at IS NULL) AS "item_count!"
               FROM wishlists w
               WHERE w.organization_id = $1 AND w.deleted_at IS NULL
               ORDER BY w.created_at DESC"#,
            organization_id
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    // Return the wishlist plus its active items, each with the joined product snapshot.
    pub async fn find_detail(pool: &PgPool, id: &Uuid) -> Result<WishlistDetail, Error> {
        let wishlist = Self::find_by_id(pool, id).await?;
        let rows: Vec<WishlistItemRow> = sqlx::query_as!(
            WishlistItemRow,
            r#"SELECT
                  wi.id, wi.wishlist_id, wi.notes, wi.created_at, wi.updated_at,
                  p.id::text AS "product_id_text",
                  p.slug     AS "product_slug",
                  p.name     AS "product_name",
                  COALESCE(p.description, '') AS "product_description",
                  p.unit     AS "product_unit",
                  p.unit_price_kobo AS "product_unit_price_kobo",
                  p.min_order_quantity AS "product_min_order_quantity",
                  p.in_stock AS "product_in_stock",
                  f.absolute_path AS "product_image_url",
                  c.slug     AS "category_slug",
                  c.name     AS "category_name"
               FROM wishlist_items wi
               JOIN products p   ON p.id = wi.product_id AND p.deleted_at IS NULL
               JOIN categories c ON c.id = p.category_id AND c.deleted_at IS NULL
               LEFT JOIN LATERAL (
                   SELECT absolute_path FROM files
                   WHERE fileable_type = 'products'
                     AND fileable_id = p.id
                     AND type IN ('product-image', 'product-image-default')
                     AND deleted_at IS NULL
                   ORDER BY CASE WHEN type = 'product-image-default' THEN 0 ELSE 1 END, created_at ASC
                   LIMIT 1
               ) f ON true
               WHERE wi.wishlist_id = $1 AND wi.deleted_at IS NULL
               ORDER BY wi.created_at ASC"#,
            id
        )
        .fetch_all(pool)
        .await?;
        Ok(WishlistDetail {
            wishlist,
            items: rows.into_iter().map(Into::into).collect(),
        })
    }

    // Create a wishlist for a customer.
    pub async fn create(
        pool: &PgPool,
        organization_id: &Uuid,
        customer_id: &Uuid,
        name: &str,
        description: Option<&str>,
        created_by: Option<&Uuid>,
    ) -> Result<Wishlist, Error> {
        sqlx::query_as!(
            Wishlist,
            r#"INSERT INTO wishlists
                   (organization_id, customer_id, name, description, created_by)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, organization_id, customer_id, name, description,
                         created_by, created_at, updated_at, deleted_at"#,
            organization_id,
            customer_id,
            name,
            description,
            created_by
        )
        .fetch_one(pool)
        .await
    }

    // Update a wishlist's name/description.
    pub async fn update(pool: &PgPool, wishlist: &Wishlist) -> Result<Wishlist, Error> {
        sqlx::query_as!(
            Wishlist,
            r#"UPDATE wishlists
               SET name        = $2,
                   description = $3,
                   updated_at  = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, organization_id, customer_id, name, description,
                         created_by, created_at, updated_at, deleted_at"#,
            wishlist.id,
            wishlist.name,
            wishlist.description
        )
        .fetch_one(pool)
        .await
    }

    // Soft-delete the wishlist. The cascade trigger soft-deletes its items.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE wishlists SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(pool)
        .await?
        .rows_affected())
    }

    // Add a product to a wishlist. ON CONFLICT DO NOTHING so adding the same product twice is a
    // no-op (the unique partial index would reject it otherwise).
    pub async fn add_item(
        pool: &PgPool,
        wishlist_id: &Uuid,
        product_id: &Uuid,
        notes: Option<&str>,
    ) -> Result<WishlistItem, Error> {
        sqlx::query_as!(
            WishlistItem,
            r#"INSERT INTO wishlist_items (wishlist_id, product_id, notes)
               VALUES ($1, $2, $3)
               ON CONFLICT (wishlist_id, product_id) WHERE deleted_at IS NULL DO UPDATE
                   SET notes = EXCLUDED.notes, updated_at = NOW()
               RETURNING id, wishlist_id, product_id, notes, created_at, updated_at, deleted_at"#,
            wishlist_id,
            product_id,
            notes
        )
        .fetch_one(pool)
        .await
    }

    // Remove a single item from a wishlist. Returns the number of rows affected so the
    // controller can distinguish a missing item from a soft-deleted one.
    pub async fn remove_item(
        pool: &PgPool,
        wishlist_id: &Uuid,
        item_id: &Uuid,
    ) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE wishlist_items
                SET deleted_at = NOW()
              WHERE id = $1 AND wishlist_id = $2 AND deleted_at IS NULL",
            item_id,
            wishlist_id
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn make_org_and_user(pool: &PgPool) -> (Uuid, Uuid) {
        let org_id = sqlx::query_scalar!(
            "INSERT INTO organizations (name, slug) VALUES ('Imacals', 'imacals') RETURNING id"
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T','w@test.com','x',NOW()) RETURNING id"
        )
        .fetch_one(pool)
        .await
        .unwrap();
        (org_id, user_id)
    }

    async fn make_customer(pool: &PgPool, org_id: &Uuid) -> Uuid {
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('C','C','c@test.com','x',NOW()) RETURNING id"
        )
        .fetch_one(pool)
        .await
        .unwrap();
        crate::repositories::customer_repository::CustomerRepository::create(
            pool, org_id, "Wishlist Owner", Some("08030000000"), None, None, &user_id,
        )
        .await
        .unwrap()
        .id
    }

    // A new wishlist should be listable for its customer.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn created_wishlist_appears_in_customer_list(pool: PgPool) {
        let (org_id, _user_id) = make_org_and_user(&pool).await;
        let customer_id = make_customer(&pool, &org_id).await;
        WishlistRepository::create(&pool, &org_id, &customer_id, "Reorder", None, None)
            .await
            .unwrap();
        let list = WishlistRepository::list_for_customer(&pool, &customer_id).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].wishlist.name, "Reorder");
        assert_eq!(list[0].item_count, 0);
    }

    // Soft-deleting a wishlist removes it from the customer list.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleted_wishlist_is_hidden(pool: PgPool) {
        let (org_id, _user_id) = make_org_and_user(&pool).await;
        let customer_id = make_customer(&pool, &org_id).await;
        let w = WishlistRepository::create(&pool, &org_id, &customer_id, "Gone", None, None)
            .await
            .unwrap();
        WishlistRepository::delete(&pool, &w.id).await.unwrap();
        let list = WishlistRepository::list_for_customer(&pool, &customer_id).await.unwrap();
        assert!(list.is_empty());
    }
}
