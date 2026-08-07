# Database + sqlx — volume reset, migrations, offline metadata.

clean-db:
	@echo "WARNING: Dropping postgres volume — all DB data will be lost."
	@docker-compose stop imacals-api imacals-db 2>/dev/null || true
	@docker-compose rm -f imacals-db 2>/dev/null || true
	@PROJECT=$${COMPOSE_PROJECT_NAME:-$$(basename $$(pwd))}; \
		docker volume ls -q --filter label=com.docker.compose.project=$$PROJECT --filter label=com.docker.compose.volume=postgres_data | xargs -r docker volume rm

create-migration:
	@docker exec imacals-api bash ./scripts/migration.sh $(filter-out $@,$(MAKECMDGOALS))

run-migration:
	@docker exec imacals-api bash -c "sqlx migrate run --source ./src/migrations"
	@docker exec imacals-api bash -c "CARGO_TARGET_DIR=/tmp/cargo-sqlx cargo sqlx prepare -- --all-targets"

rollback-migration:
	@docker exec imacals-api bash -c "sqlx migrate revert --source ./src/migrations"
	@docker exec imacals-api bash -c "CARGO_TARGET_DIR=/tmp/cargo-sqlx cargo sqlx prepare -- --all-targets"

# --all-targets so query metadata also covers the sqlx macros used inside #[cfg(test)] blocks.
prepare:
	@docker exec imacals-api bash -c "CARGO_TARGET_DIR=/tmp/cargo-sqlx cargo sqlx prepare -- --all-targets"

.PHONY: clean-db create-migration run-migration rollback-migration prepare
