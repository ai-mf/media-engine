//media-engine/spec/ai-media-format.md
# AI Media Format (AIMF) Specification
Version: 0.1.0  
Status: Draft  
Author: AI MEDIA FORMAT

---

## 1. Overview

AI Media Format (AIMF) is a family of binary container formats designed to store AI-generated media together with verifiable provenance metadata.

The AIMF family consists of:

| Format | Extension | MIME Type |
|--------|----------|----------|
| AIMG   | .aimg    | image/prs.aimg |
| AAUD   | .aaud    | audio/prs.aaud |
| AVID   | .avid    | video/prs.avid |

Each format encapsulates:
- A media payload (image, audio, or video)
- AI provenance metadata
- Integrity verification data

---

## 2. Design Goals

AIMF is designed to:

- Provide **verifiable AI provenance**
- Be **backward compatible** with existing media formats
- Support **tamper detection**
- Enable **future cryptographic signing**
- Remain **lightweight and extensible**

---

## 3. Relationship to MIME Types

Each AIMF format maps to a distinct media type:

- `image/prs.aimg`
- `audio/prs.aaud`
- `video/prs.avid`

Media types follow the IANA personal tree (`prs.`), which allows custom formats pending standardization. :contentReference[oaicite:0]{index=0}

---

## 4. Layout Overview

An AIMF file consists of a fixed-size binary preamble, followed by a variable-length JSON metadata block, and terminates with the raw binary media stream. This streaming-friendly structure allows players to parse metadata instantly without reading the entire file.


---

## 5. Magic Numbers

| Format | Magic (ASCII) | Hex        |
|--------|--------------|------------|
| AIMG   | AIMG         | 41 49 4D 47 |
| AAUD   | AAUD         | 41 41 55 44 |
| AVID   | AVID         | 41 56 49 44 |

---

## 6. Structure Details

### 6.1 Version (1 byte)
Specifies the format revision. The current version is `0x01`.

### 6.2 Flags (1 byte)
Bitmask for format options (e.g., encryption, compression status). Default is `0x00`.

Bitmask defining features:

| Bit | Meaning |
|-----|--------|
| 0   | Metadata present |
| 1   | Integrity hash present |
| 2   | Signature present |
| 3-7 | Reserved |


### 6.3 Header Length (4 bytes)
An unsigned 32-bit integer (Big-Endian) indicating the exact size of the **7. Metadata Section** in bytes.

---

## 7. Metadata Section

The metadata section is encoded strictly as UTF-8 JSON. It provides explicit structural context about the underlying media stream. Unlike the binary sections, this header is variable in length and must match the byte count specified in the **Header Length** field.

### 7.1 JSON Schema Definition

```json
{
  "$schema": "https://json-schema.org",
  "title": "AIMFMetadata",
  "type": "object",
  "properties": {
    "media_type": {
      "type": "string",
      "enum": ["image", "audio", "video"]
    },
    "codec": {
      "type": "string"
    },
    "hash": {
      "type": "string",
      "description": "Hexadecimal cryptographic hash of the subsequent raw payload"
    },
    "custom_metadata": {
      "type": "object",
      "description": "Arbitrary key-value pairs for AI model tags or processing data"
    }
  },
  "required": ["media_type", "codec", "hash"]
}
```

### 7.2 JSON Structure Example

```json
{
  "media_type": "video",
  "codec": "h264",
  "hash": "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6",
  "custom_metadata": {
    "model_inference": "resnet50",
    "detected_objects": ["person", "car"]
  }
}
```

---

## 8. Media Payload Section

The remainder of the file following the JSON metadata block contains the raw, unencoded binary media payload (e.g., raw JPEG, MP3, or MP4 stream bytes). Parsers read this section continuously until the End-Of-File (EOF) marker is reached