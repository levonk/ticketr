.PHONY: doctor bootstrap profile help clean clean-inner build build-inner debug debug-inner install install-inner lint lint-inner test test-inner run run-inner docker-build docker-run

PROJECT_SLUG :=
NIX := $(shell if [ -x /nix/var/nix/profiles/default/bin/nix ]; then echo /nix/var/nix/profiles/default/bin/nix; else echo nix; fi)
NIX_DEV := $(NIX) develop -c

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

doctor: ## Validate prerequisites and print setup suggestions
	@if ! $(NIX) --version >/dev/null 2>&1; then \
		echo "Error: nix not found in PATH" >&2; \
		echo "Suggestion: install Nix (https://nixos.org/download/)" >&2; \
		exit 1; \
	fi
	@if [ ! -f flake.nix ]; then \
		echo "Error: flake.nix not found (expected in project root)" >&2; \
		exit 1; \
	fi
	@echo "OK: nix + flake.nix present"
	@if command -v direnv >/dev/null 2>&1; then \
		echo "OK: direnv present"; \
		echo "Next: direnv allow"; \
	else \
		echo "Warning: direnv not found"; \
		echo "Suggestion: install direnv (https://direnv.net/)"; \
		echo "Then run: direnv allow"; \
	fi
	@echo "Suggestion: make bootstrap"
	@echo "Optional: make profile (installs toolchain into user profile)"

bootstrap: doctor ## Warm the Nix dev shell for this project (no user env changes)
	@$(NIX) develop -c true

profile: doctor ## Install toolchain into user profile (optional)
	@$(NIX) profile install nixpkgs#rustc nixpkgs#cargo nixpkgs#clippy nixpkgs#rustfmt nixpkgs#gnumake nixpkgs#git


clean: doctor ## Clean build artifacts (via nix dev shell)
	@$(NIX_DEV) $(MAKE) clean-inner

clean-inner: ## Clean build artifacts
	cargo clean


build: doctor bootstrap ## Build the project (via nix dev shell)
	@$(NIX_DEV) $(MAKE) build-inner

build-inner: ## Build the project
	cargo build --release

debug: doctor bootstrap ## Build the project in debug mode (via nix dev shell)
	@$(NIX_DEV) $(MAKE) debug-inner

debug-inner: ## Build the project in debug mode
	cargo build


install: doctor bootstrap ## Install the binary (via nix dev shell)
	@$(NIX_DEV) $(MAKE) install-inner

install-inner: ## Install the binary
	cargo install --path .


lint: doctor bootstrap ## Lint the code using clippy (via nix dev shell)
	@$(NIX_DEV) $(MAKE) lint-inner

lint-inner: ## Lint the code using clippy
	cargo clippy -- -D warnings


test: doctor bootstrap ## Run tests (via nix dev shell)
	@$(NIX_DEV) $(MAKE) test-inner

test-inner: ## Run tests
	cargo test


run: doctor bootstrap ## Run the application (use ARGS="..." to pass arguments) (via nix dev shell)
	@$(NIX_DEV) $(MAKE) run-inner ARGS="$(ARGS)"

run-inner: ## Run the application (use ARGS="..." to pass arguments)
	cargo run -- $(ARGS)

docker-build: ## Build docker image
	docker build -t $(PROJECT_SLUG):latest .

docker-run: ## Run via docker
	docker run --rm -it $(PROJECT_SLUG):latest $(ARGS)
