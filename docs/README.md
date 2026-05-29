# AI Media Format Engine — Documentation

Welcome to the AIMF docs. Start here depending on what you need:

## 📖 For Users (CLI)

| Document | Purpose |
|----------|---------|
| [USAGE.md](USAGE.md) | Complete CLI guide — all commands, all formats |
| [BATCH.md](BATCH.md) | Batch processing thousands of files |
| [WORKFLOWS.md](WORKFLOWS.md) | Real-world examples (training, distribution, audit) |
| [SCHEMA.md](SCHEMA.md) | JSON input formats for ingestion |

## 🔧 For Developers (Rust API)

| Document | Purpose |
|----------|---------|
| [API.md](API.md) | Rust API reference (types, functions, codecs) |
| [../spec/README.md](../spec/README.md) | Binary format specification |

## 🏢 For Organizations

| Document | Purpose |
|----------|---------|
| [COMPANIES.md](COMPANIES.md) | Adopters and case studies |
| [IANA-APPLICATION.md](IANA-APPLICATION.md) | Media type registration draft |
| [../SECURITY.md](../SECURITY.md) | Security policy and key management |

## 📖 Core Documents

| Document | Purpose |
|----------|---------|
| [VISION.md](VISION.md) | Why AIMF exists, benefits, roadmap |
| [FAQ.md](FAQ.md) | Common questions |
| [ROADMAP.md](ROADMAP.md) | Future plans |

## 🧪 Examples

Run these from the project root:

```bash
cargo run --example ai_generate_video_simple   # Creates test_video_10sec.avid
cargo run --example ai_generate_image          # Creates test_image.aimg
cargo run --example ai_generate_audio          # Creates test_audio.aaud

xample source code: /examples/
❓ Getting Help
Channel	Best for
GitHub Discussions	Questions, ideas, general help
Issues	Bug reports, feature requests
#aimf on Rust Community Discord	Real-time chat (if set up)
📚 External Resources

    PNG Specification (RFC 2083)

    WAV Format (Microsoft)

    MP4 (ISO/IEC 14496-12)

    Ed25519 (RFC 8032)

📝 Contributing to Docs

Found a typo? Direct PRs welcome. Larger changes? Please open an issue first.

Docs live in /docs/ and /spec/ in the repository.