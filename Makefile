# scaffolder — developer task runner
# Run `make` or `make help` to list targets.

# Package version, read from Cargo.toml. Override on the CLI for `bump`,
# e.g. `make bump VERSION=0.2.0`.
VERSION := $(shell grep -m1 '^version = ' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')

# Semver components of the current version, used by bump-{patch,minor,major}.
_MAJOR := $(shell echo $(VERSION) | sed -E 's/^([0-9]+)\.([0-9]+)\.([0-9]+).*/\1/')
_MINOR := $(shell echo $(VERSION) | sed -E 's/^([0-9]+)\.([0-9]+)\.([0-9]+).*/\2/')
_PATCH := $(shell echo $(VERSION) | sed -E 's/^([0-9]+)\.([0-9]+)\.([0-9]+).*/\3/')

# Extra args forwarded to `make run`, e.g. `make run ARGS="new typescript-node demo"`.
ARGS ?=

# Install location for `make install` / `uninstall`. Matches install.sh's default;
# override like `make install BINDIR=$$HOME/.local/bin`.
BINDIR ?= /usr/local/bin

.DEFAULT_GOAL := help
.PHONY: help build run test fmt fmt-check lint check install uninstall dist \
	prepare commit ship bump bump-patch bump-minor bump-major release clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

build: ## Build an optimized release binary (target/release/scaffolder)
	cargo build --release

run: ## Run locally: make run ARGS="new typescript-node demo"
	cargo run -- $(ARGS)

test: ## Run unit + integration tests
	cargo test

fmt: ## Format the code
	cargo fmt --all

fmt-check: ## Check formatting without writing
	cargo fmt --all --check

lint: ## Lint with clippy (warnings are errors)
	cargo clippy --all-targets -- -D warnings

check: fmt-check lint test ## Run everything CI runs (fmt + clippy + tests)

install: build ## Build & install to /usr/local/bin (override: BINDIR=...)
	@if [ -w "$(BINDIR)" ] || { [ ! -e "$(BINDIR)" ] && mkdir -p "$(BINDIR)" 2>/dev/null; }; then \
		install -m 0755 target/release/scaffolder "$(BINDIR)/scaffolder"; \
	else \
		echo "elevated permission needed to write $(BINDIR)"; \
		sudo install -m 0755 target/release/scaffolder "$(BINDIR)/scaffolder"; \
	fi
	@echo "→ installed $$("$(BINDIR)/scaffolder" --version 2>/dev/null || echo scaffolder) to $(BINDIR)/scaffolder"

uninstall: ## Remove scaffolder from /usr/local/bin (override: BINDIR=...)
	@if [ -w "$(BINDIR)" ]; then rm -f "$(BINDIR)/scaffolder"; else sudo rm -f "$(BINDIR)/scaffolder"; fi
	@echo "→ removed $(BINDIR)/scaffolder"

dist: ## Build local release artifacts via cargo-dist (archives in target/distrib)
	dist build --artifacts=local

prepare: ## Pre-commit prep: autofix lints, format, type-check
	cargo clippy --fix --allow-dirty --allow-staged --all-targets
	cargo fmt --all
	cargo check --all-targets

commit: ## Smart-commit all changes via Claude Code + git-commit skill
	claude --model claude-sonnet-4-6 --effort high -p "使用 git-commit Skill 提交下**所有**代码，注意格式和换行"

ship: ## prepare, then smart-commit everything
	$(MAKE) prepare
	@$(MAKE) commit

bump-patch: ## Bump the patch version (x.y.Z)
	@$(MAKE) --no-print-directory bump VERSION=$(_MAJOR).$(_MINOR).$(shell expr $(_PATCH) + 1)

bump-minor: ## Bump the minor version (x.Y.0)
	@$(MAKE) --no-print-directory bump VERSION=$(_MAJOR).$(shell expr $(_MINOR) + 1).0

bump-major: ## Bump the major version (X.0.0)
	@$(MAKE) --no-print-directory bump VERSION=$(shell expr $(_MAJOR) + 1).0.0

bump: ## Set an explicit version: make bump VERSION=0.2.0
	@test -n "$(VERSION)" || { echo "usage: make bump VERSION=x.y.z"; exit 1; }
	@echo "$(VERSION)" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+' || { echo "error: VERSION must look like x.y.z"; exit 1; }
	@if command -v cargo-set-version >/dev/null 2>&1; then \
		cargo set-version "$(VERSION)"; \
	else \
		sed -i.bak -E 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml && rm -f Cargo.toml.bak; \
		cargo update -w >/dev/null 2>&1 || true; \
	fi
	@echo "→ version is now $(VERSION). Next: review, 'git commit -am \"chore: release v$(VERSION)\"', then 'make release'."

release: ## Tag the current version and push it (triggers the release CI)
	@git diff --quiet && git diff --cached --quiet || { echo "error: working tree not clean — commit first"; exit 1; }
	@git rev-parse "v$(VERSION)" >/dev/null 2>&1 && { echo "error: tag v$(VERSION) already exists"; exit 1; } || true
	git tag "v$(VERSION)"
	git push origin "v$(VERSION)"
	@echo "→ pushed tag v$(VERSION); cargo-dist will build all platforms and publish the GitHub Release."

clean: ## Remove build artifacts
	cargo clean
