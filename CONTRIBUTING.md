# Contributing to AI Media Engine

Thanks for your interest! This doc will get you from "I want to help" to "PR merged."

## 📋 Before you start

1. **Search existing issues** — someone might already be working on it
2. **Open an issue first** for bugs or features (unless it's a typo)
3. **Sign your commits** (`git commit -S`) — not strictly required, but appreciated

## 🧪 Development setup

```bash
# Clone your fork
git clone https://github.com/ai-mf/media-engine.git
cd media-engine

# Build and test everything
cargo build --workspace
cargo test --workspace

# Run the examples to verify they work
cargo run --example ai_generate_video_simple

✅ Before submitting a PR

Run these checks locally:
bash

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Test
cargo test --workspace

# Build release (catches some errors)
cargo build --release --workspace

🔒 Special rules for aimf_core/

Changes to aimf_core/ (hash, crypto, serialization) will be rejected unless:

    You open an issue first explaining why

    The issue gets a "core-change-approved" label

    Your PR includes additional tests for the change

Why? This code handles cryptographic verification — stability matters.
📝 Pull request guidelines

    One change per PR — easier to review and revert

    Update documentation if you change CLI behavior

    Add tests for new functionality

    Keep the commit history clean (rebase, don't merge main)

PR title format:
text

type(scope): brief description

types: fix, feat, docs, refactor, test, chore
scope: core, cli, codecs, docs, examples

Example:
fix(cli): correct --output path handling on Windows

🐛 Reporting bugs

Include:

    OS and architecture (uname -a on Linux, sw_vers on macOS)

    Rust version (rustc --version)

    Exact command that failed

    Full error output

    A small reproduction if possible

💡 Suggesting features

Open an issue with:

    What problem does it solve?

    How would it work? (CLI flags, new commands, API changes)

    Is this a breaking change?

📖 Documentation changes

Found a typo or confusing section? Direct PRs are welcome — no issue needed.
❓ Questions?

Open a Discussion (not an issue) on GitHub, or ping @maintainers in an existing thread.

Thank you for helping make AI content verifiable! 🎬