# AI Media Format (AIMF) Specification

**Version:** 0.1.0  
**Status:** Draft  
**Author:** AI MEDIA FORMAT

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

## 2. Design Goals

AIMF is designed to:

- Provide **verifiable AI provenance**
- Be **backward compatible** with existing media formats
- Support **tamper detection**
- Enable **future cryptographic signing**
- Remain **lightweight and extensible**

## 3. Relationship to MIME Types

Each AIMF format maps to a distinct media type:

- `image/prs.aimg`
- `audio/prs.aaud`
- `video/prs.avid`

Media types follow the IANA personal tree (`prs.`), which allows custom formats pending standardization.

## 4. Layout Overview

An AIMF file consists of a fixed-size binary preamble, followed by a variable-length JSON metadata block, and terminates with the raw binary media stream. This streaming-friendly structure allows players to parse metadata instantly without reading the entire file.

## 5. Magic Numbers

| Format | Magic (ASCII) | Hex        |
|--------|--------------|------------|
| AIMG   | AIMG         | 41 49 4D 47 |
| AAUD   | AAUD         | 41 41 55 44 |
| AVID   | AVID         | 41 56 49 44 |

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
An unsigned 32-bit integer (Big-Endian) indicating the exact size of the **Metadata Section** in bytes.

## 7. Metadata Section

The metadata section is encoded strictly as UTF-8 JSON. It provides explicit structural context about the underlying media stream.

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