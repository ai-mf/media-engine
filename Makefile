# AI Media Engine Makefile
# Usage: make <target>

.PHONY: help build release test test-all clean doc lint format fix clippy audit
.PHONY: install uninstall examples verify-examples keygen release-build release-upload
.PHONY: install-mime uninstall-mime install-desktop

# Colors for output
GREEN := \033[0;32m
RED := \033[0;31m
YELLOW := \033[1;33m
RESET := \033[0m

help: ## Show this help message
	@echo "$(GREEN)AI Media Engine — Available Commands$(RESET)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(RESET) %s\n", $$1, $$2}'
	@echo ""

build: ## Build all binaries (debug mode)
	@echo "$(YELLOW)🔨 Building...$(RESET)"
	cargo build --workspace
	@echo "$(GREEN)✅ Build complete$(RESET)"

release: ## Build all binaries (release mode)
	@echo "$(YELLOW)🚀 Building release binaries...$(RESET)"
	cargo build --release --workspace
	@echo "$(GREEN)✅ Release build complete$(RESET)"
	@ls -lh target/release/aimf target/release/aimg target/release/aaud target/release/avid 2>/dev/null || true

test: ## Run all tests
	@echo "$(YELLOW)🧪 Running tests...$(RESET)"
	cargo test --workspace --lib
	@echo "$(GREEN)✅ Tests passed$(RESET)"

test-all: ## Run all tests including integration and doc tests
	@echo "$(YELLOW)🧪 Running all tests...$(RESET)"
	cargo test --workspace --all-targets --all-features
	@echo "$(GREEN)✅ All tests passed$(RESET)"

test-verbose: ## Run tests with verbose output
	cargo test --workspace -- --nocapture

test-integration: ## Run integration tests only
	cargo test --test '*' --workspace

doc: ## Generate documentation
	@echo "$(YELLOW)📚 Generating documentation...$(RESET)"
	cargo doc --no-deps --workspace --document-private-items
	@echo "$(GREEN)✅ Documentation generated at target/doc/aimf_core/index.html$(RESET)"

doc-open: doc ## Generate and open documentation
	open target/doc/aimf_core/index.html 2>/dev/null || \
	xdg-open target/doc/aimf_core/index.html 2>/dev/null || \
	start target/doc/aimf_core/index.html 2>/dev/null || \
	echo "Open target/doc/aimf_core/index.html manually"

lint: ## Run linter
	@echo "$(YELLOW)🔍 Linting...$(RESET)"
	cargo clippy --workspace -- -D warnings
	@echo "$(GREEN)✅ Linting passed$(RESET)"

format: ## Format code
	@echo "$(YELLOW)🎨 Formatting code...$(RESET)"
	cargo fmt --all
	@echo "$(GREEN)✅ Format complete$(RESET)"

fix: ## Automatically fix clippy warnings where possible
	@echo "$(YELLOW)🔧 Auto-fixing clippy warnings...$(RESET)"
	cargo clippy --workspace --fix --allow-dirty --allow-staging
	cargo fmt --all
	@echo "$(GREEN)✅ Fixes applied$(RESET)"

audit: ## Check for security vulnerabilities
	@echo "$(YELLOW)🔒 Security audit...$(RESET)"
	cargo install --force cargo-audit 2>/dev/null || true
	cargo audit
	@echo "$(GREEN)✅ Audit complete$(RESET)"

clean: ## Clean build artifacts
	@echo "$(YELLOW)🧹 Cleaning...$(RESET)"
	cargo clean
	rm -rf target/ docs/build/
	@echo "$(GREEN)✅ Clean complete$(RESET)"

install: release ## Install binaries to ~/.cargo/bin
	@echo "$(YELLOW)📦 Installing binaries...$(RESET)"
	cargo install --path tools/aimf-cli --force
	cargo install --path tools/aaud-cli --force
	cargo install --path tools/aimg-cli --force
	cargo install --path tools/avid-cli --force
	@echo "$(GREEN)✅ Installed: aimf, aaud, aimg, avid$(RESET)"

uninstall: ## Remove installed binaries
	@echo "$(YELLOW)🗑️  Uninstalling...$(RESET)"
	cargo uninstall aimf 2>/dev/null || true
	cargo uninstall aaud 2>/dev/null || true
	cargo uninstall aimg 2>/dev/null || true
	cargo uninstall avid 2>/dev/null || true
	@echo "$(GREEN)✅ Uninstall complete$(RESET)"

examples: ## Build and run all examples
	@echo "$(YELLOW)🎬 Running examples...$(RESET)"
	@echo "  📷 Image example..."
	cargo run --example ai_generate_image || echo "⚠️  Image example failed"
	@echo "  🔊 Audio example..."
	cargo run --example ai_generate_audio || echo "⚠️  Audio example failed"
	@echo "  🎥 Video example..."
	cargo run --example ai_generate_video_simple || echo "⚠️  Video example failed"
	@echo "$(GREEN)✅ Examples complete$(RESET)"

verify-examples: ## Verify the generated examples are valid
	@echo "$(YELLOW)✅ Verifying example files...$(RESET)"
	@if [ -f test_image.aimg ]; then \
		cargo run --bin aimf -- verify test_image.aimg && echo "  ✅ Image verified" || echo "  ❌ Image invalid"; \
	fi
	@if [ -f test_audio.aaud ]; then \
		cargo run --bin aimf -- verify test_audio.aaud && echo "  ✅ Audio verified" || echo "  ❌ Audio invalid"; \
	fi
	@if [ -f test_video_10sec.avid ]; then \
		cargo run --bin aimf -- verify test_video_10sec.avid && echo "  ✅ Video verified" || echo "  ❌ Video invalid"; \
	fi

keygen: ## Generate a test key pair
	@echo "$(YELLOW)🔑 Generating test key...$(RESET)"
	cargo run --bin aimf -- gen-key --output test.key
	@echo "$(GREEN)✅ Key saved to test.key$(RESET)"
	@echo "$(RED)⚠️  DO NOT use test.key in production!$(RESET)"

pre-commit: format lint test ## Run all pre-commit checks
	@echo "$(GREEN)✅ All pre-commit checks passed!$(RESET)"

ci: ## Run CI pipeline locally (same as GitHub Actions)
	@echo "$(YELLOW)🔄 Running CI locally...$(RESET)"
	make format
	make lint
	make test-all
	make audit
	@echo "$(GREEN)✅ CI pipeline passed!$(RESET)"

bench: ## Run benchmarks
	@echo "$(YELLOW)📊 Running benchmarks...$(RESET)"
	cargo bench --workspace
	@echo "$(GREEN)✅ Benchmarks complete$(RESET)"

coverage: ## Generate test coverage report
	@echo "$(YELLOW)📈 Generating coverage report...$(RESET)"
	cargo install --force cargo-tarpaulin 2>/dev/null || true
	cargo tarpaulin --out Html --workspace --output-dir ./coverage
	@echo "$(GREEN)✅ Coverage report at coverage/tarpaulin-report.html$(RESET)"
	open coverage/tarpaulin-report.html 2>/dev/null || xdg-open coverage/tarpaulin-report.html 2>/dev/null || echo "Open coverage/tarpaulin-report.html manually"

release-build: release ## Build release binaries for all platforms (requires cross-compilation setup)
	@echo "$(YELLOW)🏗️  Building cross-platform binaries...$(RESET)"
	@echo "$(RED)Note: This requires cross-compilation toolchains$(RESET)"
	@echo "  Linux x86_64..."
	cargo build --release --target x86_64-unknown-linux-gnu --bin aimf 2>/dev/null || echo "  ⚠️  Skipped (install target)"
	@echo "  macOS x86_64..."
	cargo build --release --target x86_64-apple-darwin --bin aimf 2>/dev/null || echo "  ⚠️  Skipped (install target)"
	@echo "  Windows x86_64..."
	cargo build --release --target x86_64-pc-windows-msvc --bin aimf 2>/dev/null || echo "  ⚠️  Skipped (install target)"
	@echo "$(GREEN)✅ Binaries built in target/*/release/$(RESET)"

release-upload: ## Upload release binaries to GitHub (requires GITHUB_TOKEN)
	@echo "$(YELLOW)📤 Uploading release artifacts...$(RESET)"
	@echo "$(RED)This should be done by GitHub Actions, not manually$(RESET)"
	@echo "  Use: git tag v0.1.0 && git push origin v0.1.0"

docker-build: ## Build Docker image
	docker build -t aimf:latest -f Dockerfile .

docker-run: ## Run AIMF in Docker
	docker run --rm -v $(PWD):/workspace aimf:latest aimf --help

dev-shell: ## Start a development shell with all dependencies
	@echo "$(YELLOW)🐚 Starting dev shell...$(RESET)"
	@bash --rcfile <(echo "PS1='(aimf-dev) \w\$ '; echo 'Ready to hack on AIMF!'")

version: ## Show version information
	@echo "AIMF Version Information"
	@echo "========================"
	@cargo run --bin aimf -- --version 2>/dev/null || echo "  aimf: not built"
	@echo "  Rust: $$(rustc --version)"
	@echo "  Cargo: $$(cargo --version)"

deps: ## Show dependency graph
	cargo tree --workspace --depth 1

check-updates: ## Check for outdated dependencies
	cargo install --force cargo-outdated 2>/dev/null || true
	cargo outdated --workspace --root-deps-only


install-mime: ## Install MIME types (Linux)
	@if [ "$(shell uname)" = "Linux" ]; then \
		sudo ./scripts/install-mime.sh; \
	elif [ "$(shell uname)" = "Darwin" ]; then \
		./scripts/install-mime-macos.sh; \
	elif [ "$(shell uname | cut -c1-5)" = "MINGW" ]; then \
		powershell -ExecutionPolicy Bypass -File ./scripts/install-mime.ps1; \
	else \
		echo "Unknown OS"; \
	fi

uninstall-mime: ## Uninstall MIME types (Linux)
	@if [ "$(shell uname)" = "Linux" ]; then \
		sudo ./scripts/uninstall-mime.sh; \
	else \
		echo "Manual uninstall required for $(shell uname)"; \
	fi

install-desktop: install-mime ## Install desktop integration (all platforms)
	@echo "Desktop integration complete"