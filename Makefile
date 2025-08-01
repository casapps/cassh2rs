.PHONY: all build test clean install release help check fmt lint doc
.DEFAULT_GOAL := help

# Project variables
PROJECT_NAME = cassh2rs
VERSION = $(shell grep '^version' Cargo.toml | sed 's/.*= "\(.*\)"/\1/')
CARGO = cargo
CROSS = cross
GH = gh

# Build targets
TARGETS = \
	x86_64-unknown-linux-gnu \
	x86_64-unknown-linux-musl \
	aarch64-unknown-linux-gnu \
	x86_64-apple-darwin \
	aarch64-apple-darwin \
	x86_64-pc-windows-gnu

# Colors for output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[1;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

help: ## Show this help message
	@echo "$(BLUE)cassh2rs Makefile$(NC)"
	@echo "$(YELLOW)Usage:$(NC)"
	@echo "  make [target]"
	@echo ""
	@echo "$(YELLOW)Targets:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2}'

all: fmt lint test build ## Run everything (format, lint, test, build)

check: ## Check if the project compiles
	@echo "$(BLUE)Checking project...$(NC)"
	$(CARGO) check --all-features

fmt: ## Format code
	@echo "$(BLUE)Formatting code...$(NC)"
	$(CARGO) fmt

lint: ## Run clippy linter
	@echo "$(BLUE)Running clippy...$(NC)"
	$(CARGO) clippy -- -D warnings

test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	$(CARGO) test --all-features

doc: ## Generate documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	$(CARGO) doc --all-features --no-deps
	@echo "$(GREEN)Documentation available at:$(NC) target/doc/$(PROJECT_NAME)/index.html"

build: ## Build for current platform (debug)
	@echo "$(BLUE)Building $(PROJECT_NAME) (debug)...$(NC)"
	$(CARGO) build --all-features
	@echo "$(GREEN)Build complete!$(NC)"

build-release: ## Build for current platform (release)
	@echo "$(BLUE)Building $(PROJECT_NAME) (release)...$(NC)"
	$(CARGO) build --release --all-features
	@echo "$(GREEN)Release build complete!$(NC)"

install: build-release ## Install locally
	@echo "$(BLUE)Installing $(PROJECT_NAME)...$(NC)"
	$(CARGO) install --path .
	@echo "$(GREEN)Installed to:$(NC) $$HOME/.cargo/bin/$(PROJECT_NAME)"

uninstall: ## Uninstall locally
	@echo "$(BLUE)Uninstalling $(PROJECT_NAME)...$(NC)"
	$(CARGO) uninstall $(PROJECT_NAME)

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning...$(NC)"
	$(CARGO) clean
	rm -rf dist/
	@echo "$(GREEN)Clean complete!$(NC)"

# Cross-compilation targets
dist: ## Build all release binaries
	@echo "$(BLUE)Building release binaries for all platforms...$(NC)"
	@mkdir -p dist
	@for target in $(TARGETS); do \
		echo "$(YELLOW)Building for $$target...$(NC)"; \
		if [ "$$target" = "x86_64-apple-darwin" ] || [ "$$target" = "aarch64-apple-darwin" ]; then \
			if [ "$$(uname)" = "Darwin" ]; then \
				$(CARGO) build --release --target $$target --all-features || true; \
			else \
				echo "$(YELLOW)Skipping $$target (requires macOS)$(NC)"; \
			fi; \
		else \
			if command -v $(CROSS) >/dev/null 2>&1; then \
				$(CROSS) build --release --target $$target --all-features || true; \
			else \
				$(CARGO) build --release --target $$target --all-features || true; \
			fi; \
		fi; \
		if [ -f "target/$$target/release/$(PROJECT_NAME)" ]; then \
			cp target/$$target/release/$(PROJECT_NAME) dist/$(PROJECT_NAME)_$${target%%-*}_$${target##*-} || true; \
		fi; \
		if [ -f "target/$$target/release/$(PROJECT_NAME).exe" ]; then \
			cp target/$$target/release/$(PROJECT_NAME).exe dist/$(PROJECT_NAME)_$${target%%-*}_$${target##*-}.exe || true; \
		fi; \
	done
	@echo "$(GREEN)Distribution builds complete! Binaries in dist/$(NC)"
	@ls -la dist/

compress-dist: dist ## Compress distribution binaries
	@echo "$(BLUE)Compressing binaries...$(NC)"
	@cd dist && for file in *; do \
		if [ -f "$$file" ]; then \
			if command -v upx >/dev/null 2>&1; then \
				upx --best "$$file" || true; \
			fi; \
			tar -czf "$$file.tar.gz" "$$file"; \
			echo "$(GREEN)Compressed: $$file.tar.gz$(NC)"; \
		fi; \
	done

# GitHub Release
check-gh: ## Check GitHub CLI is available
	@command -v $(GH) >/dev/null 2>&1 || { echo "$(RED)Error: GitHub CLI (gh) not found. Install from: https://cli.github.com/$(NC)"; exit 1; }
	@$(GH) auth status >/dev/null 2>&1 || { echo "$(RED)Error: Not authenticated with GitHub. Run: gh auth login$(NC)"; exit 1; }

release-notes: ## Generate release notes
	@echo "# Release v$(VERSION)" > RELEASE_NOTES.md
	@echo "" >> RELEASE_NOTES.md
	@echo "## What's New" >> RELEASE_NOTES.md
	@git log --pretty=format:"- %s" $$(git describe --tags --abbrev=0 2>/dev/null || echo "")..HEAD >> RELEASE_NOTES.md 2>/dev/null || echo "- Initial release" >> RELEASE_NOTES.md
	@echo "" >> RELEASE_NOTES.md
	@echo "" >> RELEASE_NOTES.md
	@echo "## Installation" >> RELEASE_NOTES.md
	@echo '```bash' >> RELEASE_NOTES.md
	@echo 'curl -fsSL https://raw.githubusercontent.com/casapps/cassh2rs/main/scripts/install.sh | bash' >> RELEASE_NOTES.md
	@echo '```' >> RELEASE_NOTES.md
	@echo "$(GREEN)Release notes generated: RELEASE_NOTES.md$(NC)"

create-release: check-gh release-notes compress-dist ## Create GitHub release (draft)
	@echo "$(BLUE)Creating GitHub release v$(VERSION)...$(NC)"
	@$(GH) release create v$(VERSION) \
		--draft \
		--title "$(PROJECT_NAME) v$(VERSION)" \
		--notes-file RELEASE_NOTES.md \
		dist/*.tar.gz
	@echo "$(GREEN)Draft release created! Review and publish at:$(NC)"
	@echo "https://github.com/casapps/$(PROJECT_NAME)/releases"

release: check-gh test lint compress-dist release-notes ## Full release process
	@echo "$(BLUE)Starting full release process for v$(VERSION)...$(NC)"
	@echo "$(YELLOW)This will:$(NC)"
	@echo "  1. Run all tests"
	@echo "  2. Build binaries for all platforms"
	@echo "  3. Create compressed archives"
	@echo "  4. Create a draft GitHub release"
	@echo ""
	@read -p "Continue? [y/N] " -n 1 -r; \
	echo ""; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		$(MAKE) create-release; \
	else \
		echo "$(YELLOW)Release cancelled$(NC)"; \
	fi

# Development helpers
watch: ## Watch for changes and rebuild
	@echo "$(BLUE)Watching for changes...$(NC)"
	$(CARGO) watch -x build

bench: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	$(CARGO) bench

coverage: ## Generate test coverage
	@echo "$(BLUE)Generating test coverage...$(NC)"
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		cargo tarpaulin --out Html --output-dir coverage; \
		echo "$(GREEN)Coverage report: coverage/tarpaulin-report.html$(NC)"; \
	else \
		echo "$(YELLOW)cargo-tarpaulin not installed. Install with:$(NC)"; \
		echo "  cargo install cargo-tarpaulin"; \
	fi

# Shell completions
completions: build ## Generate shell completions
	@echo "$(BLUE)Generating shell completions...$(NC)"
	@mkdir -p completions
	@./target/debug/$(PROJECT_NAME) completions bash > completions/$(PROJECT_NAME).bash
	@./target/debug/$(PROJECT_NAME) completions zsh > completions/$(PROJECT_NAME).zsh
	@./target/debug/$(PROJECT_NAME) completions fish > completions/$(PROJECT_NAME).fish
	@echo "$(GREEN)Completions generated in completions/$(NC)"

# Quick commands
dev: fmt lint test ## Quick development check (format, lint, test)

push: dev ## Run checks before git push
	@echo "$(GREEN)All checks passed! Ready to push.$(NC)"

# Version management
version: ## Show current version
	@echo "$(PROJECT_NAME) v$(VERSION)"

bump-patch: ## Bump patch version (0.0.X)
	@cargo bump patch
	@echo "$(GREEN)Version bumped to $$(grep '^version' Cargo.toml | sed 's/.*= "\(.*\)"/\1/')$(NC)"

bump-minor: ## Bump minor version (0.X.0)
	@cargo bump minor
	@echo "$(GREEN)Version bumped to $$(grep '^version' Cargo.toml | sed 's/.*= "\(.*\)"/\1/')$(NC)"

bump-major: ## Bump major version (X.0.0)
	@cargo bump major
	@echo "$(GREEN)Version bumped to $$(grep '^version' Cargo.toml | sed 's/.*= "\(.*\)"/\1/')$(NC)"