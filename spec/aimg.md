# AIMG Specification — AI Media Format for Images

**Version:** 1.0  
**Extension:** `.aimg`  
**Container:** PNG (Portable Network Graphics)  
**MIME Type:** `image/prs.aimg` (proposed)  
**Status:** ✅ Stable

## Overview

AIMG (AI Media Image) embeds AI provenance metadata into standard PNG files while maintaining backward compatibility with all PNG viewers.

## Why PNG?

- ✅ Universal support (every browser, every OS)
- ✅ Lossless compression
- ✅ Unknown chunks are ignored by viewers
- ✅ Alpha channel support
- ✅ Progressive rendering

## Format Structure

A PNG file consists of a signature followed by chunks:

```
PNG Signature (8 bytes): 89 50 4E 47 0D 0A 1A 0A

Chunks (repeated):
┌──────────────┬──────────────┬──────────────────┬──────────────┐
│ Length (4)   │ Type (4)     │ Data (Length)    │ CRC (4)      │
└──────────────┴──────────────┴──────────────────┴──────────────┘
```

### AIMG Chunk (Actual Implementation)

**Unlike the specification's tEXt chunk approach, AIMG uses a custom chunk type `aiMg` (lowercase 'a' to avoid conflicts with registered chunk types):**

```
AIMG Chunk Structure:
┌──────────────┬──────────────┬──────────────────────────┬──────────────┐
│ Length (4)   │ 'aiMg' (4)   │ CBOR-serialized AiContainer │ CRC (4)     │
└──────────────┴──────────────┴──────────────────────────┴──────────────┘
```

**Chunk Type:** `0x61 0x69 0x4D 0x67` (ASCII: `aiMg`)

**Why `aiMg` instead of `AIMG`?**
- Registered PNG chunks are case-sensitive
- Lowercase first letter indicates "ancillary" (safe to ignore)
- Prevents conflicts with future official chunk types

### Chunk Location (Actual Implementation)

The AIMG chunk is placed **immediately before the IEND chunk** (not after IHDR):

```
PNG File Structure:
┌──────────────────────────────────────┐
│ PNG Signature (8 bytes)              │
├──────────────────────────────────────┤
│ IHDR chunk (image header)            │
├──────────────────────────────────────┤
│ PLTE chunk (palette, optional)       │
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

**Why before IEND?**
- Simpler implementation (just insert before the end marker)
- Preserves existing chunk ordering
- Players ignore unknown chunks regardless of position
- IEND marks the end — anything before it is safe

## Magic Bytes Detection

### PNG Signature (all PNG files)

| Offset | Bytes (hex) | ASCII |
|--------|-------------|-------|
| 0 | `89 50 4E 47 0D 0A 1A 0A` | `‰PNG␍␊␚␊` |

### AIMG Marker

The AIMG marker appears as a custom chunk type `aiMg`:

```
Found chunk with type 'aiMg' → this is an AIMG file
```

**Detection code (from actual implementation):**

```rust
fn extract_aimg_from_png(png_data: &[u8]) -> Result<AiContainer, ImageCodecError> {
    let mut pos = 8;  // After PNG signature
    
    while pos + 12 <= png_data.len() {
        let chunk_len = u32::from_be_bytes(png_data[pos..pos+4].try_into().unwrap()) as usize;
        let chunk_type = &png_data[pos+4..pos+8];
        
        if chunk_type == b"aiMg" {
            let data_start = pos + 8;
            let data_end = data_start + chunk_len;
            let container_bytes = &png_data[data_start..data_end];
            return Ok(AiContainer::deserialize(container_bytes)?);
        }
        
        pos += 8 + chunk_len + 4;  // Skip to next chunk
        if chunk_type == b"IEND" { break; }
    }
    
    Err(ImageCodecError::NoAimgChunk)
}
```

## Serialization Format

### Step 1: Create AiContainer

```rust
let container = AiContainer {
    media_type: MediaType::Image,  // 0
    encoding: "png".to_string(),
    payload_type: PayloadType::Encoded,  // 0
    metadata: AiMetadata {
        model_name: "StableDiffusion".to_string(),
        model_version: "1.5".to_string(),
        width: Some(512),
        height: Some(512),
        timestamp: 1705315200,
        // ... other fields
    },
    payload: original_png_data,  // Original PNG bytes
    hash: compute_hash(),
};
```

### Step 2: Serialize to CBOR

```rust
let cbor_bytes = cbor::to_vec(&container);
// Size: typically 200-500 bytes + metadata
```

### Step 3: Create `aiMg` chunk

**No base64 encoding!** CBOR bytes are stored raw:

```rust
fn build_aimf_chunk(data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::new();
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(b"aiMg");
    chunk.extend_from_slice(data);
    
    // CRC32 of chunk type + data
    let mut crc_data = Vec::new();
    crc_data.extend_from_slice(b"aiMg");
    crc_data.extend_from_slice(data);
    chunk.extend_from_slice(&crc32fast::hash(&crc_data).to_le_bytes());
    
    chunk
}
```

### Step 4: Insert into PNG

The chunk is inserted before IEND:

```rust
fn embed_fresh(png_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, ImageCodecError> {
    let container_bytes = container.serialize()?;
    let iend_pos = find_iend_chunk(png_data)?;
    let new_chunk = build_aimf_chunk(&container_bytes);
    
    let mut result = Vec::with_capacity(png_data.len() + new_chunk.len());
    result.extend_from_slice(&png_data[..iend_pos]);
    result.extend_from_slice(&new_chunk);
    result.extend_from_slice(&png_data[iend_pos..]);
    
    Ok(result)
}
```

## Extraction Process

1. Parse PNG chunks sequentially
2. Find chunk with type `aiMg`
3. Read CBOR bytes directly (no decoding)
4. CBOR deserialize to AiContainer
5. Verify hash (optional)
6. Verify signature (optional)

## Compatibility Matrix

| Software | Can open? | Shows image? | Shows metadata? |
|----------|-----------|--------------|-----------------|
| Web Browser | ✅ | ✅ | ❌ |
| Photoshop | ✅ | ✅ | ❌ |
| GIMP | ✅ | ✅ | ❌ |
| ImageMagick | ✅ | ✅ | ❌ |
| Windows Photos | ✅ | ✅ | ❌ |
| macOS Preview | ✅ | ✅ | ❌ |
| VLC | ✅ | ✅ | ❌ |
| AIMF tools | ✅ | ✅ | ✅ |

## Security Considerations

### Malformed Chunks

Always validate chunk boundaries:

```rust
// DO: Validate chunk size
if chunk_len > MAX_CHUNK_SIZE {
    return Err("Chunk too large");
}

// DO: Check bounds before reading
if pos + 8 + chunk_len + 4 > png_data.len() {
    break;
}
```

### Hash Verification

Always verify the stored hash:

```rust
let computed_hash = sha256(&payload + &serialized_metadata);
if computed_hash != container.hash {
    return Err("File integrity check failed");
}
```

### Legacy Marker Support

For backward compatibility with older AIMG files that used `AIMG` marker:

```rust
fn extract_from_legacy_marker(png_data: &[u8]) -> Result<AiContainer, ImageCodecError> {
    for i in 0..png_data.len().saturating_sub(8) {
        if &png_data[i..i+4] == b"AIMG" {
            // Legacy format: length + raw CBOR
            let len_bytes: [u8; 4] = png_data[i+4..i+8].try_into().unwrap();
            let data_len = u32::from_le_bytes(len_bytes) as usize;
            return Ok(AiContainer::deserialize(&png_data[i+8..i+8+data_len])?);
        }
    }
    Err(ImageCodecError::NoAimgChunk)
}
```

## Example: Minimal PNG with AIMG

### Hex dump (simplified)

```
89 50 4E 47 0D 0A 1A 0A  ← PNG signature

00 00 00 0D              ← IHDR length (13)
49 48 44 52              ← IHDR type
00 00 00 10 00 00 00 10  ← 16x16 image
08 02 00 00 00           ← 8-bit, RGB, no compression
XX XX XX XX              ← CRC

... IDAT chunks ...

00 00 00 1F              ← aiMg length (31)
61 69 4D 67              ← 'aiMg' type
[CBOR metadata...]       ← Raw CBOR (31 bytes)
XX XX XX XX              ← CRC

00 00 00 00              ← IEND length
49 45 4E 44              ← IEND type
AE 42 60 82              ← CRC
```

## File Size Overhead

| Original PNG | Metadata | AIMG Size | Overhead |
|--------------|----------|-----------|----------|
| 100 KB | 200 B | 100.2 KB | 0.2% |
| 1 MB | 200 B | 1.00 MB | 0.02% |
| 10 MB | 200 B | 10.00 MB | 0.002% |

**Note:** No base64 encoding means minimal overhead (raw CBOR bytes only).

## Testing Vectors

### Create Test AIMG

```bash
# Generate raw RGB and embed
cat image.rgb | cargo run --bin aimf -- raw \
  --output test.aimg \
  --type image \
  --width 512 \
  --height 512 \
  --model "StableDiffusion" \
  --version "1.5"

# Verify structure
pngcheck -v test.aimg
# Expected output: ... chunk 'aiMg' at position X ...

# Extract metadata
cargo run --bin aimf -- info test.aimg
```

### Expected Output

```
File: test.aimg
Format: AIMG (AI Image)
Container: PNG
Model: StableDiffusion v1.5
Dimensions: 512x512
Timestamp: 2024-01-15 12:00:00 UTC
Hash: 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
Signature: Not signed
Valid: ✅ Yes
```

## References

- [PNG Specification (RFC 2083)](https://datatracker.ietf.org/doc/html/rfc2083)
- [CBOR Specification (RFC 8949)](https://datatracker.ietf.org/doc/html/rfc8949)
- [PNG Chunk Types](http://www.libpng.org/pub/png/spec/1.2/PNG-Chunks.html)

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-15 | Initial specification (custom `aiMg` chunk, raw CBOR, inserted before IEND) |
```