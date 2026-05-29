# AAUD Specification — AI Media Format for Audio

**Version:** 1.0  
**Extension:** `.aaud`  
**Container:** WAV (RIFF Waveform Audio)  
**MIME Type:** `audio/aaud` (proposed)  
**Status:** ✅ Stable

## Overview

AAUD (AI Audio) embeds AI provenance metadata into standard WAV files while maintaining backward compatibility with all WAV players.

## Why WAV?

- ✅ Universal support (every OS, every audio player)
- ✅ Lossless PCM audio (no quality loss)
- ✅ Supports LIST chunks for arbitrary metadata
- ✅ Simple RIFF structure (easy to parse)
- ✅ Supports up to 4GB files (standard) or 16EB (W64)

## Format Structure

A WAV file uses the RIFF (Resource Interchange File Format) container:
RIFF Header (12 bytes):
┌──────────────┬──────────────┬──────────────┐
│ 'RIFF' (4) │ Size (4) │ 'WAVE' (4) │
└──────────────┴──────────────┴──────────────┘

Chunks (repeated):
┌──────────────┬──────────────┬──────────────┐
│ ID (4) │ Size (4) │ Data (Size) │
└──────────────┴──────────────┴──────────────┘
text


### Required Chunks (all WAV files)

| Chunk ID | Description | Required |
|----------|-------------|----------|
| `fmt ` | Format information | ✅ Yes |
| `data` | Audio samples | ✅ Yes |

### AAUD Chunk

AAUD adds a **LIST chunk** of type `"AAUD"`:

LIST Chunk Structure:
┌──────────────┬──────────────┬──────────────────────────────────┐
│ 'LIST' (4) │ Size (4) │ 'AAUD' (4) + Data (Size-4) │
└──────────────┴──────────────┴──────────────────────────────────┘

Data format:
┌──────────────┬──────────────────────────────────────────────────┐
│ 'AAUD' (4) │ CBOR-serialized AiContainer (Size-4 bytes) │
└──────────────┴──────────────────────────────────────────────────┘
text


### Chunk Location

The AAUD LIST chunk SHOULD be placed **after `fmt `** and **before `data`**:

RIFF Header
├── fmt chunk (audio format)
├── LIST chunk (type="AAUD") ← NEW
├── LIST chunk (type="INFO") (optional, existing)
├── data chunk (audio samples)
└── ... other chunks
text


### Why Before Data?

- Allows players to read metadata without loading audio
- Enables quick verification without decoding samples
- Preserves compatibility (unknown chunks are ignored)

## Magic Bytes Detection

### WAV Signature (all WAV files)

| Offset | Bytes (hex) | ASCII |
|--------|-------------|-------|
| 0 | `52 49 46 46` | `RIFF` |
| 8 | `57 41 56 45` | `WAVE` |

### AAUD Marker

The AAUD marker appears in a LIST chunk:

LIST chunk found → chunk type == "AAUD" → this is an AAUD file
text


**Detection code:**

```python
def is_aaud(data):
    # Check RIFF header
    if data[0:4] != b'RIFF' or data[8:12] != b'WAVE':
        return False
    
    # Parse chunks
    pos = 12  # After RIFF header
    while pos < len(data):
        chunk_id = data[pos:pos+4]
        chunk_size = int.from_bytes(data[pos+4:pos+8], 'little')
        
        if chunk_id == b'LIST':
            list_type = data[pos+8:pos+12]
            if list_type == b'AAUD':
                return True
        
        pos += 8 + chunk_size  # id(4)+size(4)+data
    
    return False

Serialization Format
Step 1: Create AiContainer
rust

let container = AiContainer {
    media_type: MediaType::Audio,  // 1
    encoding: "wav".to_string(),
    payload_type: PayloadType::Encoded,  // 0
    metadata: AiMetadata {
        model_name: "AudioLDM".to_string(),
        model_version: "2.0".to_string(),
        sample_rate: Some(44100),
        channels: Some(2),
        timestamp: 1705315200,
        // ... other fields
    },
    payload: original_wav_data,  // Original WAV bytes
    hash: compute_hash(),
};

Step 2: Serialize to CBOR
rust

let cbor_bytes = cbor::to_vec(&container);
// Size: typically 200-500 bytes + metadata

Step 3: Create LIST chunk

Unlike PNG, WAV LIST chunks store raw binary data (no base64 encoding):
text

LIST chunk:
  ID:     'LIST'
  Size:   4 + len(cbor_bytes)
  Data:   'AAUD' + cbor_bytes

Step 4: Insert into WAV

The LIST chunk is inserted after fmt and before data.
Audio Format Requirements

The underlying WAV audio MUST be:

    Format: PCM (uncompressed) or IEEE float

    Sample rate: Any (8kHz - 384kHz)

    Bit depth: 16-bit, 24-bit, or 32-bit

    Channels: Mono (1), Stereo (2), or multichannel

AI-generated audio using compressed formats (MP3, AAC) should be decoded to PCM before embedding.
Extraction Process

    Parse RIFF chunks

    Find LIST chunk with type "AAUD"

    Read CBOR bytes directly (no decoding)

    CBOR deserialize to AiContainer

    Verify hash (optional)

    Verify signature (optional)

Compatibility Matrix
Software	Can open?	Plays audio?	Shows metadata?
VLC	✅	✅	❌
Windows Media Player	✅	✅	❌
macOS QuickTime	✅	✅	❌
Audacity	✅	✅	❌
ffplay	✅	✅	❌
Adobe Audition	✅	✅	❌
AIMF tools	✅	✅	✅
Security Considerations
Malformed Chunks

WAV files can have arbitrary chunks. Always validate:
rust

// DO: Check chunk size limits
if chunk_size > MAX_CHUNK_SIZE {
    return Err("Chunk too large");
}

// DO: Validate LIST chunk structure
if list_type != b'AAUD' && list_type != b'INFO' {
    // Skip unknown LIST chunks
    continue;
}

// DON'T: Trust chunk sizes blindly
let data = &data[pos+8:pos+8+chunk_size];  // Potential overflow

Hash Verification

Always verify the stored hash:
rust

let computed_hash = sha256(&payload + &serialized_metadata);
if computed_hash != container.hash {
    return Err("Audio integrity check failed");
}

Example: Minimal WAV with AAUD
Hex dump (simplified)
text

52 49 46 46              ← 'RIFF'
XX XX XX XX              ← File size
57 41 56 45              ← 'WAVE'

66 6D 74 20              ← 'fmt ' chunk
10 00 00 00              ← fmt size (16)
01 00                    ← PCM
02 00                    ← 2 channels
44 AC 00 00              ← 44100 Hz
10 B1 02 00              ← byte rate
04 00                    ← block align
10 00                    ← 16 bits

4C 49 53 54              ← 'LIST' chunk
XX XX XX XX              ← LIST size
41 41 55 44              ← 'AAUD' type
[CBOR metadata...]       ← AiContainer

64 61 74 61              ← 'data' chunk
XX XX XX XX              ← data size
[audio samples...]       ← PCM data

File Size Overhead
Original WAV	Metadata	AAUD Size	Overhead
1 MB	200 B	1.00 MB	0.02%
10 MB	200 B	10.00 MB	0.002%
100 MB	500 B	100.00 MB	0.0005%

Raw CBOR (no base64) means minimal overhead.
Large File Support (W64)

For files >4GB, use the W64 (RF64) format extension:
text

W64 Header:
┌──────────────┬──────────────┬──────────────────┐
│ 'riff' (4)   │ -1 (0xFFFFFFFF) │ 'WAVE' (4)       │
└──────────────┴──────────────┴──────────────────┘
┌──────────────┬──────────────────────────────────┐
│ 'ds64' (4)   │ Size (4) + 64-bit sizes          │
└──────────────┴──────────────────────────────────┘

AAUD supports W64 via the same LIST chunk mechanism.
Testing Vectors
Test AAUD File Generation
bash

# Create test AAUD
echo '{"sample_rate":44100,"samples":[0.5,-0.3,0.2]}' | \
  cargo run --bin aimf -- ingest --type audio --output test.aaud --model test --version 1.0

# Verify structure
ffprobe -v quiet -print_format json -show_format test.aaud

# Extract metadata
cargo run --bin aimf -- info test.aaud

Expected Output
text

File: test.aaud
Format: AAUD (AI Audio)
Container: WAV
Model: test v1.0
Sample Rate: 44100 Hz
Channels: 1
Duration: 0.023 sec
Timestamp: 1705315200
Hash: 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
Signature: Not signed
Valid: ✅ Yes

References

    WAV Format (Microsoft)

    RIFF (Resource Interchange File Format)

    RF64 (EBU Tech 3306)

    CBOR Specification (RFC 8949)

Changelog
Version	Date	Changes
1.0	2026-01-15	Initial specification