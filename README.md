# imacals

Ecommerce and distribution for Nigeria, run out of a base warehouse in **Aba, Abia State**.
Customers order online or by phone; we pick, deliver and distribute.

Three apps: a Rust/Actix API, a Vue 3 back-office dashboard (`dashboard.imacals.com`) and a Vue 3
customer storefront (`imacals.com`). PostgreSQL, MinIO, Mailpit behind them.

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
| Mail inbox (Mailpit) | http://localhost:8026 | `MAIL_UI_PORT` |
| MinIO console | http://localhost:9003 | `MINIO_CONSOLE_PORT` |
| Postgres | localhost:5437 | `DB_HOST_PORT` |

Host-port defaults deliberately avoid the usual ones (5432/9000/9002/8025) so a sibling project's
stack can run at the same time. In-container hostnames and ports never change.

**Email never leaves your machine in dev.** Every send goes to Mailpit and shows up in the inbox
above.

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

## Configuration lives in the database, not in `.env`

This is the one rule to internalise before adding a provider.

Environment variables **seed** integrations on first boot and are never read again. Credentials then
live in the `integrations` + `attributes` tables, and `IntegrationResolverService` re-reads them on
every use — so changing a provider or its credentials in the dashboard takes effect on the next send
with **no restart**. Editing an env var after the first boot has no effect; edit the integration.

A fresh install comes up with the credential-free **Log** provider live (it writes mail to the API
log instead of sending), so nothing is ever delivered to a real inbox by accident. Configure a real
provider on the Integrations page, then switch to it.

See `docs/business_logic.md §7` for the full rule set.

---

## Optional: FontAwesome Pro

`@fortawesome/pro-solid-svg-icons` comes from a private registry and is an **optional** dependency of
the dashboard: without a token the install skips it and the app runs, with the map toolbar showing
text labels instead of icons. To get the icons, add your token to `imacals-dashboard/.env`:

```
FONTAWESOME_PACKAGE_TOKEN=<your-token>
```

The storefront does not depend on it.

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
