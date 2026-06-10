```markdown
# 🎬 AI Media Engine

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-red.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Documentation](https://img.shields.io/badge/docs-USAGE-blue.svg)](USAGE.md)

**Verifiable AI media containers** — embed provenance into PNG, WAV, and MP4 files while staying playable in standard players (VLC, ffplay, etc.).

```bash
# Create a signed AI video from raw frames
cat video_frames.rgb | cargo run --bin avid -- raw \
  --width 640 --height 480 --fps 30 --frame-count 300 \
  --output my_video.avid --model "GenAI" --version "1.0"

# View it
cargo run --bin aimf -- view my_video.avid
```

✨ **Features**

| Format | Extension | Base | Plays in VLC | Use case |
|--------|-----------|------|--------------|----------|
| AIMG | `.aimg` | PNG | ❌  | AI-generated images |
| AAUD | `.aaud` | WAV | ✅ | AI-generated audio |
| AVID | `.avid` | MP4 | ✅ | AI-generated video |

- 🔐 **Cryptographic signatures (Ed25519)** — prove authenticity
- 🔄 **Convert existing media** — MP3 → AAUD, MP4 → AVID
- 🧩 **Backward compatible** — existing media players work normally
- 📦 **Batch processing** — handle thousands of files
- 🚀 **Fast** — written in Rust, zero-copy where possible

## 📚 Documentation

| What do you want? | Go here |
|------------------|---------|
| Get started (5 min) | [USAGE.md](USAGE.md) — complete CLI guide |
| Batch operations | [BATCH.md](BATCH.md) — process many files |
| API reference | [docs/API.md](docs/API.md) — Rust integration |
| Common workflows | [docs/WORKFLOWS.md](docs/WORKFLOWS.md) — real examples |
| Format specification | [spec/README.md](spec/README.md) — technical deep dive |

## ⚡ Quick start

```bash
# Clone and build
git clone https://github.com/ai-mf/media-engine.git
cd media-engine
cargo build --release

# Generate a key (for signing)
cargo run --bin aimf -- gen-key --output private.key

# Create a signed image from raw RGB data
cat image.rgb | cargo run --bin aimf -- raw \
  --width 800 \
  --height 600 \
  --output my_image.aimg \
  --type image \
  --model "StableDiffusion" \
  --version "1.5" \
  --key private.key

# Verify it
cargo run --bin aimf -- verify my_image.aimg

# Convert MP3 to AAUD (new!)
cargo run --bin aaud -- convert input.mp3 --output output.aaud \
  --model "Whisper" --version "1.0"
```

## 🧪 Try the examples

```bash
cargo run --example ai_generate_video_simple   # Creates test_video.avid
cargo run --example ai_generate_image          # Creates test_image.aimg
cargo run --example ai_generate_audio          # Creates test_audio.aaud

# View them (opens in system player)
cargo run --bin aimf -- view test_video.avid
```

## 🗂️ Project structure

```
media-engine/
├── README.md              # You are here
├── USAGE.md               # Complete CLI guide
├── BATCH.md               # Batch processing guide
├── aimf_core/             # Core types & crypto (locked)
├── codecs/                # PNG, WAV, MP4 embeddings
├── commands/              # CLI commands
├── services/              # Business logic
├── tools/                 # CLI binaries (aimf, aimg, aaud, avid)
├── examples/              # Ready-to-run demos
├── docs/                  # API, workflows
└── spec/                  # Format specifications
```

## 🔒 Core directory policy

**`aimf_core/` is locked** — changes require approval. This ensures hash and signature logic stays consistent.

## 🤝 Contributing

See CONTRIBUTING.md. TL;DR:
- All PRs must pass `cargo test && cargo fmt && cargo clippy`
- `aimf_core/` changes need an issue first
- Use the PR template

## 💻 Compatibility

Tested and working with:
- VLC 3.x
- Firefox 100+
- Ubuntu Image Viewer (GNOME 42+)

**Note:** Some ancient or overly strict parsers may reject files, but all modern software handles our embedding method.

## 📄 License

Apache 2.0 — see LICENSE for details.

## 🙏 Acknowledgments

- [ed25519-dalek](https://github.com/dalek-cryptography/ed25519-dalek) for signatures
- FFmpeg for encoding/conversion
- Inspired by C2PA, EXIF, and ID3 standards