# imacals

Ecommerce and distribution for Nigeria, run out of a base warehouse in **Aba, Abia State**.
Customers order online or by phone; we pick, deliver and distribute.

Three apps: a Rust/Actix API, a Vue 3 back-office dashboard (`dashboard.imacals.com`) and a Vue 3
customer storefront (`imacals.com`). PostgreSQL behind them.

---

## Quick start

```bash
cp imacals-api/.env.example       imacals-api/.env
cp imacals-dashboard/.env.example imacals-dashboard/.env
cp imacals-web/.env.example       imacals-web/.env
make dev
```

`make dev` builds and starts everything and applies migrations on boot. First run compiles the Rust
API, so give it a few minutes.

### Sign in

The migrations seed a platform superuser:

| | |
|---|---|
| **Email** | `admin@imacals.com` |
| **Password** | `P@ssw0rd!` |

Dev credentials — change them before any deployment that is reachable from outside your machine.

### Where things run

| Service | URL | Env var |
|---|---|---|
| Storefront (`imacals.com`) | http://localhost:5175 | `WEB_HOST_PORT` |
| Dashboard (`dashboard.imacals.com`) | http://localhost:5174 | `DASHBOARD_HOST_PORT` |
| API | http://localhost:3032 | `APP_PORT` |
| Postgres | localhost:5437 | `DB_HOST_PORT` |

Host-port defaults deliberately avoid the usual ones (5432, 5173) so a sibling project's stack can
run at the same time. In-container hostnames and ports never change.

There is **no object storage and no mail service** — nothing uploads or sends yet. Add them back
alongside the feature that needs them (product images, order confirmation email), sized to that
need.

---

## The two front ends

| | `imacals-dashboard` | `imacals-web` |
|---|---|---|
| Audience | Staff — warehouse, dispatch, order desk | Customers |
| Domain | `dashboard.imacals.com` | `imacals.com` |
| Auth | Required; every route behind a token | Public; token only for a signed-in customer |
| Port (dev) | 5174 | 5175 |

They are separate Vite apps on purpose: a storefront deploy must never ship back-office code to the
public, and the two scale independently. They share the design tokens in `docs/design.md` — the
token block at the top of each app's `src/style.css` is the same contract, so keep them in step.

### Storefront status

The storefront ships against catalogue and order endpoints that **do not exist yet**
(`/catalog/products`, `/catalog/categories`, `/orders`). Until they land, `VITE_USE_PREVIEW_CATALOG=true`
serves the sample catalogue in `imacals-web/src/services/catalog.ts` so the layout, cart and
checkout flow can be exercised end to end — and a banner tells visitors the products are samples.
Flip it to `false` and delete the `PREVIEW_CATALOG` block once the API is built.

---

## Commands

`make help` lists every target. The root Makefile is thin — one group file per family under
`scripts/mk/`.

| | |
|---|---|
| `make dev` | start the whole stack |
| `make down` | stop it |
| `make web` / `make dashboard` / `make api` | start one service |
| `make check-api` / `make check-dashboard` / `make check-web` | build checks — run before calling anything done |
| `make test-api` / `make test-dashboard` / `make test-web` | Rust suite / Playwright e2e |
| `make run-migration` / `make rollback-migration` | migrations |
| `make prepare` | regenerate sqlx offline query metadata |
| `make clean-db` | drop the postgres volume (destroys all local data) |
| `make free-cargo` | clear a stuck cargo artifact lock |

---

## What's in the box

The platform core, and nothing more — five tables:

```
users  roles  permissions  role_permissions  user_permissions
```

That is auth, staff accounts, and a role/permission model. Nothing else.

Everything inherited from the property-renovation codebase this started as has been stripped:
multi-tenancy, domains, geo reference tables, polygons/zones, trades, materials, spaces, bank
accounts, user documents, file/object storage, the email-provider integrations layer, and ~200
permission rows naming features that never existed.

The ecommerce domain — products, stock, orders, delivery, payments — is next; the shape it should
take is specified in `docs/business_logic.md §3`.

There is **no organization or tenant layer**. Imacals is one business; a permission check needs
only the user.

---

## Layout

```
imacals-api/          Rust API (Actix, sqlx) — controllers → services → repositories
imacals-dashboard/    Vue 3 + Vite back office        (dashboard.imacals.com)
imacals-web/          Vue 3 + Vite customer storefront (imacals.com)
docs/                 business_logic.md (rules) · design.md (tokens, components)
scripts/mk/           Makefile command groups
AGENTS.md             conventions — read before writing code
```
