# Root Makefile — thin. Shared setup, then one include per command group.
# Group files live under scripts/mk/. Add a new group = drop a *.mk there and
# include it below. `make help` lists the groups.

include ./imacals-api/.env
export

.DEFAULT_GOAL := dev

include scripts/mk/dev.mk    # dev / api / down / dashboard
include scripts/mk/db.mk     # clean-db / migrations / prepare
include scripts/mk/check.mk  # check-api / check-dashboard / free-cargo
include scripts/mk/test.mk   # test suites (api / dashboard)

help:
	@echo "Command groups (scripts/mk/*.mk):"
	@echo "  dev      dev  api  down  dashboard  web"
	@echo "  db       clean-db  create-migration  run-migration  rollback-migration  prepare"
	@echo "  check    check-api  check-dashboard  check-web  free-cargo"
	@echo "  test     test  test-up  test-api  test-dashboard[-spec/-watch/-ui/-report]"
	@echo "           test-web[-spec/-ui/-report]"

.PHONY: help
