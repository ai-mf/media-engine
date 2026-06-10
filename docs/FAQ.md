Here's your corrected **FAQ.md**:

```markdown
# Frequently Asked Questions

## General

### What is AIMF?

A set of container formats (AIMG, AAUD, AVID) that embed AI provenance metadata into standard media files (PNG, WAV, MP4) while remaining playable in any standard media player.

### Why not just use EXIF or ID3 tags?

Those formats are:
- Limited in size (EXIF: 64KB typical)
- Not designed for cryptographic signatures
- Not consistent across media types
- No standard for AI-specific fields

AIMF provides a unified format across all media types with built-in crypto.

### Is AIMF an open standard?

Yes — Apache 2.0 licensed, specification published, seeking IANA registration.

## Technical

### How much overhead does AIMF add?

| Format | Overhead |
|--------|----------|
| AIMG | ~200 bytes + metadata size |
| AAUD | ~100 bytes + metadata size |
| AVID | ~150 bytes + metadata size |

For a 1MB image: ~0.02% overhead.

### Can I recover the original file?

Yes — `aimf extract` removes all AIMF metadata and gives you the pure media file.

### Are signatures required?

No — they're optional. Create without `--key` for unsigned files.

### What happens if I modify a signed file?

Verification will fail (hash mismatch). You can still play the file normally.

### Can I sign an already-signed file?

Yes — new signature replaces old one.

### Do I need to trust a central authority?

No — verification uses public key cryptography. You decide which public keys to trust.

### What input formats are supported?

RAW binary only (JSON format was removed in v1.0):
- Audio: 16-bit signed little-endian PCM
- Image: RGB24 (3 bytes per pixel)
- Video: RGB24 frames + optional PCM audio

### Can I convert existing MP3/MP4 files?

Yes — use the `convert` command:
```bash
cargo run --bin aaud -- convert input.mp3 --output output.aaud --model "Model" --version "1.0"
cargo run --bin avid -- convert input.mp4 --output output.avid --model "Model" --version "1.0"
```

## Security

### How secure is Ed25519?

- 128-bit security level (same as AES-128)
- No known practical attacks
- Used by OpenSSH, Tor, WhatsApp

### Can someone forge my signature?

Only if they have your private key. Keep it secure.

### What if I lose my private key?

You can't sign new files with that identity. Already-signed files remain verifiable.

### Can I revoke a compromised key?

Not in the format itself — that requires a separate revocation list (planned for v2).

## Compatibility

### Will my files work in 10 years?

Yes — PNG, WAV, and MP4 are stable, well-documented formats. Unknown chunks are ignored by future players.

### Can I open AIMF files in Photoshop?

Yes — Photoshop will open AIMG files as normal PNGs (ignoring metadata).

### Does YouTube accept .avid files?

Yes — they're valid MP4 files. Upload normally.

### Can I convert AIMF to other formats?

Use `aimf extract` to get the original media, then any converter.

## Performance

### How fast is verification?

~100,000 verifications/second on a modern CPU (Ed25519 is very fast).

### Can I stream large videos?

Current version loads entire file. Streaming API is planned.

### What's the maximum file size?

No hard limit — tested with 50GB files. Memory usage scales with file size.

## Troubleshooting

### "Not a valid AIMF file"

The file was created by a different tool or corrupted. Check with `file` command:

```bash
file mystery.file
# Should show PNG, WAV, or MP4 data
```

### Signature verification fails

Possible causes:
- File was modified after signing
- Wrong public key (different signer)
- File corruption
- Bug (unlikely — open an issue)

### FFmpeg not found (video)

Install FFmpeg with extra codecs:

```bash
# Ubuntu/Debian (recommended - full codec support)
sudo apt install ffmpeg libavcodec-extra

# macOS
brew install ffmpeg

# Windows (choco)
choco install ffmpeg

# Windows (winget)
winget install ffmpeg

# Arch Linux
sudo pacman -S ffmpeg
```

**Note:** `libavcodec-extra` provides additional codecs (MP3, AAC, H.264, etc.) beyond the basic `ffmpeg` package. Required for MP4 conversion.

### Build fails on Windows

Ensure you have:
- Visual Studio Build Tools (with C++ support)
- LLVM (for libclang)
- FFmpeg installed

### "Permission denied" for private key

```bash
chmod 600 private.key   # Unix
# Windows: Right-click → Properties → Security
```

## Comparisons

### vs. C2PA (Content Authenticity Initiative)

| Aspect | AIMF | C2PA |
|--------|------|------|
| Complexity | Simple | Complex (JSON-LD, X.509) |
| Dependencies | None (pure Rust) | Many (C2PA SDK) |
| Backward compat | Yes (PNG/WAV/MP4) | Partial (some formats break) |
| Signatures | Ed25519 | X.509 (requires CA) |
| Use case | Lightweight provenance | Enterprise DRM |

**Choose AIMF if:** You want simple, embeddable, no external dependencies.

### vs. EXIF for images

| Aspect | AIMF | EXIF |
|--------|------|------|
| Max metadata size | Unlimited | ~64KB |
| Signatures | Yes | No |
| Audio/video support | Yes | No |
| AI-specific fields | Yes (model, prompt hash) | No |

## Development

### Can I use AIMF in my Rust project?

Yes — add to Cargo.toml:

```toml
[dependencies]
aimf_core = { git = "https://github.com/ai-mf/media-engine" }
aimf_image_codec = { git = "https://github.com/ai-mf/media-engine" }
```

### Is there a Python binding?

Not yet — planned for v1.1.

### How do I contribute?

See CONTRIBUTING.md.

## Legal

### What's the license?

Apache 2.0 — commercial-friendly, patent grant included.

### Can I use AIMF in proprietary software?

Yes — Apache 2.0 allows proprietary use (just include attribution).

### Do I need to pay royalties?

No — completely free.

### Is AIMF patented?

No — we explicitly disclaim any patent rights (Apache 2.0 includes patent grant).

## Future

### What's planned for v1.0?

- IANA registration
- Stable API (no breaking changes)
- Performance improvements

### What's planned for v2.0?

- Streaming API
- Key revocation lists
- Multi-signature support
- WASM bindings

### How do I request a feature?

Open a GitHub issue with `[FEATURE]` in the title.