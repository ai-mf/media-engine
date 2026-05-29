# AI Media Format (AIMF) Specification

**Version:** 1.0 (draft)  
**Status:** Stable, seeking IANA registration  
**License:** Apache 2.0

## Overview

AIMF defines three container formats that embed AI provenance metadata into standard media files while maintaining backward compatibility.

| Format | Extension | Container | Marker | MIME (proposed) |
|--------|-----------|-----------|--------|-----------------|
| AIMG | `.aimg` | PNG | `AIMG` text chunk | `image/aimg` |
| AAUD | `.aaud` | WAV | `AAUD` LIST-INFO chunk | `audio/aaud` |
| AVID | `.avid` | MP4 | `AVID` UUID box | `video/avid` |

## Design Principles

1. **Backward compatibility** — Any standard player can view the underlying media
2. **Forward extractability** — AI metadata can be removed to recover original file
3. **Cryptographic optionality** — Signatures are optional but supported
4. **Self-describing** — Metadata includes model, version, timestamp, modality

## Common Structure

All AIMF containers embed a binary serialized `AiContainer` struct:

```rust
struct AiContainer {
    media_type: u8,      // 0=Image, 1=Audio, 2=Video
    encoding: String,    // e.g., "png", "wav", "mp4"
    payload_type: u8,    // 0=Encoded, 1=Raw
    metadata: AiMetadata,
    payload: Vec<u8>,    // Original media bytes
    hash: [u8; 32],      // SHA-256 of payload + metadata
}

AiMetadata Structure
rust

struct AiMetadata {
    is_ai_generated: bool,      // Always true for AIMF
    model_name: String,
    model_version: String,
    prompt_hash: Option<[u8; 32]>,  // SHA-256 of generation prompt
    modality: String,               // "image", "audio", "video"
    format: String,                 // MIME-like: "image/png", "audio/wav"
    width: Option<u32>,
    height: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    fps: Option<u32>,
    timestamp: u64,                 // Unix seconds
    signature: Option<Vec<u8>>,     // Ed25519 signature (64 bytes)
    public_key: Option<Vec<u8>>,    // Ed25519 public key (32 bytes)
}

Format AIMG (AI Image)
Container: PNG

AIMG embeds metadata in a tEXt chunk with keyword "AIMG".
PNG Structure
text

PNG Signature (8 bytes)
├── IHDR chunk (width, height, bit depth, etc.)
├── tEXt chunk (keyword="AIMG", text=serialized AiContainer)
├── IDAT chunk(s) (image data)
└── IEND chunk

Serialization Format

The tEXt chunk contains CBOR (Concise Binary Object Representation) of the AiContainer struct, then base64-encoded to fit PNG's text constraints.
text

[AIMG]\0[base64(cbor(AiContainer))]

Example: Extract metadata
bash

# Using PNG tools directly
pngcheck -v image.aimg | grep -A20 "tEXt"

# Using aimf
aimf info image.aimg

Compatibility

    ✅ Standard PNG viewers show the image (ignore unknown tEXt chunks)

    ✅ ffmpeg, ImageMagick, GIMP all work

    ✅ Web browsers display as normal PNG

Format AAUD (AI Audio)
Container: WAV (RIFF)

AAUD embeds metadata in a LIST-INFO chunk with type "AAUD".
WAV Structure
text

RIFF Header
├── fmt  chunk (audio format info)
├── data chunk (audio samples)
└── LIST chunk
    └── INFO subchunk (type="AAUD", data=serialized AiContainer)

Serialization Format

The LIST-INFO chunk contains CBOR of the AiContainer struct (raw, not base64).
text

"AAUD" (4 bytes) + size (4 bytes) + cbor(AiContainer)

Example: Extract metadata
bash

# Using ffprobe
ffprobe -v quiet -print_format json -show_format audio.aaud

# Using aimf
aimf info audio.aaud

Compatibility

    ✅ All WAV players play the audio (ignore unknown chunks)

    ✅ Audacity, VLC, ffplay all work

    ✅ Windows Media Player (basic WAV support)

Format AVID (AI Video)
Container: MP4 (ISO/IEC 14496-12)

AVID embeds metadata in a UUID box (uuid) with UUID {A1B2C3D4-E5F6-4789-AB12-CD34EF567890}.
MP4 Structure
text

ftyp box (file type)
├── moov box (metadata)
│   ├── mvhd (movie header)
│   ├── trak (track)
│   └── uuid (AVID metadata) ← NEW
│       ├── uuid_type = "AVID"
│       └── data = cbor(AiContainer)
└── mdat box (media data)

UUID Value
text

A1B2C3D4-E5F6-4789-AB12-CD34EF567890

Registered with ISO for private use.
Serialization Format

The UUID box contains raw CBOR of the AiContainer struct.
Example: Extract metadata
bash

# Using MP4Box
MP4Box -info video.avid

# Using ffprobe
ffprobe -v quiet -print_format json -show_format video.avid

# Using aimf
aimf info video.avid

Compatibility

    ✅ Any MP4 player (VLC, ffplay, QuickTime) plays the video

    ✅ YouTube accepts .avid files (they're valid MP4)

    ✅ Unknown UUID boxes are ignored by standard parsers

Cryptographic Signing
Ed25519 Integration
text

┌─────────────────────────────────────────────────────────┐
│ 1. Serialize AiContainer (without signature/public_key) │
│    ↓                                                    │
│ 2. SHA-256 hash of serialized bytes                    │
│    ↓                                                    │
│ 3. Sign hash with Ed25519 private key                  │
│    ↓                                                    │
│ 4. Store signature (64 bytes) + public key (32 bytes)  │
│    in AiContainer fields                               │
└─────────────────────────────────────────────────────────┘

Verification
text

1. Extract signature and public_key from AiContainer
2. Remove them (temporarily) from the struct
3. Re-serialize and hash
4. Verify signature using public key
5. Compare hash with stored hash

Security Properties
Property	Mechanism
Integrity	SHA-256 hash of payload+metadata
Authenticity	Ed25519 signature
Non-repudiation	Public key identifies signer
Tamper detection	Hash mismatch = corrupted
Migration from v0 (if any)

No v0 exists — this is the initial specification.
IANA Registration Status

    Proposed: Q2 2026

    Review: TBD

    See: docs/IANA-APPLICATION.md

Reference Implementations
Language	Repository	Status
Rust	This repo	✅ Complete
Python	(planned)	❌ Not started
JavaScript	(planned)	❌ Not started
Conformance Testing

To claim AIMF conformance, an implementation must:

    Read all three container formats

    Preserve unknown chunks when round-tripping

    Verify Ed25519 signatures correctly

    Extract original media losslessly

Test suite: cargo test --workspace
Version History
Version	Date	Changes
1.0-draft	2026-01	Initial specification
1.0	TBD	IANA submission
Appendix A: CBOR Schema (CDDL)
cddl

AiContainer = {
    media_type: 0..2,
    encoding: tstr,
    payload_type: 0..1,
    metadata: AiMetadata,
    payload: bytes,
    hash: bytes .size 32,
}

AiMetadata = {
    is_ai_generated: true,
    model_name: tstr,
    model_version: tstr,
    ? prompt_hash: bytes .size 32,
    modality: tstr,
    format: tstr,
    ? width: uint,
    ? height: uint,
    ? sample_rate: uint,
    ? channels: uint,
    ? fps: uint,
    timestamp: uint,
    ? signature: bytes .size 64,
    ? public_key: bytes .size 32,
}

Appendix B: Magic Bytes
Format	Offset	Bytes
AIMG	PNG offset + IHDR	‰PNG
AAUD	0	RIFF....WAVE
AVID	4	ftypmp4

AIMF markers appear after the magic bytes.
Appendix C: Example Files

Test files are in /examples/output/ after running:
bash

cargo run --example ai_generate_image
cargo run --example ai_generate_audio
cargo run --example ai_generate_video_simple

References

    PNG Specification (RFC 2083)

    WAV Format (Microsoft)

    MP4 (ISO/IEC 14496-12)

    CBOR (RFC 8949)

    Ed25519 (RFC 8032)