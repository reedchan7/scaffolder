PM := {{pm}}

.DEFAULT_GOAL := help
.PHONY: help install dev build start typecheck test lint format check

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

install: ## Install dependencies
	$(PM) install

dev: ## Run in watch mode
	$(PM) run dev

build: ## Compile to dist/
	$(PM) run build

start: ## Run the app (from source)
	$(PM) run start

typecheck: ## Type-check without emitting
	$(PM) run typecheck

test: ## Run tests
	$(PM) test

lint: ## Lint code
	$(PM) run lint

format: ## Format with Prettier
	$(PM) run format

check: ## Format check + lint + typecheck + test
	$(PM) run check
