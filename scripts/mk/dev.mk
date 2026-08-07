# Dev stack — docker-compose up/run for the local environment.

dev:
	docker-compose up --build

api:
	docker-compose up imacals-api $(filter-out $@,$(MAKECMDGOALS))

down:
	@docker-compose down

dashboard:
	docker-compose up imacals-dashboard

web:
	docker-compose up imacals-web

.PHONY: dev api down dashboard web
