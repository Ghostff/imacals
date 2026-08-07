# Business Rules: Imacals

Source of truth for all business rules. AI must follow strictly — no exceptions.

---

## 0. What Imacals is

Imacals is an **ecommerce and distribution business** operating in Nigeria.

- **Base distribution warehouse: Aba, Abia State.** Every order is picked there. Delivery lead
  times are quoted as distance from Aba — same day inside Aba, longer the further out.
- **Two ordering channels, one order book.** Customers order online at `imacals.com` or by calling
  the order desk, which enters the order on their behalf. A phone order and an online order become
  the same record, carry the same reference, and move through the same status flow. Nothing may be
  built that works for one channel and not the other.
- **We deliver.** Fulfilment is our own — dispatch, routing and proof of delivery are part of the
  product, not an outsourced afterthought.

### The three apps

| App | Audience | Domain |
|---|---|---|
| `imacals-api` | — | Rust/Actix API behind both front ends |
| `imacals-dashboard` | Staff: warehouse, dispatch, order desk | `dashboard.imacals.com` |
| `imacals-web` | Customers | `imacals.com` |

### Build status — read before planning work

The platform layer below (users, roles, permissions, organizations, domains, geo, polygons/zones,
files, integrations) is **built and working**. The ecommerce domain itself — products, stock,
orders, delivery zones, payments — is **not built yet**; see §3.

The renovation-era entities this codebase started from (trades, materials, material sources,
spaces, user trades, user businesses) have been **removed**. Some renovation vocabulary still
survives in seeded data and is called out where it appears.

---

## 1. Roles

Roles are split into two distinct concepts:

### 1a. Permission Roles (`roles` table)

Grant a standard bundle of permissions within an organization. Stored in the `roles` table and linked to permissions via `role_permissions`.

| Role | Key | Description |
|---|---|---|
| Super Admin | `super-admin` | Unrestricted system access. Cannot be assigned to a regular user. |
| Admin | `admin` | Full access: user management, financial oversight, system config. |

### Hierarchy (highest → lowest)
`super-admin` > `admin`

A user's permission role is stored as `role_id` on their `organization_users` record. It controls what API actions they can perform within an organization.

### 1b. Organization User Roles / Job Titles (`organization_user_role` table)

Describe what a person **does** — their profession or function — not what they are allowed to access. Stored in `organization_user_role` and linked as `user_role_id` on `organization_users`.

Job title entries with `organization_id IS NULL` are global/platform defaults available to all orgs.
Orgs can also create custom job titles scoped to their own `organization_id`.

> **These rows are still the renovation-era set.** `20260510222220_create_organization_user_role_table`
> seeds `ai`, `broker`, `contractor`, `hml`, `insurance`, `operator`, `project-manager`, `realtor` —
> none of which describe an Imacals job. They are left in place because nothing has replaced them
> yet, and code still reads them (`system_user_eligible`, the proof-of-funds gate on the user
> profile). Replacing them with the real set — order desk, warehouse picker, dispatch rider,
> delivery manager, accounts — is a migration that has not been written.

The job titles Imacals actually needs, for whoever writes that migration:

| Job Title | Key | Description |
|---|---|---|
| Order Desk | `order-desk` | Takes phone orders and enters them on the customer's behalf. |
| Warehouse | `warehouse` | Picks and packs orders in the Aba warehouse; adjusts stock. |
| Dispatch | `dispatch` | Assigns orders to vehicles and routes; confirms delivery. |
| Rider / Driver | `rider` | Carries the load and captures proof of delivery. |
| Accounts | `accounts` | Reconciles payments, issues refunds. |

---

## 2. Organizations & User Membership

- Every user belongs to at least one organization. The default organization is **"imacals"** (the platform operator's own org, identified by slug `imacals`).
- A user can be a member of **multiple organizations** simultaneously.
- Each user has an **independent set of permissions per organization** — membership and permissions in one org have no effect in another.
- **Superusers** (`is_superuser = true`) bypass all permission checks across all organizations.
- Internal users (`is_internal = true`, i.e., Imacals employees) have cross-org visibility: they can see all organizations and their users.

### Ways a user can join an organization

| Path | Who triggers it | Rule |
|---|---|---|
| **Self-registration** | The user themselves | If an org is specified at signup (e.g., via invite link), the user joins that org. If no org is specified, they are added to the default "imacals" org. |
| **Admin creates user** | Any member with `users.create` permission | The new user is added to the acting user's current org. If no org is specified, defaults to the acting user's org. |
| **Invitation** | Any org member with `users.create` permission | A member can invite a user into their org. The invited user joins upon acceptance. |
| **Cross-org assignment** | Internal users from the "imacals" context only | Only Imacals staff can assign a user to multiple orgs or to an org other than their own in a single operation. |

### Permission model

- Permissions are stored directly on each user ↔ org membership (`organization_users_permissions`).
- Roles define a standard permission bundle (`role_permissions`). When assigning a user a role in an org, the role's permissions are resolved and written as direct permissions on the user's membership record.
- Permission sync is explicit: changing or revoking a role does not automatically update permissions — they must be re-synced.
- Soft-deleting a user from an organization cascades and removes all their permissions for that org.

---

## 3. The ecommerce domain — not built yet

Nothing below exists in the database. It is written down so the next person builds the same thing
the storefront already assumes. `imacals-web` calls `/catalog/products`, `/catalog/categories`,
`/orders` and `/orders/:reference/track`; until they exist it runs on the preview catalogue in
`imacals-web/src/services/catalog.ts` behind `VITE_USE_PREVIEW_CATALOG`.

### Tables to build

| Table | Notes |
|---|---|
| `warehouses` | The Aba base warehouse is the first row. Orders are picked from a warehouse. |
| `categories` | Domain-scoped, self-referential for sub-categories. |
| `products` | Domain-scoped. `slug` unique per `(domain_id, slug)` where `deleted_at IS NULL`. |
| `stock_levels` | Per `(product_id, warehouse_id)`. Never a bare column on `products`. |
| `customers` | A buyer. May exist without a `users` row — phone orders create one from a name and number. |
| `customer_addresses` | Multiple per customer; one default. |
| `orders` | Carries `channel` (`online` \| `phone`), `reference`, `status`, totals, warehouse. |
| `order_items` | Line snapshot: unit price copied at order time so later price changes never rewrite history. |
| `order_status_history` | Append-only. One row per transition, with actor and timestamp. |
| `delivery_zones` | Ties a geographic area to a tariff. Should reuse `polygons` / `zones`. |
| `delivery_fees` | Fee per zone, per weight or value band. |
| `payments` | Against an order. Partial payment and refund must both be representable. |

### Rules the storefront already depends on

- **Money is an integer count of kobo** (₦1 = 100 kobo) everywhere it crosses the wire. No floats,
  no decimal strings in JSON. `imacals-web` sums cart and order totals as integers on that promise.
- **`min_order_quantity` is real.** Wholesale lines cannot be bought below it. The cart drops a line
  rather than let its quantity fall under the minimum, and the API must reject one that does.
- **Prices are re-resolved server side at order time.** The client sends `product_id` and
  `quantity` only. Never trust a price that arrived from a browser.
- **`reference` is customer-facing** and is read aloud on the phone — short, unambiguous, no
  lookalike characters. Both channels get one from the same sequence.
- **Delivery fee is quoted at checkout**, not in the cart, because it depends on the destination.

### Order status flow

```
pending → confirmed → picked → dispatched → delivered
              ↓          ↓          ↓
           cancelled  cancelled  returned
```

- A phone order may be created straight into `confirmed` — the desk confirmed it on the call.
- Every transition writes an `order_status_history` row. The customer-facing tracking endpoint
  reads that history, so anything not recorded there is invisible to the customer.
- `delivered` is terminal except for `returned`.

---
## 4. Domains

A **Domain** is a geographic namespace that scopes reference data to a specific location. The same entity name can exist in multiple domains with different properties or availability. For Imacals this is how a catalogue, a price list or a delivery tariff can differ between Aba and, say, Lagos without duplicating the whole product tree.

### Structure

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `name` | TEXT | Human-readable label, e.g. "Default US", "Texas" |
| `country_id` | UUID FK | Always required |
| `state_id` | UUID FK | NULL = country-level domain |
| `city_id` | UUID FK | NULL = state-level (or country-level) domain |

No `organization_id` — domains are global admin data, not tenant-scoped.

Only one domain can exist per unique `(country_id, state_id, city_id)` combination (enforced by unique constraint).

### Resolution

Given a user's location, the system finds the single winning domain by checking from most to least specific:

1. City-level: `city_id = given_city`
2. State-level: `state_id = given_state AND city_id IS NULL`
3. Country-level: `country_id = given_country AND state_id IS NULL AND city_id IS NULL`

The first match wins. Resolution logic lives in a service, not in controllers or repositories.

### Default domains

Each country should have a country-level domain as the final fallback.

> The platform still ships **"Default US"** (`country = USA, state = NULL, city = NULL`), seeded at
> migration time — a leftover of the codebase's origin, and the fallback `integration_seed` and
> the integration repository still look up by slug `default-us`. Imacals operates in Nigeria, so a
> `default-ng` country-level domain (and an Abia/Aba domain beneath it) needs seeding, with the
> `default-us` references repointed. That migration has not been written.

### Domain-scoped entities

Any reference entity that varies by location carries `domain_id NOT NULL REFERENCES domains(id)`. A slug or name must be unique **within a domain**, not globally. When fetching entities for a location, resolve the domain first, then query by `domain_id`.

Current domain-scoped entities: **integrations**. The catalogue tables in §3 should be
domain-scoped too when they are built — that is what lets a price list vary by market.

---

## 5. System Users

A **System User** is a real platform user designated to fill a specific role within a domain — the
person the system acts through when it needs a named human for that region.

> The eligible roles are still the renovation-era `hml`, `insurance`, `broker`, `realtor` (set by
> `20260511290000_add_system_user_eligible_to_user_roles`), and the original purpose was signing
> generated offers. For Imacals the mechanism is the right shape for naming a regional delivery
> manager or accounts contact per domain, but the eligible set needs to change with the job titles
> in §1b.

### Structure (`domain_system_users` table)

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `domain_id` | UUID FK | The geographic domain this assignment applies to |
| `user_id` | UUID FK | The platform user filling the role |
| `role` | VARCHAR | `'broker'` or `'realtor'` |
| `created_by` | UUID FK | Superuser who made the assignment |
| `created_at / updated_at` | TIMESTAMPTZ | Standard audit timestamps |
| `deleted_at` | TIMESTAMPTZ | Soft-delete |

### Rules

- Only one active assignment per `(domain_id, role)` at any time (enforced by unique partial index on `deleted_at IS NULL`).
- Assigning a new user to an already-filled slot soft-deletes the previous holder and creates the new record (upsert semantics).
- Only superusers may create or remove system-user assignments.
- Any authenticated user may read the list (needed for offer-generation pipelines).
- No `organization_id` — this is global platform configuration, not tenant-scoped.
- Resolve the domain for the delivery location first, then look up the assignment for that domain.
  If no domain-specific assignment exists, fall back to the country-level domain assignment.

---

## 6. Files (Polymorphic Upload Storage)

All uploaded files — product photos, PDFs, signatures, waybills, identity documents, etc. — are stored as rows in a single `files` table using a polymorphic (`fileable_type` / `fileable_id`) association. Never store file paths directly on entity tables; always create a `files` record.

### Structure (`files` table)

| Column | Type | Notes |
|---|---|---|
| `id` | UUID | PK |
| `created_by` | UUID FK | User who uploaded the file |
| `fileable_type` | VARCHAR | The entity table name the file belongs to (e.g. `'users'`, `'products'`, `'orders'`) |
| `fileable_id` | UUID | The PK of the owning entity row |
| `type` | VARCHAR | `FileType` enum value (see below) — distinguishes purpose within the same owner |
| `name` | VARCHAR | Original filename as uploaded by the client |
| `absolute_path` | TEXT | Full S3/MinIO URL (e.g. `https://…/bucket/path/file.jpg`) |
| `relative_path` | TEXT | Path within the bucket (e.g. `uploads/users/uuid/signature.png`) |
| `size` | BIGINT | File size in bytes — required, set from the upload response |
| `mime_type` | VARCHAR | MIME type (e.g. `image/png`, `application/pdf`) — required |
| `created_at / updated_at` | TIMESTAMPTZ | Standard audit timestamps |
| `deleted_at` | TIMESTAMPTZ | Soft-delete |

No `organization_id` — ownership is derived through the `fileable_type` / `fileable_id` link.

### FileType enum

All valid `type` values are defined as a Rust enum (`FileType`) and stored as snake-case strings in the DB. Add new variants here when introducing a new upload use-case.

| Variant | DB value | Owner (`fileable_type`) | Purpose |
|---|---|---|---|
| `FileType::UserSignature` | `user-signature` | `users` | A user's signature image |
| `FileType::UserInitials` | `user-initials` | `users` | A user's initials image |
| `FileType::UserProofOfFunds` | `user-proof-of-funds` | `users` | Proof-of-funds document |
| `FileType::ProductImage` | `product-image` | `products` | Catalogue photo for a product |
| `FileType::OrderAttachment` | `order-attachment` | `orders` | Proof of payment, waybill, proof of delivery |

`ProductImage` and `OrderAttachment` have no owning table yet — they are the file types §3 will
need. The `files` table is polymorphic, so the variants can exist ahead of the tables.

Add new variants to the enum and this table whenever a new upload type is introduced. Never use a free-form string.

### Rules

- Files are uploaded to MinIO (S3-compatible). The service layer handles the upload and returns both paths before writing the `files` row.
- Soft-delete only — never hard-delete a file row. The object in MinIO may be retained or expired separately.
- When querying files for an entity, always filter `WHERE fileable_type = $1 AND fileable_id = $2 AND deleted_at IS NULL`.
- A single entity can have multiple files of the same type (e.g. multiple product photos). When exactly one is allowed (e.g. a user's current signature), enforce it at the service layer or with a partial unique index: `CREATE UNIQUE INDEX … ON files (fileable_type, fileable_id, type) WHERE deleted_at IS NULL`.

---

## 7. Integrations

An **Integration** is a provider connection Imacals uses to send or verify email — order
confirmations, dispatch notices and delivery updates today, marketing sends later. Integrations are
platform-level: they belong to the `imacals` organization and a domain, not to individual tenants.

### The runtime-configuration rule (load-bearing)

Environment variables **seed** integrations; they are never the source of truth.

1. On first boot `integration_seed` copies `MAIL_*` / `SMTP_*` / `MAILGUN_*` / … out of the
   environment into `integrations` + `attributes` rows. Each provider is skipped when its slug
   already exists, so the seed is idempotent across restarts.
2. From then on, nothing reads those variables again. `IntegrationResolverService` reads the
   database on **every** use, so editing credentials or switching providers in the dashboard
   changes behaviour on the next send — no redeploy, no restart, no `.env` edit.
3. Consequence: **editing an env var after the first boot has no effect.** Edit the integration.

Nothing caches resolved credentials in process memory. A cache would reintroduce exactly the
restart-to-apply problem this design removes.

### Provider types and categories

`integration_category` is always derived from `integration_type` by the service — a client cannot
file a row under the wrong family. Providers within a category are interchangeable.

| Category | Type | Value | Required credential fields |
|---|---|---|---|
| Email | Log | `log` | `LOG_FROM_EMAIL` |
| Email | SMTP Relay | `smtp` | `SMTP_HOST`, `SMTP_PORT`, `SMTP_FROM_EMAIL` (optional: `SMTP_USERNAME`, `SMTP_PASSWORD`†, `SMTP_FROM_NAME`, `SMTP_USE_TLS`) |
| Email | Mailgun | `mailgun` | `MAILGUN_API_KEY`†, `MAILGUN_DOMAIN`, `MAILGUN_FROM_EMAIL` (optional: `MAILGUN_REGION`, `MAILGUN_FROM_NAME`, `MAILGUN_REPLY_TO`) |
| Email | Mailchimp | `mailchimp` | `MAILCHIMP_API_KEY`†, `MAILCHIMP_FROM_EMAIL`, `MAILCHIMP_FROM_NAME` |
| Email | Gmail | `google` | `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`†, `GOOGLE_REFRESH_TOKEN`†, `GOOGLE_FROM_EMAIL` |
| Email | Outlook | `outlook` | `OUTLOOK_CLIENT_ID`, `OUTLOOK_CLIENT_SECRET`†, `OUTLOOK_TENANT_ID`, `OUTLOOK_USER_ID`, `OUTLOOK_USERNAME` (optional: `OUTLOOK_CLIENT_STATE`†) |
| Email Validation | ZeroBounce | `zero-bounce` | `ZEROBOUNCE_API_KEY`† |
| Other | Custom | `custom` | Free-form — any key/value pairs the user defines |

† encrypted at rest.

Rules:

- **`Log` is the zero-config fallback.** It writes the message to the API log instead of sending,
  needs no credentials, and is seeded first — so a fresh install always has one working sender and
  never starts delivering to real inboxes by accident.
- **Email Validation is deliberately a separate category.** A verifier selected as the sender would
  break every campaign send, so `zero-bounce` can never be resolved as a provider for Email.
- The field template for each type lives in `utilities/integration_type_defs.rs` and is served by
  `GET /integrations/provider-types`. The dashboard renders its credential forms from that response
  rather than keeping a second copy of the field list in TypeScript.

### One live provider per category

- `is_enabled` marks the provider a category currently uses.
- At most one enabled row per `(organization_id, domain_id, integration_category)` — enforced by a
  partial unique index, not application logic, so two concurrent "make this live" requests cannot
  both win. `other` (the Custom catch-all) is exempt: those rows are free-form config, not
  interchangeable providers.
- Switching goes through `PUT /integrations/:id/enabled`, which disables the siblings and enables the
  target in one transaction — no window where two providers look live.
- On create, an explicit `is_enabled` wins; otherwise a row goes live only when its category has no
  provider yet. That is what keeps adding a second provider from tripping the unique index.

### Credentials (Attributes)

- Credentials are stored in the polymorphic `attributes` table with `attributeable_type = 'integrations'`.
- Fields marked `is_encrypted = true` are stored as AES-256-GCM ciphertext (base64-encoded
  nonce‖ciphertext). The encryption key is derived by SHA-256 of `APP_SECRET`.
- **Create flow**: credentials are submitted inline with the integration in a single atomic POST. The
  service validates required fields, opens a transaction, inserts the integration row, then inserts
  each attribute (encrypting secrets before insert), then commits. A rejected create leaves nothing
  behind.
- **Edit flow**: attributes are managed individually via `PUT /attributes/:id`. Encryption state is
  read from the existing record if the request body omits `is_encrypted`.
- **Encrypted values are never returned.** `GET /integrations/:id/attributes` returns `value: null`
  for encrypted attributes — not even the ciphertext. Secrets can be overwritten, never read back.

### Access control

- `GET /integrations`, `GET /integrations/provider-types`, `GET /integrations/:id`,
  `GET /integrations/:id/attributes` — any authenticated user.
- `POST`, `PUT`, `DELETE` on integrations and attributes, and `PUT /integrations/:id/enabled` —
  superusers only.

### Dashboard

The integrations page is where sign-in lands (`/integrations`): no order confirmation can leave the
system until a sending provider is configured. It groups rows by category, names the live sender at the top, warns
when no sender is live, and switches providers through the enable endpoint.

---

*If a rule is missing or ambiguous, ask for clarification — do not assume.*
