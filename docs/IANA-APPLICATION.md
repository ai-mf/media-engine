# IANA Media Type Registration Application (Draft)

**Status:** Not yet submitted — internal draft for future standardization.

This document outlines the planned IANA registration for AIMF media types.

## Proposed media types

| Format | Proposed MIME type | File extension |
|--------|-------------------|----------------|
| AIMG   | `image/aimg`       | `.aimg`        |
| AAUD   | `audio/aaud`       | `.aaud`        |
| AVID   | `video/avid`       | `.avid`        |

## Registration template (example for AVID)

```yaml
Type name: video
Subtype name: avid
Required parameters: none
Optional parameters: none
Encoding considerations: binary
Security considerations:
  - Ed25519 signatures provide integrity but do not guarantee safety
  - Malicious AI metadata could contain misleading claims
  - See SECURITY.md in the specification repository
Interoperability considerations:
  - Files remain playable as standard MP4 in any compliant player
  - Unknown chunks are ignored by legacy software
Published specification: https://github.com/ai-mf/media-engine/spec/avid.md
Applications that use this media type:
  - AI Media Engine tools (aimf, avid)
  - VLC, ffplay, any MP4-compatible player (as standard MP4)
Fragment identifier considerations: none
Additional information:
  - Magic number(s): "ftypmp4" at offset 4, then "AVID" UUID box
  - File extension(s): .avid
  - Macintosh file type code: 'AVID'
Person & email address to contact for further information:
  - Ai Media Format Contributors <aimediaformat@gmail.com>
Intended usage: COMMON
Restrictions on usage: none
Author:
  - AI Media Format Contributors
Change controller:
  - AI Media Format GitHub organization

Notes for IANA reviewers

    These are container formats (MP4/PNG/WAV + metadata), not new codecs

    Backward compatibility is a design goal — files remain usable without AIMF support

    Signature support is optional, not required for basic playback

Next steps

    Stabilize format specifications (target: v1.0)

    Publish reference implementations in multiple languages

    Submit to IANA via media-types@iana.org

    Request early allocation from iana-mime@iana.org if needed

This document is a draft and not legally binding.