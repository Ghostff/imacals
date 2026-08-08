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

What exists today is a **small platform core** and nothing else. Five tables:

```
users  roles  permissions  role_permissions  user_permissions
```

That is auth, staff accounts, and a role/permission model. The ecommerce domain itself — products,
stock, orders, delivery, payments — is **not built yet**; see §3.

Everything inherited from the property-renovation codebase this started as has been removed:
trades, materials, spaces, user trades/businesses, organizations and multi-tenancy, domains,
system users, polygons/zones, geo reference tables, bank accounts, user documents, file/object
storage and the email-provider integrations layer — along with the ~200 permission rows that named
features which never existed.

**No file storage and no mail.** Product images and order-confirmation email both need them, so
both come back with the feature that needs them — built to that need rather than to the
email-campaign platform this inherited.

---

## 1. Roles & Permissions

Imacals is **one business, not a multi-tenant platform**. There is no organization layer: a
permission check needs only the user.

### Roles (`roles` table)

A role is a named bundle of permissions, linked through `role_permissions`. A user carries one
role via `users.role_id`.

| Role | Key | Description |
|---|---|---|
| Super Admin | `super-admin` | Unrestricted access. Never assigned to a person — the real bypass is `users.is_superuser`. |
| Admin | `admin` | Full access: staff, catalogue, orders, settings. |
| Order Desk | `order-desk` | Takes phone orders and enters them on the customer's behalf. |
| Warehouse | `warehouse` | Picks and packs orders in the Aba warehouse; adjusts stock. |
| Dispatch | `dispatch` | Assigns orders to vehicles and routes; confirms delivery. |
| Accounts | `accounts` | Reconciles payments and issues refunds. |

### Permissions (`permissions`, `user_permissions`)

- **Grants live on the user**, in `user_permissions` — not on a role link. A role is a *template*.
- Assigning a role calls `PermissionRepository::sync_from_role`, which soft-deletes the user's
  current grants and writes the role's bundle in one transaction.
- **Sync is explicit.** Editing a role later does not retro-fit people already on it. That is
  deliberate: it keeps a hand-picked grant from being silently wiped by an unrelated role edit.
- **Superusers bypass every gate** (`users.is_superuser`).
- Soft-deleting a user cascades to their grants via `trg_soft_delete_user_permissions_on_user_delete`.

Only permissions something actually checks are seeded — currently `users.*` and `roles.*`, and
that is all. A feature seeds its own permissions in its own migration. Do not add rows for tables
that do not exist yet.

### Gates

```rust
crate::gate!(&app.pool, &user, "users.view");          // 403 and return if missing
crate::gate_any!(&app.pool, &user, &["a", "b"]);       // 403 unless at least one is held
crate::gate_all!(&app.pool, &user, &["a", "b"]);       // 403 unless all are held
if crate::can!(&app.pool, &user, "users.view") { … }   // bool, no early return
```

---

## 2. Accounts

| Path | Rule |
|---|---|
| **Self-registration** (`POST /auth/register`) | Creates an account with **no role and no permissions**. An administrator assigns a role before the person can do anything. The schema has no role field at all, so a caller cannot grant itself access. |
| **Admin creates staff** (`POST /users`) | Requires `users.create`. `role_id` is required; the role's bundle is synced onto the new user. |
| **Changing a role or extra grants** | Requires `users.manage-permissions`, not merely `users.update` — changing what someone can do is a bigger act than fixing their surname. |

`is_internal` marks an Imacals employee as opposed to a customer account. It no longer confers
cross-tenant visibility (there are no tenants); it is a plain flag.

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
| `categories` | Self-referential for sub-categories. |
| `products` | `slug` unique where `deleted_at IS NULL`. |
| `stock_levels` | Per `(product_id, warehouse_id)`. Never a bare column on `products`. |
| `customers` | A buyer. May exist without a `users` row — phone orders create one from a name and number. |
| `customer_addresses` | Multiple per customer; one default. |
| `orders` | Carries `channel` (`online` \| `phone`), `reference`, `status`, totals, warehouse. |
| `order_items` | Line snapshot: unit price copied at order time so later price changes never rewrite history. |
| `order_status_history` | Append-only. One row per transition, with actor and timestamp. |
| `delivery_zones` | Ties a delivery area to a tariff. Keyed off the destination state/town on the order — the polygon tables are gone. |
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
*If a rule is missing or ambiguous, ask for clarification — do not assume.*
