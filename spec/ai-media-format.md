# AI Media Format (AIMF) Specification

**Version:** 1.0.0  
**Status:** Active  
**Author:** AI Media Format Team  
**Updated:** 2026-01-15

---

## 1. Overview

AI Media Format (AIMF) is a family of binary container formats designed to store AI-generated media together with verifiable provenance metadata, cryptographic signatures, and tamper detection.

The AIMF family consists of:

| Format | Extension | MIME Type | Base Container |
|--------|----------|-----------|----------------|
| AIMG   | `.aimg`  | `image/prs.aimg` | PNG |
| AAUD   | `.aaud`  | `audio/prs.aaud` | WAV |
| AVID   | `.avid`  | `video/prs.avid` | MP4 |

Each format encapsulates:
- A media payload (image, audio, or video)
- AI provenance metadata (model, version, parameters)
- Cryptographic integrity verification (SHA-256 hash)
- Optional Ed25519 digital signatures
- Timestamp provenance

---

## 2. Design Goals

AIMF is designed to:

- ✅ **Provide verifiable AI provenance** — know which AI model created the content
- ✅ **Be backward compatible** — existing media players work normally
- ✅ **Support tamper detection** — detect modifications after creation
- ✅ **Enable cryptographic signing** — prove authenticity of origin
- ✅ **Remain lightweight** — minimal overhead (0.02% for typical files)
- ✅ **Be extensible** — support future metadata fields

---

## 3. Relationship to MIME Types

Each AIMF format maps to a distinct media type:

- `image/prs.aimg` — AI-generated image in PNG container
- `audio/prs.aaud` — AI-generated audio in WAV container
- `video/prs.avid` — AI-generated video in MP4 container

Media types follow the IANA personal tree (`prs.`), pending future standardization.

---

## 4. Common Core Structure

All AIMF formats share a common core data structure serialized using **CBOR** (Concise Binary Object Representation):

### 4.1 AiContainer Core Schema

```rust
struct AiContainer {
    media_type: MediaType,      // 0 = Image, 1 = Audio, 2 = Video
    encoding: String,            // "png", "wav", "mp4"
    payload_type: PayloadType,   // 0 = Encoded, 1 = Raw, 2 = Reference
    metadata: AiMetadata,
    payload: Vec<u8>,            // Original media bytes
    hash: [u8; 32],              // SHA-256 of payload + metadata
    signature: Option<[u8; 64]>, // Ed25519 signature (optional)
    public_key: Option<[u8; 32]>, // Ed25519 public key (optional)
}
```

### 4.2 AiMetadata Structure

```rust
struct AiMetadata {
    model_name: String,           // AI model identifier
    model_version: String,        // Version string
    timestamp: u64,               // Unix timestamp (seconds)
    sample_rate: Option<u32>,     // Audio: samples per second
    channels: Option<u16>,        // Audio: 1 (mono) or 2 (stereo)
    width: Option<u32>,           // Image/Video: width in pixels
    height: Option<u32>,          // Image/Video: height in pixels
    fps: Option<u32>,             // Video: frames per second
    frame_count: Option<u32>,     // Video: total number of frames
    prompt_hash: Option<String>,  // Optional SHA-256 of generation prompt
    custom: Option<HashMap<String, String>>, // Extensible metadata
}
```

### 4.3 CBOR Serialization Example

```json
// JSON representation (for readability only — actual is binary CBOR)
{
  "media_type": 1,
  "encoding": "wav",
  "payload_type": 0,
  "metadata": {
    "model_name": "AudioLDM-v2",
    "model_version": "2.0",
    "timestamp": 1705315200,
    "sample_rate": 44100,
    "channels": 1
  },
  "hash": "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
}
```

---

## 5. Format-Specific Embedding

### 5.1 AIMG (PNG Container)

**Magic Bytes:** None (uses PNG signature `89 50 4E 47 0D 0A 1A 0A`)

**Embedding Method:** Custom `aiMg` chunk inserted **before IEND chunk**

```
PNG File Structure:
┌──────────────────────────────────────┐
│ PNG Signature (8 bytes)              │
├──────────────────────────────────────┤
│ IHDR chunk (image header)            │
├──────────────────────────────────────┤
│ IDAT chunk(s) (image data)           │
├──────────────────────────────────────┤
│ ... other chunks                     │
├──────────────────────────────────────┤
│ aiMg chunk ← NEW (CBOR AiContainer)  │
├──────────────────────────────────────┤
│ IEND chunk (end marker)              │
└──────────────────────────────────────┘
```

**Chunk Format:**
```
┌──────────┬──────────┬──────────────────┬──────────┐
│ Length   │ Type     │ Data             │ CRC32    │
│ (4 bytes)│ (4 bytes)│ (CBOR encoded)   │ (4 bytes)│
│ 0x00000123│ 'aiMg'   │ AiContainer      │ checksum │
└──────────┴──────────┴──────────────────┴──────────┘
```

**Why before IEND?** Players ignore unknown chunks, and IEND marks the end — placing metadata before it ensures compatibility.

### 5.2 AAUD (WAV Container)

**Magic Bytes:** `RIFF....WAVE` (RIFF header)

**Embedding Method:** Custom `AAUD` chunk **appended at end** with RIFF header size updated

```
WAV File Structure:
┌──────────────────────────────────────┐
│ RIFF Header (12 bytes)               │
│ 'RIFF' + file_size + 'WAVE'          │
├──────────────────────────────────────┤
│ fmt chunk (audio format)             │
├──────────────────────────────────────┤
│ data chunk (PCM samples)             │
├──────────────────────────────────────┤
│ ... other chunks (optional)          │
├──────────────────────────────────────┤
│ AAUD chunk ← NEW (appended)          │
│ 'AAUD' + size + CBOR AiContainer     │
└──────────────────────────────────────┘
```

**Chunk Format:**
```
┌──────────┬──────────┬──────────────────┐
│ ID       │ Size     │ Data             │
│ (4 bytes)│ (4 bytes)│ (CBOR encoded)   │
│ 'AAUD'   │ N        │ AiContainer      │
└──────────┴──────────┴──────────────────┘
```

**Why appended?** Simplest implementation — just update the RIFF header size and append.

### 5.3 AVID (MP4 Container)

**Magic Bytes:** `ftyp` box (e.g., `66 74 79 70 6D 70 34 32` for MP4)

**Embedding Method:** Custom `uuid` box inserted **inside or after moov box**

```
MP4 File Structure:
┌──────────────────────────────────────┐
│ ftyp box (file type)                 │
├──────────────────────────────────────┤
│ moov box (metadata)                  │
│   ├── mvhd (movie header)            │
│   ├── trak (track)                   │
│   └── uuid box ← NEW                 │
│        (AVID metadata)               │
├──────────────────────────────────────┤
│ mdat box (media data)                │
└──────────────────────────────────────┘
```

**UUID Box Format:**
```
┌──────────┬──────────┬──────────────────────────┬──────────────┐
│ Size     │ Type     │ UUID                     │ Data         │
│ (4 bytes)│ (4 bytes)│ (16 bytes)               │ (CBOR)       │
│ N        │ 'uuid'   │ avid-meta-box\0\0\0      │ AiContainer  │
└──────────┴──────────┴──────────────────────────┴──────────────┘
```

**Fixed UUID:** `61 76 69 64 2D 6D 65 74 61 2D 62 6F 78 00 00 00` ("avid-meta-box")

---

## 6. Cryptographic Integrity

### 6.1 Hash Calculation

The SHA-256 hash is computed over:

```
hash_input = payload_bytes + serialized_metadata
```

Where:
- `payload_bytes` = original media bytes (without AIMF wrapper)
- `serialized_metadata` = CBOR-encoded AiMetadata (without payload field)

```rust
let mut hasher = Sha256::new();
hasher.update(&container.payload);
hasher.update(&metadata_bytes);
let hash = hasher.finalize();
```

### 6.2 Hash Verification

On verification:
1. Extract `payload` and `metadata` from container
2. Recompute hash using same method
3. Compare with stored `hash` field
4. If mismatch → file has been modified

---

## 7. Digital Signatures (Ed25519)

### 7.1 Signing Process

```rust
// 1. Create container with populated metadata and payload
let mut container = AiContainer { /* ... */ };

// 2. Compute hash (required before signing)
container.hash = compute_hash(&container);

// 3. Sign the hash (not the whole container)
let signature = private_key.sign(&container.hash);
container.signature = Some(signature.to_bytes());
container.public_key = Some(public_key.to_bytes());
```

### 7.2 Signature Verification

```rust
// 1. Verify hash first (tamper detection)
if recomputed_hash != container.hash {
    return Err("Hash mismatch - file modified");
}

// 2. Verify signature (authenticity)
let public_key = PublicKey::from_bytes(&container.public_key)?;
public_key.verify(&container.hash, &container.signature)?;
```

### 7.3 Security Properties

| Property | Description |
|----------|-------------|
| **128-bit security** | Equivalent to AES-128 |
| **No trusted third party** | Pure public-key cryptography |
| **Fast verification** | ~100,000 verifications/second |
| **Small signatures** | 64 bytes per signature |

---

## 8. Magic Numbers & File Detection

| Format | Detection Method | Magic Bytes |
|--------|------------------|-------------|
| AIMG | PNG signature + `aiMg` chunk | `89 50 4E 47 0D 0A 1A 0A` |
| AAUD | RIFF header + `AAUD` chunk | `52 49 46 46` + `57 41 56 45` |
| AVID | `ftyp` box + `uuid` box | `66 74 79 70` + `61 76 69 64` |

### Detection Priority:

1. Check file extension (`.aimg`, `.aaud`, `.avid`)
2. Parse container format (PNG/WAV/MP4)
3. Look for format-specific marker:
   - PNG: scan for `aiMg` chunk before IEND
   - WAV: scan chunks for `AAUD` ID
   - MP4: scan boxes for `uuid` with matching UUID

---

## 9. Extensibility

### 9.1 Custom Metadata Fields

The `custom` field in `AiMetadata` allows arbitrary key-value pairs:

```rust
custom: Some(HashMap::from([
    ("temperature".to_string(), "0.8".to_string()),
    ("top_p".to_string(), "0.9".to_string()),
    ("seed".to_string(), "42".to_string()),
]))
```

### 9.2 Future Versions

Version `0x01` supports:
- Current CBOR schema
- Ed25519 signatures
- SHA-256 hashes

Planned for v2.0:
- Multiple signatures (collaborative AI)
- Key revocation lists
- Streaming API for large files
- WASM bindings

---

## 10. Complete Example

### Creating an AIMG File Programmatically

```rust
use aimf_core::{AiContainer, AiMetadata, MediaType, PayloadType};

let metadata = AiMetadata {
    model_name: "StableDiffusion-v1.5".to_string(),
    model_version: "2024-01-15".to_string(),
    timestamp: 1705315200,
    sample_rate: None,
    channels: None,
    width: Some(512),
    height: Some(512),
    fps: None,
    frame_count: None,
    prompt_hash: Some("5e884898da28047151d0e56f8dc62927".to_string()),
    custom: None,
};

let container = AiContainer {
    media_type: MediaType::Image,
    encoding: "png".to_string(),
    payload_type: PayloadType::Encoded,
    metadata,
    payload: png_bytes,  // Original PNG data
    hash: [0u8; 32],     // Will be computed
    signature: None,
    public_key: None,
};

// Embed into PNG
let aimg_bytes = embed_aimg_into_png(&png_bytes, &container)?;
```

### Verifying a File

```bash
$ aimf verify image.aimg --simple
$ aimf verify image.aimg --json

🔍 Verification Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Integrity Check:     ✅ PASS - File has not been modified
Signature:           ⚠️  NOT SIGNED
Metadata:            ✅ Valid
Model:               StableDiffusion-v1.5
Timestamp:           2024-01-15 12:00:00 UTC
Overall:             ⚠️  FILE IS UNSIGNED (but valid)
```

---

## 11. IANA Media Type Registration (Proposed)

### 11.1 image/prs.aimg

- **Type name:** image
- **Subtype name:** prs.aimg
- **Required parameters:** none
- **Optional parameters:** none
- **Magic number(s):** PNG signature + `aiMg` chunk
- **File extension(s):** .aimg
- **Macintosh file type code:** PNGf

### 11.2 audio/prs.aaud

- **Type name:** audio
- **Subtype name:** prs.aaud
- **Required parameters:** none
- **Optional parameters:** none
- **Magic number(s):** RIFF header + `AAUD` chunk
- **File extension(s):** .aaud

### 11.3 video/prs.avid

- **Type name:** video
- **Subtype name:** prs.avid
- **Required parameters:** none
- **Optional parameters:** none
- **Magic number(s):** `ftyp` box + `uuid` box
- **File extension(s):** .avid

---

## 12. References

- [CBOR Specification (RFC 8949)](https://datatracker.ietf.org/doc/html/rfc8949)
- [Ed25519 Signatures (RFC 8032)](https://datatracker.ietf.org/doc/html/rfc8032)
- [PNG Specification (RFC 2083)](https://datatracker.ietf.org/doc/html/rfc2083)
- [WAV Format (Microsoft)](https://learn.microsoft.com/en-us/windows/win32/multimedia/waveform-audio-file-format)
- [MP4 Format (ISO/IEC 14496-14)](https://www.iso.org/standard/75929.html)

---

## 13. Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-01-15 | Initial specification |
| | | - CBOR instead of JSON |
| | | - Ed25519 crypto support |
| | | - Format-specific embedding details |
| | | - IANA registration templates |

---

**Appendix A: CBOR Tag Assignments (Proposed)**

| Tag | Meaning |
|-----|---------|
| 300 | AiContainer (AIMF core) |
| 301 | AiMetadata (AIMF metadata) |

**Appendix B: Example CBOR Hex Dump**

```
A4                          # map of 4 pairs
   68 6D 65 64 69 61 5F 74 # "media_type"
   79 70 65                #
   00                       # 0 (Image)
   68 65 6E 63 6F 64 69 6E # "encoding"
   67                       #
   63 70 6E 67              # "png"
   ...
```