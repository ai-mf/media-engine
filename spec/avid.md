# AVID Specification — AI Media Format for Video

**Version:** 1.0  
**Extension:** `.avid`  
**Container:** MP4 (ISO/IEC 14496-12)  
**MIME Type:** `video/prs.avid` (proposed)  
**Status:** ✅ Stable

## Overview

AVID (AI Video) embeds AI provenance metadata into standard MP4 files while maintaining backward compatibility with all MP4 players.

## Why MP4?

- ✅ Universal support (every device, every browser)
- ✅ Streaming capable (moov atom before mdat)
- ✅ Supports UUID boxes for custom metadata
- ✅ Industry standard (ISO/IEC 14496-12)
- ✅ Supports any codec (H.264, H.265, AV1, VP9)

## Format Structure

MP4 files use the ISO Base Media File Format (ISOBMFF) with a box-based structure:

```
MP4 File:
┌─────────────────────────────────────────────────────────────┐
│ ftyp box (file type and compatibility)                      │
├─────────────────────────────────────────────────────────────┤
│ moov box (metadata)                                         │
│ ├── mvhd (movie header)                                     │
│ ├── trak (video track)                                      │
│ └── trak (audio track)                                      │
├─────────────────────────────────────────────────────────────┤
│ uuid box ← NEW (appended AFTER moov)                        │
│   (AVID metadata)                                           │
├─────────────────────────────────────────────────────────────┤
│ mdat box (media data)                                       │
│ ├── video frames                                            │
│ └── audio samples                                           │
└─────────────────────────────────────────────────────────────┘
```

### UUID Box Definition (Actual Implementation)

AVID uses a **UUID box** (type `uuid`) with a **fixed UUID**:

```
UUID Box Structure:
┌──────────────┬──────────────┬──────────────────────────────────────────┐
│ Size (4)     │ Type 'uuid' (4)                                         │
├──────────────┴──────────────┴──────────────────────────────────────────┤
│ UUID (16 bytes): 61 76 69 64 2D 6D 65 74 61 2D 62 6F 78 00 00 00       │
├─────────────────────────────────────────────────────────────────────────┤
│ Data (Size-24 bytes): CBOR-serialized AiContainer                       │
└─────────────────────────────────────────────────────────────────────────┘
```

### UUID Value (Actual)

```
ASCII: "avid-meta-box\0\0\0"
Hex:   61 76 69 64 2D 6D 65 74 61 2D 62 6F 78 00 00 00
```

**Not a random UUID** — it's a fixed ASCII-based identifier for easy debugging and consistency.

### Box Location (Actual Implementation)

Unlike the specification's recommendation, AVID places the UUID box **AFTER the moov box** (not inside it):

```
Actual AVID Structure:
┌──────────────────────────────────────┐
│ ftyp box                             │
├──────────────────────────────────────┤
│ moov box (unchanged)                 │
├──────────────────────────────────────┤
│ uuid box ← NEW (after moov)          │
├──────────────────────────────────────┤
│ mdat box                             │
└──────────────────────────────────────┘
```

**Why after moov?**
- Simpler implementation (no need to parse/rebuild moov)
- Works with existing moov structure
- Players ignore unknown boxes anywhere in the file
- Easy to strip for extraction

## Magic Bytes Detection

### MP4 Signature (all MP4 files)

| Offset | Bytes (hex) | ASCII |
|--------|-------------|-------|
| 4 | `66 74 79 70` | `ftyp` |

### AVID Marker

The AVID marker appears as a UUID box with the fixed UUID:

```
UUID box found → UUID == "avid-meta-box\0\0\0" → this is an AVID file
```

**Detection code (from actual implementation):**

```rust
const AVID_UUID: [u8; 16] = [
    0x61, 0x76, 0x69, 0x64, 0x2d, 0x6d, 0x65, 0x74,
    0x61, 0x2d, 0x62, 0x6f, 0x78, 0x00, 0x00, 0x00
];

pub fn extract_avid_from_mp4(mp4_data: &[u8]) -> Result<AiContainer, VideoCodecError> {
    let mut pos = 0;
    
    while pos + 8 <= mp4_data.len() {
        let box_size = u32::from_be_bytes(mp4_data[pos..pos+4].try_into().unwrap()) as usize;
        let box_type = &mp4_data[pos+4..pos+8];
        
        if box_type == b"uuid" && pos + 24 <= mp4_data.len() {
            let box_uuid = &mp4_data[pos+8..pos+24];
            
            if box_uuid == AVID_UUID {
                let container_bytes = &mp4_data[pos+24..pos+box_size];
                return Ok(AiContainer::deserialize(container_bytes)?);
            }
        }
        
        pos += box_size;
        if box_size == 0 { break; }
    }
    
    Err(VideoCodecError::NoAvidMetadata)
}
```

## Serialization Format

### Step 1: Create AiContainer

```rust
let container = AiContainer {
    media_type: MediaType::Video,  // 2
    encoding: "mp4".to_string(),
    payload_type: PayloadType::Encoded,  // 0
    metadata: AiMetadata {
        model_name: "Sora".to_string(),
        model_version: "1.0".to_string(),
        width: Some(1920),
        height: Some(1080),
        fps: Some(30),
        sample_rate: Some(48000),
        channels: Some(2),
        timestamp: 1705315200,
        // ... other fields
    },
    payload: original_mp4_data,  // Original MP4 bytes
    hash: compute_hash(),
};
```

### Step 2: Serialize to CBOR

```rust
let cbor_bytes = cbor::to_vec(&container);
// Size: typically 200-500 bytes + metadata
```

### Step 3: Create UUID box

```rust
fn build_uuid_box(data: &[u8]) -> Vec<u8> {
    let mut uuid_box = Vec::new();
    let total_size = 8 + 16 + data.len(); // header(8) + UUID(16) + data
    
    uuid_box.extend_from_slice(&(total_size as u32).to_be_bytes());
    uuid_box.extend_from_slice(b"uuid");
    uuid_box.extend_from_slice(&AVID_UUID);
    uuid_box.extend_from_slice(data);
    
    uuid_box
}
```

### Step 4: Insert after moov box

```rust
fn embed_avid_fresh(mp4_data: &[u8], container: &AiContainer) -> Result<Vec<u8>, VideoCodecError> {
    let container_bytes = container.serialize()?;
    let uuid_box = build_uuid_box(&container_bytes);
    
    let moov_pos = find_box(mp4_data, b"moov").ok_or(VideoCodecError::NoMoovBox)?;
    let moov_end = moov_pos + get_box_size(mp4_data, moov_pos);
    
    let mut output = Vec::new();
    output.extend_from_slice(&mp4_data[..moov_end]);
    output.extend_from_slice(&uuid_box);
    output.extend_from_slice(&mp4_data[moov_end..]);
    
    Ok(output)
}
```

### Step 5: Replace existing metadata

If AVID metadata already exists, it's removed and re-added:

```rust
pub fn replace_avid_in_mp4(mp4_data: &[u8], new_container: &AiContainer) -> Result<Vec<u8>, VideoCodecError> {
    let new_container_bytes = new_container.serialize()?;
    let moov_pos = find_box(mp4_data, b"moov").ok_or(VideoCodecError::NoMoovBox)?;
    let moov_end = moov_pos + get_box_size(mp4_data, moov_pos);
    
    let mut result = Vec::new();
    result.extend_from_slice(&mp4_data[..moov_pos]);
    
    // Rebuild moov without old AVID box
    let mut pos = moov_pos;
    while pos < moov_end {
        let box_size = get_box_size(mp4_data, pos);
        let box_type = &mp4_data[pos+4..pos+8];
        
        let is_avid = if box_type == b"uuid" && pos + 24 <= mp4_data.len() {
            &mp4_data[pos+8..pos+24] == AVID_UUID
        } else {
            false
        };
        
        if !is_avid {
            result.extend_from_slice(&mp4_data[pos..pos + box_size]);
        }
        
        pos += box_size;
    }
    
    result.extend_from_slice(&new_uuid_box);
    result.extend_from_slice(&mp4_data[moov_end..]);
    
    Ok(result)
}
```

## Extraction Process

1. Parse MP4 boxes sequentially
2. Find `uuid` box with matching `AVID_UUID`
3. Read CBOR bytes directly (starting at offset 24)
4. CBOR deserialize to AiContainer
5. Verify hash (optional)
6. Verify signature (optional)

## Original MP4 Extraction

To recover the original MP4 without AVID metadata:

```rust
pub fn extract_original_mp4(mp4_data: &[u8]) -> Result<Vec<u8>, VideoCodecError> {
    let moov_pos = find_box(mp4_data, b"moov").ok_or(VideoCodecError::NoMoovBox)?;
    let moov_end = moov_pos + get_box_size(mp4_data, moov_pos);
    
    let mut result = Vec::new();
    result.extend_from_slice(&mp4_data[..moov_pos]);
    
    let mut pos = moov_pos;
    while pos < moov_end {
        let box_size = get_box_size(mp4_data, pos);
        let box_type = &mp4_data[pos+4..pos+8];
        
        let is_avid = if box_type == b"uuid" && pos + 24 <= mp4_data.len() {
            &mp4_data[pos+8..pos+24] == AVID_UUID
        } else {
            false
        };
        
        if !is_avid {
            result.extend_from_slice(&mp4_data[pos..pos + box_size]);
        }
        
        pos += box_size;
    }
    
    result.extend_from_slice(&mp4_data[moov_end..]);
    Ok(result)
}
```

## Compatibility Matrix

| Software | Can open? | Plays video? | Shows metadata? |
|----------|-----------|--------------|-----------------|
| VLC | ✅ | ✅ | ❌ |
| YouTube | ✅ | ✅ | ❌ |
| Chrome (HTML5) | ✅ | ✅ | ❌ |
| Safari | ✅ | ✅ | ❌ |
| Firefox | ✅ | ✅ | ❌ |
| QuickTime | ✅ | ✅ | ❌ |
| ffplay | ✅ | ✅ | ❌ |
| Adobe Premiere | ✅ | ✅ | ❌ |
| AIMF tools | ✅ | ✅ | ✅ |

## Security Considerations

### Box Parsing Safety

Always validate box sizes:

```rust
// DO: Check for infinite loops
let mut pos = 0;
let mut boxes_processed = 0;
while pos < data.len() && boxes_processed < MAX_BOXES {
    let size = read_u32(&data[pos..]);
    if size == 0 || size > data.len() - pos {
        return Err("Invalid box size");
    }
    pos += size;
    boxes_processed += 1;
}

// DO: Validate UUID before processing
if box_uuid == AVID_UUID {
    // Process metadata safely
}
```

### Hash Verification

```rust
let computed_hash = sha256(&payload + &serialized_metadata);
if computed_hash != container.hash {
    return Err("Video integrity check failed");
}
```

## Codec Compatibility

AVID works with any MP4 codec:

| Codec | Support | Notes |
|-------|---------|-------|
| H.264 (AVC) | ✅ Full | Most compatible |
| H.265 (HEVC) | ✅ Full | Better compression |
| AV1 | ✅ Full | Royalty-free |
| VP9 | ✅ Full | YouTube standard |
| AAC | ✅ Full | Audio |
| MP3 | ✅ Full | Audio |
| Opus | ✅ Full | Modern audio |

**Note:** AVID does NOT re-encode video; it preserves original codec.

## Example: Minimal MP4 with AVID

### Box structure

```
[ftyp] size=24
  brand=mp42
  minor=0

[moov] size=1500
  [mvhd] size=108
    creation_time=...
    duration=...
  [trak] size=800
    [tkhd] size=92
    [mdia] ...

[uuid] size=256 ← AVID metadata (after moov)
  uuid=61 76 69 64 2D 6D 65 74 61 2D 62 6F 78 00 00 00
  data=[CBOR...]

[mdat] size=1048576
  [video frames...]
  [audio samples...]
```

## File Size Overhead

| Original MP4 | Metadata | AVID Size | Overhead |
|--------------|----------|-----------|----------|
| 1 MB | 200 B | 1.00 MB | 0.02% |
| 10 MB | 200 B | 10.00 MB | 0.002% |
| 1 GB | 500 B | 1.00 GB | 0.00005% |

## Testing Vectors

### Create Test AVID

```bash
# Generate raw frames + audio and embed
cat frames.rgb audio.pcm | cargo run --bin aimf -- raw \
  --output test.avid \
  --type video \
  --width 1920 \
  --height 1080 \
  --fps 30 \
  --frame-count 300 \
  --sample-rate 48000 \
  --channels 2 \
  --model "Sora" \
  --version "1.0"

# Verify structure
MP4Box -info test.avid

# Extract metadata
cargo run --bin aimf -- info test.avid
```

### Expected Output

```
File: test.avid
Format: AVID (AI Video)
Container: MP4
Model: Sora v1.0
Resolution: 1920x1080
FPS: 30
Duration: 10.000 sec
Codec: H.264
Timestamp: 2024-01-15 12:00:00 UTC
Hash: 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
Signature: Not signed
Valid: ✅ Yes
```

## References

- [ISO/IEC 14496-12 (ISOBMFF)](https://www.iso.org/standard/83102.html)
- [MP4 Registration Authority](https://mp4ra.org/)
- [CBOR Specification (RFC 8949)](https://datatracker.ietf.org/doc/html/rfc8949)

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-15 | Initial specification (UUID box after moov, fixed ASCII UUID `avid-meta-box\0\0\0`) |
```