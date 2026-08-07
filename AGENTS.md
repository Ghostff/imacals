# imacals — AI Generation Guide

Ecommerce and distribution for Nigeria, run out of a base warehouse in **Aba, Abia State**.
Customers order **online or by phone**; we pick, deliver and distribute. Both channels produce the
same order record — never build for one and not the other.

AI-only project. Read this file before writing any code.

**Always read first:**
- `docs/business_logic.md` — entities, roles, rules (every backend feature)
- `docs/design.md` — tokens, components (every UI feature)

| App | Audience | Domain | Dev port |
|---|---|---|---|
| `imacals-api` | — | Rust/Actix API behind both front ends | 3032 |
| `imacals-dashboard` | Staff | `dashboard.imacals.com` | 5174 |
| `imacals-web` | Customers | `imacals.com` | 5175 |

---

## Stack

- Rust (edition 2021), Actix-web 4, PostgreSQL 17, sqlx, JWT, MinIO

```
imacals-api/src/
├── controllers/api/   ← HTTP handlers (thin)
├── models/            ← DB-mirroring structs
├── repositories/      ← all SQL lives here
├── services/          ← business logic
├── routes/api.rs      ← URL wiring
├── middlewares/       ← User + Organization from request
├── macros/            ← gate!, can!
├── utilities/         ← ErrorBag, JsonResponse
└── migrations/        ← SQL up/down files
```

---

## Feature checklist (follow in order)

```
[ ]  1. Read business_logic.md — columns, roles, rules
[ ]  2. If UI: read design.md — tokens, single-accent rule
[ ]  3. Migration — table with id, org, timestamps, soft-delete, INDEXES (every FK + soft-delete partials + unique-active), and soft_delete_cascade_* triggers for any parent → child relationship
[ ]  4. Model — struct + input schema + plain-English comments
[ ]  5. Repository — CRUD, all SQL here
[ ]  6. Service — business logic, returns ErrorBag not sqlx::Error
[ ]  7. Controller — thin: gate!, call repo/service, return JSON
[ ]  8. Route — GET/POST/PUT/DELETE in routes/api.rs
[ ]  9. Permissions migration — seed + role assignments
[ ] 10. Tests — schema deserialization + validation + any branching logic
[ ] 11. Update docs/business_logic.md
[ ] 12. If UI: update docs/design.md if new component introduced
[ ] 13. Update AGENTS.md if new code pattern introduced
[ ] 14. MANDATORY: run `make check-api` and fix every error before moving on.
        Never skip. Never mark a task done if this fails.
[ ] 15. If UI: add Playwright e2e next to the app you changed —
        imacals-dashboard/e2e/<feature>.spec.ts  or  imacals-web/e2e/<feature>.spec.ts
        Mock all API calls with page.route(). Cover: renders, data display, error state.
[ ] 16. If a customer-facing endpoint: confirm it works for a phone order too, not just online.
```

---

## Migration

File: `src/migrations/<timestamp>_create_<entity>s_table.up.sql`

Required columns on every table:
- `id UUID PRIMARY KEY DEFAULT uuid_generate_v4()`
- `organization_id UUID NOT NULL REFERENCES organizations(id)` — **only for app entities that belong to a tenant** (e.g. products, orders, customers). Do NOT add it to global/admin data (geo reference tables, polygons, domains, system config, etc.).
- `domain_id UUID NOT NULL REFERENCES domains(id)` — **only for reference data that varies by location** (product categories, price lists, delivery tariffs, etc.). Do NOT add it to tenant entities or geo/infrastructure tables. Slug/name uniqueness must be scoped to `(domain_id, slug)` not globally.
- `created_by UUID NOT NULL REFERENCES users(id)` (if ownership matters)
- `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- `deleted_at TIMESTAMPTZ` (soft delete — never hard delete)

### Indexes (MANDATORY — every migration must declare them)

Every `up.sql` MUST include an `-- Indexes` section right after the `CREATE TABLE`. A migration without indexes is incomplete — do not commit one. Apply these rules:

- **Every FK column gets its own index.** PostgreSQL does not auto-index FKs, and join performance collapses without them. Skip only when the FK is the leading column of an existing UNIQUE / PRIMARY KEY index.
- **Compound PK / UNIQUE on `(a, b)` does NOT cover lookups by `b` alone.** Add a separate index on `b` when reverse lookups happen (e.g. `role_permissions.permission_id`, `polygon_neighbors.neighbor_polygon_id`).
- **Soft-delete aware indexes.** When the table has `deleted_at`, partial-index hot lookups with `WHERE deleted_at IS NULL` so the index stays small and skips tombstones. Also add a plain `(deleted_at)` index if you ever query deleted rows (audit, restore).
- **Uniqueness must be soft-delete aware.** A unique constraint over `(org_id, slug)` blocks reusing a slug after soft-delete — use `CREATE UNIQUE INDEX … WHERE deleted_at IS NULL` instead of a table-level `UNIQUE` constraint.
- **Slug / name / search columns** that drive lookups or autocomplete get their own index (partial on `deleted_at IS NULL`).
- **Common filter combinations** (e.g. `(trade_id, work_stage, phase)`, `(user_id, document_type)`) get a composite index. Order columns by selectivity / leading-equality usage.
- **At-most-one invariants** (e.g. one primary bank account per user) are enforced with a partial UNIQUE index, not application logic: `CREATE UNIQUE INDEX … ON t (user_id) WHERE is_primary = TRUE AND deleted_at IS NULL`.
- **Geospatial pairs** (`latitude`, `longitude`) get a composite index when bounding-box queries are expected.

Naming: `<table>_<col>_index` (or `idx_<table>_<col>` — match the rest of the file). Unique: `uq_<table>_<cols>`. Partial indexes that filter soft-deletes don't need a suffix — the comment explains it.

Skeleton:

```sql
-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS products_organization_id_index
    ON products (organization_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS products_created_by_index
    ON products (created_by)
    WHERE deleted_at IS NULL;

-- Slug must be unique per tenant but reusable after soft-delete.
CREATE UNIQUE INDEX IF NOT EXISTS uq_products_org_slug_active
    ON products (organization_id, slug)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering on listings / audit queries.
CREATE INDEX IF NOT EXISTS products_deleted_at_index
    ON products (deleted_at);
```

### Soft-delete cascade triggers (use where children should follow the parent)

`src/migrations/20250203165754_create_soft_delete_cascade_functions.up.sql` defines three reusable trigger functions. **Use them — never duplicate the logic inline, and never let children become orphans of a soft-deleted parent.**

| Function | When to use | Trigger arg(s) |
|---|---|---|
| `soft_delete_cascade_by_parent_id()` | Self-referential trees (e.g. `organizations.parent_id`, `spaces.parent_id`). Cascades within the SAME table. | none |
| `soft_delete_cascade_by_fk('child_table', 'fk_col')` | Direct FK from child to parent (the common case — `organization_users_permissions.organization_users_id`, etc.). | child table name, FK column |
| `soft_delete_cascade_by_owner('child_table')` | Polymorphic ownership where child has `owner_type` + `owner_id`. | child table name |

Rules:
- Attach a trigger for every parent → child relationship where the child must vanish from listings when the parent is soft-deleted. If unsure, ask: "would a user be confused to see this child row after deleting its parent?" — if yes, add the trigger.
- Trigger naming: `trg_soft_delete_<child>_on_<parent>_delete`.
- Triggers go in the parent's `up.sql` (or the child's, if added later) — keep the trigger near whichever table makes the relationship most discoverable.
- `down.sql` must `DROP TRIGGER IF EXISTS …` before dropping the table.

Attachment example (from `organization_users_permissions`):

```sql
CREATE TRIGGER trg_soft_delete_org_user_permissions_on_org_user_delete
    AFTER UPDATE OF deleted_at ON organization_users
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('organization_users_permissions', 'organization_users_id');
```

Self-referential example (organizations):

```sql
CREATE TRIGGER trg_soft_delete_organizations_on_parent_delete
    AFTER UPDATE OF deleted_at ON organizations
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_parent_id();
```

When NOT to cascade: reference tables shared across tenants, or audit tables that must outlive the parent (e.g. `order_status_history` — an order's history is the record of what happened and must survive the order being hidden). Document the omission in a comment.

---

## Model

File: `src/models/<entity>.rs` — add `pub mod <entity>;` to `models/mod.rs`

- `#[derive(Debug, Clone, Serialize, Deserialize)]` on model structs
- `#[derive(Debug, Deserialize, Validate)]` on input schemas
- Nullable SQL columns → `Option<T>`
- `#[serde(skip_serializing_if = "Option::is_none")]` on `deleted_at`
- `#[serde(skip_serializing)]` on sensitive fields (passwords, tokens)

```rust
// A sellable line in the catalogue. Stock lives in stock_levels, not here — one product can sit
// in more than one warehouse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub domain_id: Uuid,
    pub created_by: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    // The unit one quantity buys — "carton", "bag (50kg)", "piece".
    pub unit: String,
    // Kobo, never naira: integer money is the only kind that survives arithmetic.
    pub unit_price_kobo: i64,
    // Wholesale lines often cannot be bought as singles.
    pub min_order_quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// The form a caller sends when creating a product.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductSchema {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 255, message = "Slug must be between 1 and 255 characters"))]
    pub slug: String,
    pub description: Option<String>,
    pub unit: String,
    #[validate(range(min = 1, message = "Price must be greater than zero"))]
    pub unit_price_kobo: i64,
    pub min_order_quantity: Option<i32>,
}
```

---

## Repository

File: `src/repositories/<entity>_repository.rs` — add to `repositories/mod.rs`

- All SQL lives here — never in controllers or services
- `sqlx::query_as!` for SELECT; `sqlx::query!` for INSERT/UPDATE/DELETE
- Always filter `deleted_at IS NULL` on SELECT
- Return `Result<T, sqlx::Error>` — never `.unwrap()`

```rust
// ProductRepository is the only place that talks to the products table.
pub struct ProductRepository;

impl ProductRepository {
    // Returns an error if the product doesn't exist or is soft-deleted.
    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Product, Error> {
        Ok(sqlx::query_as!(Product,
            "SELECT * FROM products WHERE id = $1 AND deleted_at IS NULL LIMIT 1", id
        ).fetch_one(pool).await?)
    }

    // Soft-delete: keeps the row so past orders still resolve their line items.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE products SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL", id
        ).execute(pool).await?.rows_affected())
    }
}
```

---

## Service

File: `src/services/<entity>_service.rs` — add to `services/mod.rs`

Skip if handler needs no business logic (≤5 lines, no branching).

- Return `Result<T, ErrorBag>`, not `sqlx::Error`
- Map DB errors → `ErrorBag::InternalServerError(format!("ContextName::method failed: {:?}", e))`
- Unique-constraint violation (pg `23505`) → `ErrorBag::EmailInUse` or appropriate variant

---

## Controller

File: `src/controllers/api/<entity>_controller.rs` — add to `controllers/api/mod.rs`

- Import `JsonResponse` and `ErrorBag` directly (not via prelude)
- First line of every authenticated handler: `crate::gate!(...)`
- `show`/`update`/`delete`: match `Error::RowNotFound` → `ErrorBag::NotFound`
- `delete`: `rows_affected() == 0` → `ErrorBag::NotFound`
- Never `.unwrap()`

```rust
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

pub async fn show(user: User, organization: Organization, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "products.view");
    match ProductRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(p)                   => JsonResponse::success(p),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Product".into())),
        Err(e)                  => JsonResponse::fatal(e, "product_controller.show failed"),
    }
}

pub async fn delete(user: User, organization: Organization, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "products.delete");
    match ProductRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0)  => JsonResponse::error(ErrorBag::NotFound("Product".into())),
        Ok(_)  => JsonResponse::success(json!({ "message": "Product deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "product_controller.delete failed"),
    }
}
```

---

## Routes

`src/routes/api.rs` — import controller at top, add inside `init()`:

```rust
use crate::controllers::api::product_controller;

.service(
    web::scope("/products")
        .route("",      get().to(product_controller::index))
        .route("",      post().to(product_controller::create))
        .route("/{id}", get().to(product_controller::show))
        .route("/{id}", put().to(product_controller::update))
        .route("/{id}", delete().to(product_controller::delete))
)
```

---

## Permissions migration

```sql
INSERT INTO permissions (name, slug) VALUES
    ('View Products',   'products.view'),
    ('Create Products', 'products.create'),
    ('Update Products', 'products.update'),
    ('Delete Products', 'products.delete');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.slug = 'admin' AND p.slug LIKE 'products.%';
```

---

## Permission macros

```rust
crate::gate!(&app.pool, &user, &organization, "entity.action");          // block if missing
crate::gate_any!(&app.pool, &user, &organization, &["a", "b"]);          // block if none match
crate::gate_all!(&app.pool, &user, &organization, &["a", "b"]);          // block if any missing
if crate::can!(&app.pool, &user, &organization, "entity.action") { ... } // boolean, no block
```

`user.is_superuser == true` bypasses all gates.

---

## Error handling

| Situation | Return |
|---|---|
| Not found by id | `ErrorBag::NotFound("EntityName".into())` |
| `rows_affected() == 0` on delete | `ErrorBag::NotFound("EntityName".into())` |
| User not logged in | `ErrorBag::Unauthorized` |
| User lacks permission | `ErrorBag::Forbidden` (403) / let `gate!` handle it — never `Unauthorized`, which says "log in again" |
| Email taken | `ErrorBag::EmailInUse` |
| Field invalid | `ErrorBag::Validation { field, message }` |
| DB / unexpected | `JsonResponse::fatal(err, "context")` |
| New business error | Add variant to `utilities/error_bag.rs` |

---

## Comment style

Comments document **intent** — non-obvious constraints, invariants, access-control decisions, or data-format details that would surprise a future reader. Skip self-explanatory code.

Rules:
- Write for a developer who knows Rust but is new to this domain.
- Say WHY something exists or what non-obvious constraint it enforces, not what the code mechanically does.
- One line max. If you need two, the comment is too long.
- Do NOT comment functions, fields, or structs whose names already tell the full story.

Good: `// Soft-delete: preserving the row lets us audit or recover shapes after deletion.`
Good: `// Polygons are global admin data — never scoped to an organization.`
Good: `// Coordinates stored as [{lat, lng}] JSONB so the shape can be edited without schema changes.`
Bad:  `// Returns every polygon that has not been deleted, newest first.` (restates the query)
Bad:  `// Saves the polygon the admin just finished drawing.` (restates the method name)
Bad:  `// The admin who drew this polygon.` (restates the field name `created_by`)

---

## Tests

Every entity needs tests. No exceptions. Put them in the same file in a `#[cfg(test)]` block at the bottom. Tests live in three layers — write all three.

---

### 1 — Model tests (no database, no HTTP)

Test that the schema struct deserialises correctly and that `#[validate]` rules fire.
Use plain `#[test]`.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Missing required field must fail to parse.
    #[test]
    fn create_schema_requires_coordinates() {
        let result: Result<CreatePolygonSchema, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }

    // A well-formed payload must parse without errors.
    #[test]
    fn create_schema_accepts_valid_payload() {
        let json = r#"{"coordinates": [{"lat": 25.77, "lng": -80.19}]}"#;
        assert!(serde_json::from_str::<CreatePolygonSchema>(json).is_ok());
    }
}
```

---

### 2 — Repository tests (real database, no HTTP)

Test that the SQL does what it says: records save, soft-delete hides rows, etc.
Use `#[sqlx::test(migrations = "./src/migrations")]` — each test gets its own fresh database that is thrown away automatically.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use serde_json::json;

    // A newly created polygon should come back when we list all polygons.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn created_polygon_appears_in_index(pool: PgPool) {
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T','t@t.com','x',NOW()) RETURNING id"
        ).fetch_one(&pool).await.unwrap();

        PolygonRepository::create(&pool, &user_id, &json!([]), None).await.unwrap();
        let rows = PolygonRepository::index(&pool).await.unwrap();
        assert_eq!(rows.len(), 1);
    }

    // A soft-deleted polygon must not appear in the list.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleted_polygon_is_hidden(pool: PgPool) {
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T','t2@t.com','x',NOW()) RETURNING id"
        ).fetch_one(&pool).await.unwrap();

        let p = PolygonRepository::create(&pool, &user_id, &json!([]), None).await.unwrap();
        PolygonRepository::delete(&pool, &p.id).await.unwrap();

        let rows = PolygonRepository::index(&pool).await.unwrap();
        assert!(rows.is_empty(), "deleted polygon should not appear");
    }
}
```

---

### 3 — Controller tests (real database + real HTTP)

Test that the HTTP layer behaves correctly: right status codes, permission guards block the right people.
Use `#[sqlx::test(migrations = "./src/migrations")]` for the pool, and `actix_web::test` to make HTTP requests.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use actix_web::http::StatusCode;
    use sqlx::PgPool;
    use serde_json::json;
    use crate::AppState;
    use crate::services::jwt_service::JwtService;

    // Helper: creates a user in the DB and returns a signed Bearer token for them.
    async fn make_user_token(pool: &PgPool, email: &str, superuser: bool) -> String {
        let id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T',$1,'x',$2,NOW()) RETURNING id",
            email, superuser
        ).fetch_one(pool).await.unwrap();
        format!("Bearer {}", JwtService::create_access_token(id, 60).unwrap())
    }

    // A regular user trying to create a polygon should get 403 Forbidden.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn regular_user_cannot_create(pool: PgPool) {
        let token = make_user_token(&pool, "regular@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/polygons").route("", web::post().to(create)))
        ).await;

        let req = test::TestRequest::post()
            .uri("/polygons")
            .insert_header(("Authorization", token))
            .set_json(json!({"coordinates": []}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // A superuser should be able to create a polygon and get it back.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_create(pool: PgPool) {
        let token = make_user_token(&pool, "admin@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/polygons").route("", web::post().to(create)))
        ).await;

        let req = test::TestRequest::post()
            .uri("/polygons")
            .insert_header(("Authorization", token))
            .set_json(json!({"coordinates": [{"lat": 25.77, "lng": -80.19}]}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
```

---

## E2E Tests

Every feature that touches a UI must have e2e tests — no exceptions.

- Dashboard features → Playwright spec in `imacals-dashboard/e2e/<feature>.spec.ts`
- Storefront features → Playwright spec in `imacals-web/e2e/<feature>.spec.ts`

The storefront's `playwright.config.ts` starts its dev server with `VITE_USE_PREVIEW_CATALOG=false`
so `page.route()` mocks actually fire — with the preview catalogue on, the service short-circuits
before `fetch` and no route ever matches.

---

### Dashboard — Playwright

**Rules:**
- All API calls must be intercepted with `page.route()` — never rely on a live backend.
- **`goto()` BEFORE touching localStorage.** A fresh page is `about:blank`, where `localStorage`
  access throws `SecurityError`. Navigate to `/login` (public, no API needed), seed the token, then
  navigate to the route under test. Seeding before any navigation fails every test in the file.
- Mock `**/api/auth/me` before navigating to any authenticated route.
- Cover: page renders, data appears in the DOM, empty state, error state.
- Use semantic selectors in order of preference: `getByRole` → `getByText` → CSS class selectors that already exist in the component (never add `data-testid` to dashboard components).
- **Pass `exact: true` when a name is a prefix of another control's.** A toolbar `+ Add Provider`
  and a modal's `Add Provider` submit both match `getByRole('button', { name: 'Add Provider' })`,
  and strict mode fails the click. Same trap with `getByLabel('Password')` and a
  `Show password` toggle.

**File:** `imacals-dashboard/e2e/<feature>.spec.ts`

```typescript
import { test, expect, type Route } from '@playwright/test';

const MOCK_TOKEN = 'test-jwt-token';
const MOCK_USER  = { id: 'u1', first_name: 'Test', last_name: 'User',
                     email: 'test@imacals.com', is_superuser: false, is_internal: false };

function mockMe(route: Route): void {
  route.fulfill({ status: 200, contentType: 'application/json',
    body: JSON.stringify({ success: 'true', data: { user: MOCK_USER } }) });
}

test.describe('<Feature> page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate first: localStorage is unreachable on about:blank.
    await page.goto('/login');
    await page.evaluate((t) => localStorage.setItem('token', t), MOCK_TOKEN);
    await page.route('**/api/auth/me', mockMe);
    await page.route('**/api/<entity>', (route) =>
      route.fulfill({ status: 200, contentType: 'application/json',
        body: JSON.stringify({ success: 'true', data: [ /* mock rows */ ] }) }));
  });

  test('renders page heading', async ({ page }) => {
    await page.goto('/<route>');
    await expect(page.getByRole('heading', { name: '<Heading>' })).toBeVisible();
  });

  test('renders a row for each item', async ({ page }) => {
    await page.goto('/<route>');
    await expect(page.locator('.<table-class> tbody tr')).toHaveCount(/* n */);
  });

  test('shows error state when API fails', async ({ page }) => {
    await page.route('**/api/<entity>', (route) =>
      route.fulfill({ status: 500, contentType: 'application/json',
        body: JSON.stringify({ success: 'false', code: 'InternalServerError',
          error: { message: 'Unexpected error' } }) }));
    await page.goto('/<route>');
    await expect(page.locator('.state-msg--error')).toBeVisible();
  });
});
```

---

## TypeScript rules (both front ends)

Always write explicit types — never rely on inference.

- `ref` / `computed` — always supply generic: `ref<boolean>(false)`, `computed<number>(() => ...)`
- Functions — always annotate return type: `fn(): void`, `async fn(): Promise<void>`, composables spell out full return type
- External data — `const json: unknown = await res.json()` (not implicit `any`)
- Import Vue types with `type`: `import { ref, computed, type Ref, type ComputedRef } from 'vue'`

---

## Import paths (both front ends)

Always use `@` alias — never relative paths (`../`, `./`). `@` maps to `src/`.

```ts
import { useAuth } from '@/composables/useAuth';
import UsersAllView from '@/views/UsersAllView.vue';
```

- dashboard and storefront: `resolve.alias` in `vite.config.ts` + `paths` in `tsconfig.json`

---

## Money

Prices, totals and fees are an **integer count of kobo** (₦1 = 100 kobo) in the database, in the API
and on the wire. Never a float, never a decimal string in JSON. Format for display only at the edge
(`formatNaira()` in the storefront). Rounding drift on a quoted price is a bug you cannot apologise
your way out of.

---

## UI rules (read `docs/design.md` in full before any UI work)

- Use design tokens — never hardcode hex, font names, or px values
- One Tertiary (`#B8422E`) action per screen — reserve it for the primary CTA
- No gradients · no extra accent colors · negative space is intentional
- Update `docs/design.md` when introducing a reusable new component
- The token block at the top of `imacals-dashboard/src/style.css` and `imacals-web/src/style.css` is
  one contract — change a value in one and change it in the other

---

## File Uploads

All uploaded files use a single polymorphic `files` table. Never store paths directly on entity tables. See `docs/business_logic.md §6` for the full rule set.

### Migration skeleton

```sql
CREATE TABLE IF NOT EXISTS files (
    id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_by     UUID NOT NULL REFERENCES users(id),
    fileable_type  VARCHAR  NOT NULL,
    fileable_id    UUID          NOT NULL,
    type           VARCHAR  NOT NULL,
    name           VARCHAR  NOT NULL,
    absolute_path  TEXT          NOT NULL,
    relative_path  TEXT          NOT NULL,
    size           BIGINT       NOT NULL,
    mime_type      VARCHAR NOT NULL,
    created_at     TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    deleted_at     TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- Primary lookup: all files for a given owner row.
CREATE INDEX IF NOT EXISTS files_fileable_index
    ON files (fileable_type, fileable_id)
    WHERE deleted_at IS NULL;

-- FK: who uploaded the file.
CREATE INDEX IF NOT EXISTS files_created_by_index
    ON files (created_by)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering.
CREATE INDEX IF NOT EXISTS files_deleted_at_index
    ON files (deleted_at);

-- At-most-one constraint example (use when only one file of a given type is allowed per owner):
-- CREATE UNIQUE INDEX IF NOT EXISTS uq_files_owner_type_active
--     ON files (fileable_type, fileable_id, type)
--     WHERE deleted_at IS NULL;
```

### FileType enum (Rust)

Define in `src/models/file.rs`. Add a new variant whenever a new upload use-case is introduced — never use a free-form string.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum FileType {
    UserSignature,       // user-signature       — owner: users
    UserInitials,        // user-initials        — owner: users
    UserProofOfFunds,    // user-proof-of-funds  — owner: users
    ProductImage,        // product-image        — owner: products
    OrderAttachment,     // order-attachment     — owner: orders
}

// File mirrors the files table. Paths come from MinIO; type scopes the file's purpose.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id:             Uuid,
    pub created_by:     Uuid,
    pub fileable_type:  String,
    pub fileable_id:    Uuid,
    pub file_type:      FileType,  // mapped via "type" AS "file_type: FileType" in every query
    pub name:           String,
    pub absolute_path:  String,
    pub relative_path:  String,
    pub size:           i64,
    pub mime_type:      String,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at:     Option<DateTime<Utc>>,
}
```

### Upload flow (service layer)

1. Receive the multipart upload in the controller.
2. Call the MinIO/S3 helper to stream the object → get back `(absolute_path, relative_path)`.
3. Insert a `files` row via `FileRepository::create(pool, created_by, fileable_type, fileable_id, file_type, name, absolute_path, relative_path)`.
4. Return the `File` struct to the caller.

Never call the storage helper from a controller — always go through a service.

### Querying files for an entity

```rust
// Returns all active files for a given owner.
pub async fn find_for_owner(
    pool: &PgPool,
    fileable_type: &str,
    fileable_id: &Uuid,
) -> Result<Vec<File>, Error> {
    sqlx::query_as!(File,
        r#"SELECT id, created_by, fileable_type, fileable_id,
                  type AS "type: FileType", name,
                  absolute_path, relative_path,
                  created_at, updated_at, deleted_at
           FROM files
           WHERE fileable_type = $1
             AND fileable_id   = $2
             AND deleted_at IS NULL"#,
        fileable_type, fileable_id
    ).fetch_all(pool).await
}
```

---

## Runtime configuration (never read env at use time)

Third-party credentials and provider choices live in the database, not the environment. `.env` is a
**seed** only. See `docs/business_logic.md §7` for the full rule set; the pattern in code:

```rust
// Wrong: the value is frozen at boot, so changing it needs a restart.
let key = &ENV.mailgun_api_key;

// Right: resolved per use, so a dashboard edit applies to the next send.
let provider = IntegrationResolverService::resolve(&app.pool, IntegrationCategory::Email).await?;
let key = provider.required("MAILGUN_API_KEY")?;   // decrypted here
let region = provider.optional("MAILGUN_REGION");  // has a sensible default
```

Rules:
- A new provider = a new `IntegrationType` variant + a `FieldDef` template in
  `utilities/integration_type_defs.rs` + a `category()` arm. Nothing else hardcodes its field names.
- Add a seed function only to bootstrap a fresh install; guard it on the env var being present and
  on the slug not already existing.
- Never cache resolved credentials in a `static`/`LazyLock` — that reintroduces restart-to-apply.
- Secrets are encrypted with `APP_SECRET` before insert and masked out of API responses.

---

## Keeping docs in sync

| File | Update when |
|---|---|
| `docs/business_logic.md` | Entity added/changed, role/rule changed, feature removed |
| `docs/design.md` | New reusable component, token value changed, new layout rule |
| `AGENTS.md` | New shared code pattern, convention changed, template drifted |

---

## Build verification (required after every change)

Run inside containers (`make dev` must be running first). Fix all errors before moving on.

| Project | Command | Notes |
|---|---|---|
| `imacals-api` | `make check-api` | `cargo check` — needs a migrated DB, or `SQLX_OFFLINE=true` |
| `imacals-dashboard` | `make check-dashboard` | `vue-tsc` + Vite bundle |
| `imacals-web` | `make check-web` | `vue-tsc` + Vite bundle |

---

## Running the project

```bash
make dev            # start API + DB + MinIO + Mailpit
make run-migration  # apply pending migrations
make prepare        # regenerate sqlx offline query metadata
make help           # list every target, grouped
```

The API also runs embedded migrations on boot (`sqlx::migrate!` in `main.rs`), so a fresh
container self-migrates. Adding a migration is a SQL-only change and cargo won't rebuild on
it — touch a `.rs` file (or rebuild) or the embedded set stays stale.

### Local services

| Service | Where | Notes |
|---|---|---|
| API | `http://localhost:3032` | `APP_PORT` |
| Dashboard | `http://localhost:5174` | `DASHBOARD_HOST_PORT` |
| Storefront | `http://localhost:5175` | `WEB_HOST_PORT` |
| Postgres | `localhost:5437` | `DB_HOST_PORT` — off 5432 to avoid clashing with a local/sibling postgres |
| MinIO console | `http://localhost:9003` | `MINIO_CONSOLE_PORT` |
| Mail inbox (Mailpit) | `http://localhost:8026` | `MAIL_UI_PORT` |

Host-port defaults deliberately avoid the common ones (5432/9000/9002/8025) so a sibling
project's stack can run at the same time. In-container hostnames/ports never change.

**Email in dev never leaves the machine.** Mailpit (`imacals-mail`, SMTP at `imacals-mail:1025`)
captures every send and shows it in the web inbox above. All mail-sending code must go through
`MAIL_HOST`/`MAIL_PORT` — never hardcode a relay, or dev traffic hits real recipients.

The root `Makefile` is thin: it loads `imacals-api/.env` then includes one group file per
command family from `scripts/mk/`. Add a new command family by dropping a `*.mk` there,
including it in the root `Makefile`, and adding a line to `help`.

| Group file | Targets |
|---|---|
| `scripts/mk/dev.mk` | `dev` `api` `down` `dashboard` `web` |
| `scripts/mk/db.mk` | `clean-db` `create-migration` `run-migration` `rollback-migration` `prepare` |
| `scripts/mk/check.mk` | `check-api` `check-dashboard` `check-web` `free-cargo` |
| `scripts/mk/test.mk` | `test` `test-up` `test-api` `test-dashboard[-spec/-watch/-ui/-report]` `test-web[-spec/-ui/-report]` |

Every group file ends with a `.PHONY` line listing its targets.
