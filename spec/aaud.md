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

```
RIFF Header (12 bytes):
┌──────────────┬──────────────┬──────────────┐
│ 'RIFF' (4)   │ Size (4)     │ 'WAVE' (4)   │
└──────────────┴──────────────┴──────────────┘

Chunks (repeated):
┌──────────────┬──────────────┬──────────────┐
│ ID (4)       │ Size (4)     │ Data (Size)  │
└──────────────┴──────────────┴──────────────┘
```

### Required Chunks (all WAV files)

| Chunk ID | Description | Required |
|----------|-------------|----------|
| `fmt ` | Format information | ✅ Yes |
| `data` | Audio samples | ✅ Yes |

### AAUD Chunk (Implementation)

**Unlike traditional WAV metadata that uses LIST chunks, AAUD uses a simple custom chunk appended at the end of the file:**

```
AAUD Chunk Structure:
┌──────────────┬──────────────┬──────────────────────────┐
│ 'AAUD' (4)   │ Size (4)     │ CBOR-serialized AiContainer │
└──────────────┴──────────────┴──────────────────────────┘
```

### Chunk Location (Actual Implementation)

The AAUD chunk is **appended after all existing chunks**, and the RIFF header size is updated:

```
RIFF Header
├── fmt chunk (audio format)
├── data chunk (audio samples)
├── ... any other existing chunks
└── AAUD chunk ← APPENDED AT END (not before data)
```

**Why appended?**
- Simpler implementation (no chunk parsing/reordering)
- Works with all WAV files regardless of structure
- Unknown chunks are ignored by players
- RIFF header size update ensures players skip it

## Magic Bytes Detection

### WAV Signature (all WAV files)

| Offset | Bytes (hex) | ASCII |
|--------|-------------|-------|
| 0 | `52 49 46 46` | `RIFF` |
| 8 | `57 41 56 45` | `WAVE` |

### AAUD Marker

The AAUD marker appears as a chunk ID:

```
Found chunk with ID 'AAUD' → this is an AAUD file
```

**Detection code (from actual implementation):**

```rust
fn extract_aaud_from_wav(wav_data: &[u8]) -> Result<AiContainer, AudioCodecError> {
    let mut pos = RIFF_HEADER_SIZE;  // Start after RIFF header
    
    while pos + CHUNK_HEADER_SIZE <= wav_data.len() {
        let chunk_id = &wav_data[pos..pos+4];
        let chunk_size = u32::from_le_bytes(/* ... */) as usize;
        
        if chunk_id == b"AAUD" {
            // Found it!
            let container_bytes = &wav_data[pos+8..pos+8+chunk_size];
            return Ok(AiContainer::deserialize(container_bytes)?);
        }
        
        pos += CHUNK_HEADER_SIZE + chunk_size;
    }
    
    Err(AudioCodecError::NoAaudTag)
}
```

## Serialization Format

### Step 1: Create AiContainer

```rust
let container = AiContainer {
    media_type: MediaType::Audio,
    encoding: "wav".to_string(),
    payload_type: PayloadType::Encoded,
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
```

### Step 2: Serialize to CBOR

```rust
let cbor_bytes = cbor::to_vec(&container);
// Size: typically 200-500 bytes + metadata
```

### Step 3: Create AAUD chunk

```rust
let mut result = wav_data.to_vec();
result.extend_from_slice(b"AAUD");
result.extend_from_slice(&(container_bytes.len() as u32).to_le_bytes());
result.extend_from_slice(&container_bytes);
```

### Step 4: Update RIFF header size

```rust
let new_file_size = (result.len() as u32) - 8;
result[4..8].copy_from_slice(&new_file_size.to_le_bytes());
```

## Audio Format Requirements

The underlying WAV audio MUST be:

- Format: PCM (uncompressed) or IEEE float
- Sample rate: Any (8kHz - 384kHz)
- Bit depth: 16-bit, 24-bit, or 32-bit
- Channels: Mono (1), Stereo (2), or multichannel

**Note:** AI-generated audio using compressed formats (MP3, AAC) should be decoded to PCM before embedding.

## Extraction Process

1. Parse RIFF chunks sequentially
2. Find chunk with ID `"AAUD"`
3. Read CBOR bytes directly
4. CBOR deserialize to AiContainer
5. Verify hash (optional)
6. Verify signature (optional)

## Compatibility Matrix

| Software | Can open? | Plays audio? | Shows metadata? |
|----------|-----------|--------------|-----------------|
| VLC | ✅ | ✅ | ❌ |
| Windows Media Player | ✅ | ✅ | ❌ |
| macOS QuickTime | ✅ | ✅ | ❌ |
| Audacity | ✅ | ✅ | ❌ |
| ffplay | ✅ | ✅ | ❌ |
| Adobe Audition | ✅ | ✅ | ❌ |
| AIMF tools | ✅ | ✅ | ✅ |

## Security Considerations

### Malformed Chunks

Always validate chunk sizes:

```rust
// DO: Check chunk size limits
if chunk_size > MAX_CHUNK_SIZE {
    return Err("Chunk too large");
}

// DO: Validate bounds
if pos + CHUNK_HEADER_SIZE + chunk_size > wav_data.len() {
    break;
}
```

### Hash Verification

Always verify the stored hash:

```rust
let computed_hash = sha256(&payload + &serialized_metadata);
if computed_hash != container.hash {
    return Err("Audio integrity check failed");
}
```

## Example: Minimal WAV with AAUD

### Hex dump (simplified)

```
52 49 46 46              ← 'RIFF'
XX XX XX XX              ← File size (updated to include AAUD)
57 41 56 45              ← 'WAVE'

66 6D 74 20              ← 'fmt ' chunk
10 00 00 00              ← fmt size (16)
01 00                    ← PCM
02 00                    ← 2 channels
44 AC 00 00              ← 44100 Hz
10 B1 02 00              ← byte rate
04 00                    ← block align
10 00                    ← 16 bits

64 61 74 61              ← 'data' chunk
XX XX XX XX              ← data size
[audio samples...]       ← PCM data

41 41 55 44              ← 'AAUD' chunk (appended)
XX XX XX XX              ← AAUD size
[CBOR metadata...]       ← AiContainer
```

## File Size Overhead

| Original WAV | Metadata | AAUD Size | Overhead |
|--------------|----------|-----------|----------|
| 1 MB | 200 B | 1.00 MB | 0.02% |
| 10 MB | 200 B | 10.00 MB | 0.002% |
| 100 MB | 500 B | 100.00 MB | 0.0005% |

Raw CBOR (no base64) means minimal overhead.

## Large File Support (W64)

For files >4GB, use the W64 (RF64) format extension. AAUD supports W64 via the same chunk mechanism.

## Testing Vectors

### Create Test AAUD

```bash
# Generate raw PCM and embed
cat audio.pcm | cargo run --bin aimf -- raw \
  --output test.aaud \
  --type audio \
  --sample-rate 44100 \
  --channels 1 \
  --model test \
  --version 1.0

# Verify structure
ffprobe -v quiet -print_format json -show_format test.aaud

# Extract metadata
cargo run --bin aimf -- info test.aaud
```

### Expected Output

```
File: test.aaud
Format: AAUD (AI Audio)
Container: WAV
Model: test v1.0
Sample Rate: 44100 Hz
Channels: 1
Duration: 1.000 sec
Timestamp: 1705315200
Hash: 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
Signature: Not signed
Valid: ✅ Yes
```

## References

- [WAV Format (Microsoft)](https://learn.microsoft.com/en-us/windows/win32/multimedia/waveform-audio-file-format)
- [RIFF (Resource Interchange File Format)](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/Docs/riffmci.pdf)
- [CBOR Specification (RFC 8949)](https://datatracker.ietf.org/doc/html/rfc8949)

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-15 | Initial specification (chunk appended at end) |
```