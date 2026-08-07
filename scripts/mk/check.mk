# Type/build checks + cargo lock recovery.

check-api:
	@docker exec imacals-api bash -c "CARGO_TARGET_DIR=/tmp/cargo-check cargo check"

check-dashboard:
	@docker exec imacals-dashboard npm run build

check-web:
	@docker exec imacals-web npm run build

# "Blocking waiting for file lock on artifact directory" means another cargo/rustc
# (usually rust-analyzer's background check, or a hung/killed build) holds the lock on
# target/. This finds those processes, stops them, and clears any stale .cargo-lock —
# on the host AND inside the api container. Scoped to this project so unrelated cargo
# work on the host is left alone.
free-cargo:
	@echo "==> Host cargo/rustc processes locking this project:"
	@ps -eo pid,etime,cmd | grep -E 'cargo|rustc' | grep imacals | grep -v grep || echo "  (none)"
	@pkill -f 'imacals/imacals-api' 2>/dev/null && echo "==> stopped host processes" || echo "==> nothing to stop on host"
	@docker exec imacals-api bash -c "pkill -f cargo; pkill -f rustc" 2>/dev/null && echo "==> stopped container processes" || echo "==> nothing to stop in container"
	@rm -f imacals-api/target/debug/.cargo-lock 2>/dev/null || true
	@docker exec imacals-api bash -c "rm -f target/debug/.cargo-lock /tmp/cargo-*/debug/.cargo-lock" 2>/dev/null || true
	@echo "==> cargo lock cleared — re-run your build."

.PHONY: check-api check-dashboard check-web free-cargo
