#!/bin/bash
# This creates release packages, but doesn't commit generated files

echo "🔨 Building release..."

# Build Rust binaries
cargo build --release

# Create pip package (without committing the compiled binary)
cd pip-package
python3 -m maturin build --release
cd ..

# Create npm package
cd npm-package
npm pack
cd ..

echo "✅ Release packages created in target/ and pip-package/target/"
echo "📦 These are ready for publishing to crates.io, PyPI, and npm"
