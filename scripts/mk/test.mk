# Test suites — API (Rust), dashboard (Playwright).

# Run the full test suite. Brings the stack up first so cargo test has a live DB + MinIO.
test: test-up test-api test-dashboard test-web

# Start the containers the tests depend on.
# `imacals-minio-init` is a one-shot bucket-creator that exits 0 — start it without --wait so
# its clean exit isn't flagged as failure. Then --wait on imacals-api, which depends on
# db + minio being healthy.
test-up:
	@docker-compose up -d imacals-minio-init
	@docker-compose up -d --wait imacals-api

# Run the Rust API test suite (unit + #[sqlx::test] integration tests) inside the api container.
# #[sqlx::test] creates a fresh per-test DB and applies migrations itself, so we don't pre-run
# migrations against the dev DB — that fails when a migration was edited in place (the dev-phase rule).
test-api: test-up
	@docker exec imacals-api bash -c "CARGO_TARGET_DIR=/tmp/cargo-test cargo test"

# Run Playwright e2e tests in the official Playwright container (browser + system libs
# baked in — the bare WSL host is missing libnspr4/libnss3/etc). Reuses the host
# node_modules (run `cd imacals-dashboard && npm install` once), mocks all API calls, and
# starts its own Vite dev server in-container. HOST_UID/HOST_GID make the container write
# the Vite cache as you, not root. `run --rm` streams output and exits with the suite's code.
test-dashboard:
	@HOST_UID=$$(id -u) HOST_GID=$$(id -g) docker-compose run --rm imacals-dashboard-e2e

# Same runner, but pass through extra Playwright args, e.g.:
#   make test-dashboard-spec ARGS="login.spec.ts"
test-dashboard-spec:
	@HOST_UID=$$(id -u) HOST_GID=$$(id -g) docker-compose run --rm imacals-dashboard-e2e \
		npx playwright test --reporter=line --output=/tmp/test-results $(ARGS)

# Watch the browser drive the tests in a real window (headed). Forwards the WSLg X socket
# into the container so Chromium renders on your desktop. Single worker so windows don't
# stack. Scope to a spec to keep it watchable, e.g.:
#   make test-dashboard-watch ARGS="login.spec.ts"
test-dashboard-watch:
	@HOST_UID=$$(id -u) HOST_GID=$$(id -g) docker-compose run --rm \
		-e DISPLAY=$$DISPLAY -e XDG_RUNTIME_DIR=/tmp \
		-v /tmp/.X11-unix:/tmp/.X11-unix -v /mnt/wslg:/mnt/wslg \
		imacals-dashboard-e2e \
		npx playwright test --headed --workers=1 --reporter=line --output=/tmp/test-results $(ARGS)

# Interactive Playwright UI runner — host machine, kept out of `make test` because it blocks.
# Needs host browser deps: cd imacals-dashboard && npx playwright install --with-deps
test-dashboard-ui:
	cd imacals-dashboard && npm run test:e2e:ui

# Open the HTML report from the last run.
test-dashboard-report:
	cd imacals-dashboard && npm run test:e2e:report

# Storefront e2e — same runner contract as test-dashboard. Run `cd imacals-web && npm install`
# once so the container can reuse the host node_modules.
test-web:
	@HOST_UID=$$(id -u) HOST_GID=$$(id -g) docker-compose run --rm imacals-web-e2e

# Pass through extra Playwright args, e.g.:
#   make test-web-spec ARGS="cart.spec.ts"
test-web-spec:
	@HOST_UID=$$(id -u) HOST_GID=$$(id -g) docker-compose run --rm imacals-web-e2e \
		npx playwright test --reporter=line --output=/tmp/test-results $(ARGS)

test-web-ui:
	cd imacals-web && npm run test:e2e:ui

test-web-report:
	cd imacals-web && npm run test:e2e:report

.PHONY: test test-up test-api test-dashboard test-dashboard-spec test-dashboard-watch \
        test-dashboard-ui test-dashboard-report \
        test-web test-web-spec test-web-ui test-web-report
