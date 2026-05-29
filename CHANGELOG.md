# Changelog

All notable changes to AI Media Engine will be documented here.

Format based on [Keep a Changelog](https://keepachangelog.com/), versioning follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- (nothing yet)

### Changed
- (nothing yet)

### Fixed
- (nothing yet)

## [0.1.0] - 2026-01-15

### Added
- Initial release
- AIMG format (AI images in PNG containers)
- AAUD format (AI audio in WAV containers)
- AVID format (AI video in MP4 containers)
- `aimf`, `aimg`, `aaud`, `avid` CLI tools
- Ed25519 signing and verification
- Batch processing (`batch` command)
- JSON and raw input ingestion
- Examples: `ai_generate_video_simple`, `ai_generate_image`, `ai_generate_audio`
- Documentation: USAGE, BATCH, API, SCHEMA, WORKFLOWS

### Security
- Cryptographic hashing (SHA-256) for integrity
- Optional Ed25519 signatures for authenticity
- No hardcoded keys or backdoors

[0.1.0]: https://github.com/ai-mf/media-engine/releases/tag/v0.1.0