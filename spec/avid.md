# AVID Specification — AI Media Format for Video

**Version:** 1.0  
**Extension:** `.avid`  
**Container:** MP4 (ISO/IEC 14496-12)  
**MIME Type:** `video/avid` (proposed)  
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
MP4 File:
┌─────────────────────────────────────────────────────────────┐
│ ftyp box (file type and compatibility) │
├─────────────────────────────────────────────────────────────┤
│ moov box (metadata) │
│ ├── mvhd (movie header) │
│ ├── trak (video track) │
│ ├── trak (audio track) │
│ └── uuid (custom data) ← NEW │
├─────────────────────────────────────────────────────────────┤
│ mdat box (media data) │
│ ├── video frames │
│ └── audio samples │
└─────────────────────────────────────────────────────────────┘
text


### UUID Box Definition

AVID uses a **UUID box** (type `uuid`) with a registered UUID:

UUID Box Structure:
┌──────────────┬──────────────┬──────────────────────────────────┐
│ Size (4) │ Type 'uuid' (4) │
├──────────────┴──────────────┴──────────────────────────────────┤
│ UUID (16 bytes): A1B2C3D4-E5F6-4789-AB12-CD34EF567890 │
├─────────────────────────────────────────────────────────────────┤
│ Data (Size-24 bytes): CBOR-serialized AiContainer │
└─────────────────────────────────────────────────────────────────┘
text


### UUID Value

A1B2C3D4-E5F6-4789-AB12-CD34EF567890
text


This UUID is registered for private use. To avoid conflicts, it SHOULD be generated from your domain name:

```python
import uuid

# Generate from domain (example)
namespace = uuid.NAMESPACE_DNS
name = 'com.aimf.avid'
avid_uuid = uuid.uuid5(namespace, name)
# Result: a1b2c3d4-e5f6-4789-ab12-cd34ef567890

Box Location

The UUID box SHOULD be placed in the moov box (movie metadata) for quick access:
text

moov box
├── mvhd
├── trak (video)
├── trak (audio)
├── udta (user data)
└── uuid (AVID) ← NEW

Why in moov?

    Players read moov first (before mdat)

    Enables metadata access without loading video data

    Streaming servers can extract metadata from header

    Fast verification without downloading entire file

Magic Bytes Detection
MP4 Signature (all MP4 files)
Offset	Bytes (hex)	ASCII
4	66 74 79 70	ftyp
AVID Marker

The AVID marker appears as a UUID box with the registered UUID:
text

UUID box found → UUID == AVID_UUID → this is an AVID file

Detection code:
python

def is_avid(data):
    # Find ftyp box
    if data[4:8] != b'ftyp':
        return False
    
    # Parse boxes
    pos = 0
    while pos < len(data):
        box_size = int.from_bytes(data[pos:pos+4], 'big')
        box_type = data[pos+4:pos+8]
        
        if box_type == b'uuid' and box_size >= 24:
            box_uuid = data[pos+8:pos+24]
            if box_uuid == avid_uuid:
                return True
        
        pos += box_size
    
    return False

Serialization Format
Step 1: Create AiContainer
rust

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

Step 2: Serialize to CBOR
rust

let cbor_bytes = cbor::to_vec(&container);
// Size: typically 200-500 bytes + metadata

Step 3: Create UUID box
text

UUID Box:
  Size:   24 + len(cbor_bytes)
  Type:   'uuid'
  UUID:   A1B2C3D4-E5F6-4789-AB12-CD34EF567890
  Data:   cbor_bytes

Step 4: Insert into moov box

The UUID box is added as a child of the moov box. If moov doesn't exist (non-streaming file), it should be created.
Streaming Support

AVID supports fast start (moov before mdat) for streaming:
Standard MP4 (not streaming)
text

ftyp | mdat | moov  ← moov at end (bad for streaming)

Fast Start MP4 (streaming)
text

ftyp | moov | mdat  ← moov at beginning (good for streaming)

AVID preserves the existing structure but adds UUID box to moov.
Streaming Server Behavior
text

Client Request → GET /video.avid
Server Response:
  HTTP 206 Partial Content
  Content-Type: video/avid
  
  First few KB contain moov (includes AVID metadata)
  → Client can verify authenticity before downloading video

Extraction Process

    Parse MP4 boxes

    Find uuid box with AVID UUID

    Read CBOR bytes directly

    CBOR deserialize to AiContainer

    Verify hash (optional)

    Verify signature (optional)

Compatibility Matrix
Software	Can open?	Plays video?	Shows metadata?
VLC	✅	✅	❌
YouTube	✅	✅	❌
Chrome (HTML5)	✅	✅	❌
Safari	✅	✅	❌
Firefox	✅	✅	❌
QuickTime	✅	✅	❌
ffplay	✅	✅	❌
Adobe Premiere	✅	✅	❌
AIMF tools	✅	✅	✅
Security Considerations
Box Parsing Safety

MP4 boxes can be nested. Always validate sizes:
rust

// DO: Check for infinite loops
let mut pos = 0;
let mut boxes_processed = 0;
while pos < data.len() && boxes_processed < MAX_BOXES {
    let size = read_u32(&data[pos..]);
    if size == 0 || size > data.len() - pos {
        return Err("Invalid box size");
    }
    // Process box
    pos += size;
    boxes_processed += 1;
}

// DO: Validate UUID
if uuid == AVID_UUID {
    // Process metadata
}

// DON'T: Trust unsigned sizes
let size = unsafe { *(data[pos..pos+4].as_ptr() as *const u32) };

Hash Verification
rust

let computed_hash = sha256(&payload + &serialized_metadata);
if computed_hash != container.hash {
    return Err("Video integrity check failed");
}

Codec Compatibility

AVID works with any MP4 codec:
Codec	Support	Notes
H.264 (AVC)	✅ Full	Most compatible
H.265 (HEVC)	✅ Full	Better compression
AV1	✅ Full	Royalty-free
VP9	✅ Full	YouTube standard
AAC	✅ Full	Audio
MP3	✅ Full	Audio
Opus	✅ Full	Modern audio

AVID does NOT re-encode video; it preserves original codec.
Example: Minimal MP4 with AVID
Box structure
text

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
  
  [uuid] size=256 ← AVID metadata
    uuid=A1B2C3D4-E5F6-4789-AB12-CD34EF567890
    data=[CBOR...]

[mdat] size=1048576
  [video frames...]
  [audio samples...]

File Size Overhead

Original MP4	Metadata	AVID Size	Overhead
1 MB	200 B	1.00 MB	0.02%
10 MB	200 B	10.00 MB	0.002%
1 GB	500 B	1.00 GB	0.00005%

Testing Vectors
Test AVID File Generation
bash

# Create test AVID
echo '{"width":1920,"height":1080,"fps":30,"frames":[[255,0,0]]}' | \
  cargo run --bin aimf -- ingest --type video --output test.avid --model test --version 1.0

# Verify structure
MP4Box -info test.avid

# Extract metadata
cargo run --bin aimf -- info test.avid

Expected Output
text

File: test.avid
Format: AVID (AI Video)
Container: MP4
Model: test v1.0
Resolution: 1920x1080
FPS: 30
Duration: 0.033 sec
Codec: H.264 (assumed)
Timestamp: 1705315200
Hash: 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
Signature: Not signed
Valid: ✅ Yes

References

    ISO/IEC 14496-12 (ISOBMFF)

    MP4 Registration Authority

    UUID Specification (RFC 4122)

    CBOR Specification (RFC 8949)

Changelog
Version	Date	Changes
1.0	2026-01-15	Initial specification
