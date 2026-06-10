# AI Media Format (AIMF) Specification

**Version:** 1.0  
**Status:** Stable, seeking IANA registration  
**License:** Apache 2.0

## Overview

AIMF defines three container formats that embed AI provenance metadata into standard media files while maintaining backward compatibility.

| Format | Extension | Container | Marker | MIME (proposed) |
|--------|-----------|-----------|--------|-----------------|
| AIMG | `.aimg` | PNG | `aiMg` chunk | `image/prs.aimg` |
| AAUD | `.aaud` | WAV | `AAUD` chunk | `audio/prs.aaud` |
| AVID | `.avid` | MP4 | `uuid` box (`avid-meta-box\0\0\0`) | `video/prs.avid` |

## Design Principles

1. **Backward compatibility** — Any standard player can view the underlying media
2. **Forward extractability** — AI metadata can be removed to recover original file
3. **Cryptographic optionality** — Signatures are optional but supported
4. **Self-describing** — Metadata includes model, version, timestamp, modality
5. **Minimal overhead** — Raw CBOR storage (no base64 for PNG, no JSON overhead)

## Common Structure

All AIMF containers embed a binary serialized `AiContainer` struct using **CBOR** (Concise Binary Object Representation):

```rust
struct AiContainer {
    version: u16,           // Format version (currently 1)
    media_type: MediaType,  // 0=Image, 1=Audio, 2=Video
    encoding: String,       // e.g., "png", "wav", "mp4"
    payload_type: PayloadType, // 0=Encoded, 1=RawFrame, 2=RawVideo, 3=RawAudio
    metadata: AiMetadata,
    hash: [u8; 32],         // SHA-256 of payload + metadata (excluding signature)
}
```

### AiMetadata Structure

```rust
struct AiMetadata {
    is_ai_generated: bool,      // Always true for AIMF
    model_name: String,
    model_version: String,
    prompt_hash: Option<[u8; 32]>,  // SHA-256 of generation prompt
    modality: String,               // "image", "audio", "video"
    format: String,                 // "rgb8", "pcm16", "png", "wav", "mp4"
    width: Option<u32>,
    height: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    fps: Option<u32>,
    timestamp: u64,                 // Unix seconds
    signature: Option<Vec<u8>>,     // Ed25519 signature (64 bytes)
    public_key: Option<Vec<u8>>,    // Ed25519 public key (32 bytes)
}
```

### Binary Serialization Format

```
┌──────────────────┬──────────────────────────┐
│ Header Size (4)  │ Header (CBOR)            │
│ (big-endian)     │ (AiContainer w/o payload)│
└──────────────────┴──────────────────────────┘
```

**Note:** Payload is stored separately in the container format (PNG/WAV/MP4), not in the CBOR stream.

---

## Format AIMG (AI Image)

**Container:** PNG  
**Chunk Type:** `aiMg` (ancillary, safe to ignore)  
**Chunk Location:** Before IEND chunk  
**Data:** Raw CBOR (no base64 encoding)

### PNG Structure

```
PNG Signature (8 bytes)
├── IHDR chunk (width, height, bit depth, etc.)
├── PLTE chunk (palette, optional)
├── IDAT chunk(s) (image data)
├── ... other chunks
├── aiMg chunk ← NEW (CBOR AiContainer)
└── IEND chunk
```

### AIMG Chunk Format

```
┌──────────────┬──────────────┬──────────────────────────┬──────────────┐
│ Length (4)   │ 'aiMg' (4)   │ CBOR AiContainer (raw)   │ CRC32 (4)    │
└──────────────┴──────────────┴──────────────────────────┴──────────────┘
```

### Why `aiMg` instead of `AIMG`?
- Lowercase first letter = ancillary chunk (safe for players to ignore)
- Prevents conflicts with registered chunk types
- Follows PNG specification conventions

### Compatibility

- ✅ Standard PNG viewers show the image (ignore unknown `aiMg` chunk)
- ✅ ffmpeg, ImageMagick, GIMP all work
- ✅ Web browsers display as normal PNG

### Example: Extract metadata

```bash
# Using pngcheck
pngcheck -v image.aimg | grep -A5 "aiMg"

# Using aimf
aimf info image.aimg --simple
```

---

## Format AAUD (AI Audio)

**Container:** WAV (RIFF)  
**Chunk ID:** `AAUD`  
**Chunk Location:** Appended at end of file  
**Data:** Raw CBOR (no encoding)

### WAV Structure

```
RIFF Header (12 bytes)
├── fmt chunk (audio format info)
├── data chunk (audio samples)
├── ... other chunks
└── AAUD chunk ← APPENDED (CBOR AiContainer)
```

### AAUD Chunk Format

```
┌──────────────┬──────────────┬──────────────────────────┐
│ 'AAUD' (4)   │ Size (4)     │ CBOR AiContainer (raw)   │
└──────────────┴──────────────┴──────────────────────────┘
```

### Why appended?
- Simplest implementation (no chunk reordering)
- Update RIFF header size to include new chunk
- Players ignore unknown chunks

### Compatibility

- ✅ All WAV players play the audio (ignore unknown chunks)
- ✅ Audacity, VLC, ffplay all work
- ✅ Windows Media Player (basic WAV support)

### Example: Extract metadata

```bash
# Using ffprobe
ffprobe -v quiet -print_format json -show_format audio.aaud

# Using aimf
aimf info audio.aaud --simple
```

---

## Format AVID (AI Video)

**Container:** MP4 (ISO/IEC 14496-12)  
**Box Type:** `uuid`  
**UUID:** `61 76 69 64 2D 6D 65 74 61 2D 62 6F 78 00 00 00` ("avid-meta-box\0\0\0")  
**Box Location:** After `moov` box  
**Data:** Raw CBOR

### MP4 Structure

```
ftyp box (file type)
├── moov box (metadata)
│   ├── mvhd (movie header)
│   ├── trak (video track)
│   └── trak (audio track)
├── uuid box ← NEW (after moov)
│   ├── UUID = "avid-meta-box\0\0\0"
│   └── data = CBOR AiContainer
└── mdat box (media data)
```

### UUID Box Format

```
┌──────────────┬──────────────┬──────────────────────────────────────────┬──────────────────────────┐
│ Size (4)     │ 'uuid' (4)   │ UUID (16 bytes)                          │ Data (Size-24)           │
│              │              │ "avid-meta-box\0\0\0"                    │ CBOR AiContainer         │
└──────────────┴──────────────┴──────────────────────────────────────────┴──────────────────────────┘
```

### Why after moov?
- Simpler implementation (no need to rebuild moov)
- Works with existing moov structure
- Players ignore unknown boxes anywhere

### Compatibility

- ✅ Any MP4 player (VLC, ffplay, QuickTime) plays the video
- ✅ YouTube accepts `.avid` files (they're valid MP4)
- ✅ Unknown UUID boxes are ignored by standard parsers

### Example: Extract metadata

```bash
# Using MP4Box
MP4Box -info video.avid

# Using ffprobe
ffprobe -v quiet -print_format json -show_format video.avid

# Using aimf
aimf info video.avid --simple
```

---

## Cryptographic Signing

### Ed25519 Integration

```
┌─────────────────────────────────────────────────────────┐
│ 1. Clone AiContainer without signature/public_key       │
│    ↓                                                    │
│ 2. Serialize to CBOR                                    │
│    ↓                                                    │
│ 3. SHA-256 hash of serialized bytes                     │
│    (Already stored in AiContainer.hash)                 │
│    ↓                                                    │
│ 4. Sign hash with Ed25519 private key                   │
│    ↓                                                    │
│ 5. Store signature (64 bytes) + public key (32 bytes)   │
│    in AiMetadata fields                                 │
└─────────────────────────────────────────────────────────┘
```

### Verification

```
1. Extract signature and public_key from AiMetadata
2. Remove them (temporarily) from the struct
3. Re-serialize and compute hash
4. Compare with stored hash (integrity)
5. Verify signature using public key (authenticity)
```

### Signing Data Construction

```rust
pub fn get_signing_data(&self) -> Vec<u8> {
    let signing_metadata = AiMetadata {
        signature: None,
        public_key: None,
        ..self.metadata.clone()
    };
    
    let container_for_signing = AiContainer {
        version: self.version,
        media_type: self.media_type,
        encoding: self.encoding.clone(),
        payload_type: self.payload_type,
        metadata: signing_metadata,
        hash: self.hash,
    };
    
    container_for_signing.serialize().unwrap_or_default()
}
```

### Security Properties

| Property | Mechanism |
|----------|-----------|
| Integrity | SHA-256 hash of payload+metadata |
| Authenticity | Ed25519 signature of the hash |
| Non-repudiation | Public key identifies signer |
| Tamper detection | Hash mismatch = corrupted/modified |

---

## IANA Registration Status

| Format | MIME Type | Status |
|--------|-----------|--------|
| AIMG | `image/prs.aimg` | Proposed (personal tree) |
| AAUD | `audio/prs.aaud` | Proposed (personal tree) |
| AVID | `video/prs.avid` | Proposed (personal tree) |

See: `docs/IANA-APPLICATION.md`

---

## Reference Implementations

| Language | Repository | Status |
|----------|------------|--------|
| Rust | This repo | ✅ Complete |
| Python | (planned) | ❌ Not started |
| JavaScript | (planned) | ❌ Not started |

---

## Conformance Testing

To claim AIMF conformance, an implementation must:

1. Read all three container formats
2. Preserve unknown chunks when round-tripping
3. Verify Ed25519 signatures correctly
4. Extract original media losslessly
5. Support raw CBOR (no base64 for PNG)

Test suite: `cargo test --workspace`

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-15 | Initial specification |
| | | - CBOR instead of JSON |
| | | - Raw CBOR storage (no base64 for PNG) |
| | | - `aiMg` chunk for PNG |
| | | - `AAUD` appended chunk for WAV |
| | | - Fixed UUID `avid-meta-box\0\0\0` for MP4 |
| | | - Ed25519 crypto support |

---

## Appendix A: CBOR Schema (CDDL)

```cddl
AiContainer = {
    version: uint,
    media_type: 0..2,
    encoding: tstr,
    payload_type: 0..3,
    metadata: AiMetadata,
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
```

---

## Appendix B: Magic Bytes & Markers

| Format | Detection Method | Magic/ Marker |
|--------|------------------|----------------|
| AIMG | PNG signature + `aiMg` chunk | `89 50 4E 47 0D 0A 1A 0A` + chunk type `61 69 4D 67` |
| AAUD | RIFF header + `AAUD` chunk | `52 49 46 46` + `57 41 56 45` + chunk ID `41 41 55 44` |
| AVID | `ftyp` box + UUID box | `66 74 79 70` + UUID `61 76 69 64 2D 6D 65 74 61 2D 62 6F 78 00 00 00` |

---

## Appendix C: Example Files

Test files are in `/examples/output/` after running:

```bash
cargo run --example ai_generate_image      # Creates test_image.aimg
cargo run --example ai_generate_audio      # Creates test_audio.aaud
cargo run --example ai_generate_video_simple # Creates test_video.avid
```

---

## References

- [PNG Specification (RFC 2083)](https://datatracker.ietf.org/doc/html/rfc2083)
- [WAV Format (Microsoft)](https://learn.microsoft.com/en-us/windows/win32/multimedia/waveform-audio-file-format)
- [MP4 (ISO/IEC 14496-12)](https://www.iso.org/standard/83102.html)
- [CBOR (RFC 8949)](https://datatracker.ietf.org/doc/html/rfc8949)
- [Ed25519 (RFC 8032)](https://datatracker.ietf.org/doc/html/rfc8032)
```