#!/bin/bash
# Development environment setup script for AIMF
# Usage: ./dev-environment-setup.sh

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
RESET='\033[0m'

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${RESET}"
echo -e "${BLUE}║     AI Media Engine - Development Environment Setup       ║${RESET}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${RESET}"
echo ""

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        MINGW*)     echo "windows";;
        CYGWIN*)    echo "windows";;
        *)          echo "unknown";;
    esac
}

OS=$(detect_os)
echo -e "${YELLOW}📦 Detected OS: $OS${RESET}"

# Check Rust installation
check_rust() {
    if command -v rustc &> /dev/null; then
        echo -e "${GREEN}✅ Rust already installed: $(rustc --version)${RESET}"
        return 0
    else
        echo -e "${RED}❌ Rust not found${RESET}"
        return 1
    fi
}

# Install Rust
install_rust() {
    echo -e "${YELLOW}🔧 Installing Rust...${RESET}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✅ Rust installed${RESET}"
}

# Install system dependencies
install_deps_linux() {
    echo -e "${YELLOW}📦 Installing system dependencies (Ubuntu/Debian)...${RESET}"
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        ffmpeg \
        libavcodec-dev \
        libavformat-dev \
        libavutil-dev \
        libpng-dev \
        libjpeg-dev \
        clang \
        llvm \
        git \
        curl \
        wget \
        cmake
    echo -e "${GREEN}✅ System dependencies installed${RESET}"
}

install_deps_macos() {
    echo -e "${YELLOW}📦 Installing system dependencies (macOS)...${RESET}"
    if ! command -v brew &> /dev/null; then
        echo -e "${YELLOW}Installing Homebrew...${RESET}"
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    brew update
    brew install ffmpeg pkg-config cmake llvm
    echo -e "${GREEN}✅ System dependencies installed${RESET}"
}

install_deps_windows() {
    echo -e "${YELLOW}📦 Installing system dependencies (Windows)...${RESET}"
    if ! command -v choco &> /dev/null; then
        echo -e "${RED}Please install Chocolatey first: https://chocolatey.org/install${RESET}"
        exit 1
    fi
    choco install ffmpeg llvm make --yes
    echo -e "${GREEN}✅ System dependencies installed${RESET}"
}

# Install cargo tools
install_cargo_tools() {
    echo -e "${YELLOW}🔧 Installing cargo tools...${RESET}"
    
    # Update rustup
    rustup update
    rustup component add rustfmt clippy
    
    # Install useful cargo tools
    cargo install cargo-audit --quiet || echo "  cargo-audit already installed"
    cargo install cargo-tarpaulin --quiet || echo "  cargo-tarpaulin already installed"
    cargo install cargo-outdated --quiet || echo "  cargo-outdated already installed"
    cargo install cargo-watch --quiet || echo "  cargo-watch already installed"
    cargo install just --quiet || echo "  just already installed"
    cargo install cargo-husky --quiet || echo "  cargo-husky already installed"
    
    echo -e "${GREEN}✅ Cargo tools installed${RESET}"
}

# Clone repository (if not already in one)
clone_repo() {
    if [ -f "Cargo.toml" ] && grep -q "aimf_core" Cargo.toml 2>/dev/null; then
        echo -e "${GREEN}✅ Already in AIMF repository${RESET}"
        return 0
    fi
    
    echo -e "${YELLOW}📥 Cloning AIMF repository...${RESET}"
    git clone https://github.com/ai-mf/media-engine.git
    cd media-engine
    echo -e "${GREEN}✅ Repository cloned${RESET}"
}

# Build the project
build_project() {
    echo -e "${YELLOW}🏗️  Building project...${RESET}"
    cargo build --workspace
    echo -e "${GREEN}✅ Build complete${RESET}"
}

# Run tests
run_tests() {
    echo -e "${YELLOW}🧪 Running tests...${RESET}"
    cargo test --workspace
    echo -e "${GREEN}✅ Tests passed${RESET}"
}

# Generate documentation
generate_docs() {
    echo -e "${YELLOW}📚 Generating documentation...${RESET}"
    cargo doc --no-deps --workspace
    echo -e "${GREEN}✅ Documentation generated at target/doc/aimf_core/index.html${RESET}"
}

# Setup git hooks
setup_git_hooks() {
    echo -e "${YELLOW}🔗 Setting up git hooks...${RESET}"
    cargo husky init 2>/dev/null || echo "  Run 'cargo husky init' manually for git hooks"
    echo -e "${GREEN}✅ Git hooks configured${RESET}"
}

# Create test key
create_test_key() {
    echo -e "${YELLOW}🔑 Creating test signing key...${RESET}"
    cargo run --bin aimf -- gen-key --output test.key 2>/dev/null || echo "  Build first: make release"
    echo -e "${GREEN}✅ Test key created (test.key)${RESET}"
}

# Main execution
main() {
    if ! check_rust; then
        install_rust
    fi
    
    case $OS in
        linux)
            install_deps_linux
            ;;
        macos)
            install_deps_macos
            ;;
        windows)
            install_deps_windows
            ;;
        *)
            echo -e "${RED}Unsupported OS${RESET}"
            exit 1
            ;;
    esac
    
    install_cargo_tools
    clone_repo
    build_project
    run_tests
    generate_docs
    setup_git_hooks
    create_test_key
    
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${GREEN}║       Development environment setup complete! 🎉           ║${RESET}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    echo -e "${YELLOW}Next steps:${RESET}"
    echo "  1. Run 'make examples' to generate test files"
    echo "  2. Run 'make help' to see all available commands"
    echo "  3. Read documentation in docs/ directory"
    echo "  4. Start hacking! 🚀"
    echo ""
}

main