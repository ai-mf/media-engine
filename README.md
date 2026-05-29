# 🎬 AI Media Engine

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-red.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Documentation](https://img.shields.io/badge/docs-API-blue.svg)](docs/API.md)

**Verifiable AI media containers** — embed provenance into PNG, WAV, and MP4 files while staying playable in standard players (VLC, ffplay, etc.).

```bash
# Generate a signed AI video in 30 seconds
cargo run --example ai_generate_video_simple
vlc test_video_10sec.avid

✨ Features
Format	Extension	Base	Plays in VLC	Use case
AIMG	.aimg	PNG	✅	AI-generated images
AAUD	.aaud	WAV	✅	AI-generated audio
AVID	.avid	MP4	✅	AI-generated video

    🔐 Cryptographic signatures (Ed25519) — prove authenticity

    🧩 Backward compatible — existing media players work normally

    📦 Batch processing — handle thousands of files

    🚀 Fast — written in Rust, zero-copy where possible

📚 Documentation
What do you want?	Go here
Get started (5 min)	USAGE.md — complete CLI guide
Batch operations	BATCH.md — process many files
API reference	docs/API.md — Rust integration
JSON schemas	docs/SCHEMA.md — input formats
Common workflows	docs/WORKFLOWS.md — real examples
Format specification	spec/README.md — technical deep dive
⚡ Quick start
bash

# Clone and build
git clone https://github.com/ai-mf/media-engine.git
cd media-engine
cargo build --release

# Generate a key (for signing)
cargo run --bin aimf -- gen-key --output private.key

# Create a signed image from JSON
echo '{"width":800,"height":600,"pixels":[255,0,0,0,255,0]}' | \
  cargo run --bin aimf -- json \
  --output my_image.aimg \
  --model "StableDiffusion" \
  --version "1.5" \
  --key private.key

# Verify it
cargo run --bin aimf -- verify my_image.aimg

🧪 Try the examples
bash

cargo run --example ai_generate_video_simple   # Creates test_video_10sec.avid
cargo run --example ai_generate_image          # Creates test_image.aimg
cargo run --example ai_generate_audio          # Creates test_audio.aaud

# View them (opens in system player)
cargo run --bin aimf -- view test_video_10sec.avid

🗂️ Project structure
text

media-engine/
├── README.md              # You are here
├── USAGE.md               # Complete CLI guide
├── BATCH.md               # Batch processing guide
├── aimf_core/             # Core types & crypto (locked)
├── codecs/                # PNG, WAV, MP4 embeddings
├── commands/              # CLI commands
├── services/              # 
├── tools/                 # CLI binaries (aimf, aimg, aaud, avid)
├── examples/              # Ready-to-run demos
├── docs/                  # API, workflows, schemas
└── spec/                  # Format specifications

🔒 Core directory policy

aimf_core/ is locked — changes require approval. This ensures hash and signature logic stays consistent.
🤝 Contributing

See CONTRIBUTING.md. TL;DR:

    All PRs must pass cargo test && cargo fmt && cargo clippy

    aimf_core/ changes need an issue first

    Use the PR template

📄 License

Apache 2.0 — see LICENSE for details.
🙏 Acknowledgments

    ed25519-dalek for signatures

    FFmpeg for encoding

    Inspired by C2PA, EXIF, and ID3 standards