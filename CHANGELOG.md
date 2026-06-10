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

## [1.0.0] - 2026-01-15

### Added
- Initial release
- AIMG format (AI images in PNG containers)
- AAUD format (AI audio in WAV containers)
- AVID format (AI video in MP4 containers)
- `aimf`, `aimg`, `aaud`, `avid` CLI tools
- Ed25519 signing and verification
- Batch processing (`batch` command)
- Raw input ingestion
- Examples : Rust ->`ai_generate_video_simple`, `ai_generate_image`, `ai_generate_audio`
-          : Python ->`audio_generation`, `image_generation`, `video_generation`
- Documentation: USAGE, BATCH, API, SCHEMA, WORKFLOWS

### Security
- Cryptographic hashing (SHA-256) for integrity
- Optional Ed25519 signatures for authenticity
- No hardcoded keys or backdoors

[1.0.0]: https://github.com/ai-mf/media-engine/releases/tag/v1.0.0