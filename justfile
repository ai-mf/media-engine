# justfile for AI Media Engine
# Install: cargo install just
# Usage: just <recipe>

# Set shell for better errors
set shell := ["bash", "-ue"]

# Default recipe
default:
    @just --list

# Build
build:
    cargo build --workspace

release:
    cargo build --release --workspace

# Test
test:
    cargo test --workspace

test-all:
    cargo test --workspace --all-targets --all-features

test-integration:
    cargo test --test '*' --workspace

# Code quality
fmt:
    cargo fmt --all

lint:
    cargo clippy --workspace -- -D warnings

fix:
    cargo clippy --workspace --fix --allow-dirty --allow-staging
    cargo fmt --all

# Documentation
doc:
    cargo doc --no-deps --workspace --document-private-items
    @echo "Docs at target/doc/aimf_core/index.html"

# Security
audit:
    cargo audit

# Clean
clean:
    cargo clean
    rm -rf target/

# Run examples
examples:
    @echo "Running image example..."
    cargo run --example ai_generate_image
    @echo "Running audio example..."
    cargo run --example ai_generate_audio
    @echo "Running video example..."
    cargo run --example ai_generate_video_simple

# Verify all AIMF files in current directory
verify-all:
    @echo "Verifying all AIMF files..."
    @for file in *.aimg *.aaud *.avid 2>/dev/null; do \
        echo "  $$file: $$(cargo run --bin aimf -- verify "$$file" 2>&1 | grep -E "VALID|INVALID" || echo "FAILED")"; \
    done

# Generate a key
keygen name="test":
    cargo run --bin aimf -- gen-key --output "{{name}}.key"
    @echo "Key saved to {{name}}.key"

# Create a test image
test-image name="test":
    @echo '{"width":100,"height":100,"pixels":[255,0,0,0,255,0,0,0,255]}' | \
        cargo run --bin aimf -- ingest --output "{{name}}.aimg" --model test --version 1.0

# Create a test audio
test-audio name="test":
    @echo '{"sample_rate":44100,"samples":[0.5,-0.3,0.2,-0.1,0.4]}' | \
        cargo run --bin aimf -- ingest --type audio --output "{{name}}.aaud" --model test --version 1.0

# Create a test video
test-video name="test":
    @echo '{"width":320,"height":240,"fps":30,"frames":[[255,0,0,0,255,0],[0,0,255,255,255,0]]}' | \
        cargo run --bin aimf -- ingest --type video --output "{{name}}.avid" --model test --version 1.0

# Info about a file
info file:
    cargo run --bin aimf -- info "{{file}}"

# Verify a file
verify file:
    cargo run --bin aimf -- verify "{{file}}"

# Extract a file
extract file output="extracted":
    cargo run --bin aimf -- extract "{{file}}" --output "{{output}}"

# Batch verify a pattern
batch-verify pattern="*.aimg":
    cargo run --bin aimf -- batch --input "{{pattern}}" --verify --parallel

# Pre-commit checks
pre-commit: fmt lint test

# CI pipeline
ci: fmt lint test-all audit

# Install binaries
install:
    cargo install --path tools/aimf-cli
    cargo install --path tools/aaud-cli
    cargo install --path tools/aimg-cli
    cargo install --path tools/avid-cli

# Uninstall
uninstall:
    cargo uninstall aimf aaud aimg avid || true

# Development shell with nix (if using nix)
dev-shell:
    @echo "Starting development shell..."
    @nix-shell -p cargo rustc rustfmt clippy cargo-audit ffmpeg

# Version info
version:
    @echo "AIMF: $$(cargo run --bin aimf -- --version 2>/dev/null || echo 'not built')"
    @echo "Rust: $$(rustc --version)"
    @echo "Cargo: $$(cargo --version)"

# Dependency graph
deps:
    cargo tree --workspace --depth 1

# Outdated dependencies
outdated:
    cargo outdated --workspace

# Run benchmarks
bench:
    cargo bench --workspace

# Coverage
coverage:
    cargo tarpaulin --out Html --workspace --output-dir coverage
    @echo "Coverage report at coverage/tarpaulin-report.html"

# Clean and rebuild
rebuild: clean release

# Watch mode for development (requires cargo-watch)
watch-test:
    cargo watch -x test

watch-build:
    cargo watch -x build

# Run CI checks on save
watch-ci:
    cargo watch -x "clippy --workspace -- -D warnings" -x "test" -x "fmt -- --check"