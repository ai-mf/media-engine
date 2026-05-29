# AIMF Project Roadmap

## ✅ Completed (v0.1.0)

- [x] Core container format design
- [x] CBOR serialization
- [x] SHA-256 hashing
- [x] Ed25519 signing/verification
- [x] AIMG (PNG embedding)
- [x] AAUD (WAV embedding)
- [x] AVID (MP4 embedding)
- [x] CLI tools (aimf, aimg, aaud, avid)
- [x] JSON ingestion
- [x] Raw data ingestion
- [x] Batch processing
- [x] Examples (video, image, audio)
- [x] Comprehensive documentation
- [x] GitHub Actions CI

## 🚀 v1.0 (Q2 2026)

**Goal:** Stable, production-ready release with IANA registration

### Features
- [ ] IANA media type registration
  - `image/aimg`
  - `audio/aaud`
  - `video/avid`
- [ ] API stabilization (no breaking changes after v1.0)
- [ ] Performance benchmarks and optimization
- [ ] Official Docker images
- [ ] Pre-built binaries for all platforms (GitHub Releases)
- [ ] Homebrew formula (macOS)
- [ ] APT repository (Ubuntu/Debian)
- [ ] Chocolatey package (Windows)

### Documentation
- [ ] Video tutorial (YouTube)
- [ ] Interactive web demo
- [ ] Migration guide from v0.x

### Testing
- [ ] Fuzzing for robustness
- [ ] Property-based testing
- [ ] Performance regression tests
- [ ] Cross-platform test matrix expansion

## 🔮 v1.1 (Q3 2026)

**Goal:** Language bindings and ecosystem

### Bindings
- [ ] Python bindings (PyO3)
- [ ] Node.js bindings (N-API)
- [ ] WebAssembly (WASM) for browser verification
- [ ] C/C++ header (for FFI)

### Tooling
- [ ] VSCode extension (syntax highlighting for .aimf files)
- [ ] Pre-commit hook for verification
- [ ] `cargo-aimf` subcommand

## 🎯 v2.0 (Q1 2027)

**Goal:** Advanced features for enterprise

### Streaming
- [ ] Streaming API (verify without loading entire file)
- [ ] Partial verification (verify only header)
- [ ] Progressive extraction

### Advanced crypto
- [ ] Multi-signature support
- [ ] Key revocation lists
- [ ] Certificate chains (X.509)
- [ ] Hardware Security Module (HSM) support

### New formats
- [ ] AIGIF (GIF container)
- [ ] AIPDF (PDF container)
- [ ] AIWEBP (WebP container)

### Performance
- [ ] Parallel verification (SIMD for hash)
- [ ] Zero-copy parsing
- [ ] Memory-mapped I/O

## 📊 Beyond v2.0 (Research)

### Decentralized verification
- [ ] Blockchain timestamping
- [ ] Distributed trust networks
- [ ] Zero-knowledge proofs for prompt privacy

### AI-specific
- [ ] Model fingerprinting (detect which model generated content)
- [ ] Watermarking integration
- [ ] Adversarial robustness

## 🗓️ Timeline
Q1 2026 ──► v0.1.0 (current)
│
▼
Q2 2026 ──► v1.0 (IANA registration)
│
▼
Q3 2026 ──► v1.1 (language bindings)
│
▼
Q4 2026 ──► Maintenance & bugfixes
│
▼
Q1 2027 ──► v2.0 (streaming + advanced crypto)
text


## 🤝 How to contribute to the roadmap

1. Open a GitHub Discussion with `[ROADMAP]` in title
2. Describe your use case
3. Vote on existing proposals (👍/👎)

We prioritize features by:
- Community demand (GitHub reactions)
- Feasibility (estimated implementation effort)
- Alignment with mission (verifiable AI content)

## 📅 Release process

1. **Feature freeze** (2 weeks before release)
2. **Release candidate** (1 week testing)
3. **Final release** (tagged `vX.Y.Z`)
4. **Backport policy**: Critical bugfixes only to previous major version

## 🔒 Security releases

Critical security fixes get expedited releases:
- **Critical** — 48 hours
- **High** — 7 days
- **Medium/Low** — Next regular release

## 📢 Announcements

Follow releases:
- GitHub Releases (watch repo)
- Twitter: `@aimediaformat`
- Mastodon: `__@fosstodon.org`
- Monthly newsletter (signup on website — placeholder)