# AIMG Specification — AI Media Format for Images

**Version:** 1.0  
**Extension:** `.aimg`  
**Container:** PNG (Portable Network Graphics)  
**MIME Type:** `image/aimg` (proposed)  
**Status:** ✅ Stable

## Overview

AIMG (AI Media Image) embeds AI provenance metadata into standard PNG files while maintaining backward compatibility with all PNG viewers.

## Why PNG?

- ✅ Universal support (every browser, every OS)
- ✅ Lossless compression
- ✅ Supports tEXt chunks for arbitrary metadata
- ✅ Alpha channel support
- ✅ Progressive rendering

## Format Structure

A PNG file consists of a signature followed by chunks:
PNG Signature (8 bytes): 137 80 78 71 13 10 26 10

Chunks (repeated):
┌──────────────┬──────────────┬──────────────────┬──────────────┐
│ Length (4) │ Type (4) │ Data (Length) │ CRC (4) │
└──────────────┴──────────────┴──────────────────┴──────────────┘
text


### AIMG Chunk Insertion

AIMG adds a **tEXt chunk** with keyword `"AIMG"`:

Chunk Length: variable
Chunk Type: tEXt (0x74455874)
Chunk Data:
Keyword: "AIMG" + NULL (0x00)
Text: base64(cbor(AiContainer))
CRC: CRC32 of Type + Data
text


### Chunk Location

The AIMG tEXt chunk SHOULD be placed **after IHDR** and **before IDAT** chunks:

PNG Signature
├── IHDR (image header)
├── tEXt "AIMG" ← NEW
├── tEXt "Software" (optional, existing)
├── tEXt "Description" (optional, existing)
├── IDAT (image data)
│ ├── IDAT chunk 1
│ ├── IDAT chunk 2
│ └── ...
└── IEND (end marker)
text


### Why Before IDAT?

- Allows streaming decoders to read metadata without loading image data
- Enables quick verification without decoding pixels
- Preserves compatibility (unknown chunks are ignored)

## Magic Bytes Detection

### PNG Signature (all PNG files)

| Offset | Bytes (hex) | ASCII |
|--------|-------------|-------|
| 0 | `89 50 4E 47 0D 0A 1A 0A` | `‰PNG␍␊␚␊` |

### AIMG Marker

The AIMG marker appears in the tEXt chunk at variable offset:

tEXt chunk found → keyword == "AIMG" → this is an AIMG file
text


**Detection code:**

```python
def is_aimg(data):
    # Check PNG signature
    if not data.startswith(b'\x89PNG\r\n\x1a\n'):
        return False
    
    # Parse chunks
    pos = 8  # After signature
    while pos < len(data):
        length = int.from_bytes(data[pos:pos+4], 'big')
        chunk_type = data[pos+4:pos+8]
        
        if chunk_type == b'tEXt':
            keyword_end = data[pos+8:pos+8+length].find(b'\x00')
            keyword = data[pos+8:pos+8+keyword_end]
            if keyword == b'AIMG':
                return True
        
        pos += 12 + length  # length(4)+type(4)+crc(4)+data
    
    return False

Serialization Format
Step 1: Create AiContainer
rust

let container = AiContainer {
    media_type: MediaType::Image,  // 0
    encoding: "png".to_string(),
    payload_type: PayloadType::Encoded,  // 0
    metadata: AiMetadata {
        model_name: "StableDiffusion".to_string(),
        model_version: "1.5".to_string(),
        timestamp: 1705315200,
        // ... other fields
    },
    payload: original_png_data,  // Original PNG bytes
    hash: compute_hash(),
};

Step 2: Serialize to CBOR
rust

let cbor_bytes = cbor::to_vec(&container);
// Size: typically 200-500 bytes + metadata

Step 3: Base64 Encode (for PNG tEXt chunk)

PNG tEXt chunks require Latin-1 (ISO-8859-1) text. CBOR bytes must be base64-encoded:
rust

let base64_text = base64::encode(&cbor_bytes);
// Base64 increases size by ~33%

Step 4: Create tEXt chunk
text

Keyword: "AIMG\0"
Text:    base64(cbor(AiContainer))
Length:  4 + len(base64_text) + 1 (null terminator)

Step 5: Insert into PNG

The tEXt chunk is inserted between IHDR and IDAT.
Extraction Process

    Parse PNG chunks

    Find tEXt chunk with keyword "AIMG"

    Base64 decode the text field

    CBOR deserialize to AiContainer

    Verify hash (optional)

    Verify signature (optional)

Compatibility Matrix
Software	Can open?	Shows image?	Shows metadata?
Web Browser	✅	✅	❌
Photoshop	✅	✅	❌
GIMP	✅	✅	❌ (ignores tEXt)
ImageMagick	✅	✅	❌
Windows Photos	✅	✅	❌
macOS Preview	✅	✅	❌
VLC	✅	✅	❌
AIMF tools	✅	✅	✅
Security Considerations
Injection Attacks

Malicious PNGs could craft tEXt chunks that look like AIMG but contain arbitrary data. Always validate:
rust

// DO: Validate size and format
if text.len() > MAX_AIMG_SIZE {
    return Err("AIMG metadata too large");
}

// DO: Validate CBOR structure
let container: AiContainer = cbor::from_slice(&decoded)?;

// DON'T: Assume data is safe
let container: AiContainer = unsafe { cbor::from_slice_unchecked(&decoded) };

Hash Verification

Always verify the stored hash against recomputed hash:
rust

let computed_hash = sha256(&payload + &serialized_metadata);
if computed_hash != container.hash {
    return Err("File integrity check failed");
}

Example: Minimal PNG with AIMG
Hex dump (simplified)
text

89 50 4E 47 0D 0A 1A 0A  ← PNG signature
00 00 00 0D              ← IHDR length (13)
49 48 44 52              ← IHDR type
00 00 00 10 00 00 00 10  ← 16x16 image
08 02 00 00 00           ← 8-bit, RGB, no compression
XX XX XX XX              ← CRC

00 00 00 3F              ← tEXt length (63)
74 45 58 74              ← tEXt type
41 49 4D 47 00           ← "AIMG\0"
[base64 encoded CBOR...] ← metadata
XX XX XX XX              ← CRC

... IDAT chunks ...
00 00 00 00              ← IEND length
49 45 4E 44              ← IEND type
AE 42 60 82              ← CRC

File Size Overhead
Original PNG	Metadata	AIMG Size	Overhead
100 KB	200 B	100.3 KB	0.3%
1 MB	200 B	1.00 MB	0.02%
10 MB	200 B	10.00 MB	0.002%

Base64 encoding adds ~33% overhead to metadata only, not the entire file.
Testing Vectors
Test AIMG File Generation
bash

# Create test AIMG
echo '{"width":10,"height":10,"pixels":[0,0,0]}' | \
  cargo run --bin aimf -- ingest --type image --output test.aimg --model test --version 1.0

# Verify structure
pngcheck -v test.aimg
# Expected output: ... tEXt 'AIMG' ...

# Extract metadata
cargo run --bin aimf -- info test.aimg

Expected Output
text

File: test.aimg
Format: AIMG (AI Image)
Container: PNG
Model: test v1.0
Timestamp: 1705315200
Hash: 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
Signature: Not signed
Valid: ✅ Yes

References

    PNG Specification (RFC 2083)

    CBOR Specification (RFC 8949)

    Base64 (RFC 4648)

Changelog
Version	Date	Changes
1.0	2026-01-15	Initial specification