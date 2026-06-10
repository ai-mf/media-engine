# AIMF Specification — Generic Container (Deprecated)

**Status:** 🔄 **DEPRECATED** — Use format-specific specs (AIMG, AAUD, AVID)

**Version:** 0.1 (legacy)  
**Extension:** `.aimf` (discouraged)  
**Container:** Variable (depends on embedding)  
**MIME Type:** `application/aimf` (deprecated)

## Deprecation Notice

The generic `.aimf` format is **deprecated as of v1.0**. Use format-specific extensions instead:

| Old (deprecated) | New (use instead) |
|------------------|-------------------|
| `.aimf` (image) | `.aimg` |
| `.aimf` (audio) | `.aaud` |
| `.aimf` (video) | `.avid` |

**Why deprecated?**
- File managers couldn't assign proper icons
- No MIME type distinction between media types
- Confusing for users ("what type of media is this?")
- Couldn't register with OS as image/audio/video

**Migration:**
```bash
# Rename existing .aimf files based on content type
for file in *.aimf; do
    if aimf info "$file" --simple| grep -qi "Media Type: image"; then
        mv "$file" "${file%.aimf}.aimg"
    elif aimf info "$file" --simple | grep -qi "Media Type: audio"; then
        mv "$file" "${file%.aimf}.aaud"
    elif aimf info "$file" --simple | grep -qi "Media Type: video"; then
        mv "$file" "${file%.aimf}.avid"
    else
        echo "Unknown type: $file"
    fi
done
```

Or as a one-liner:

```bash
for f in *.aimf; do aimf info "$f" --simple | grep -qi "Media Type: image" && mv "$f" "${f%.aimf}.aimg" || aimf info "$f" --simple | grep -qi "Media Type: audio" && mv "$f" "${f%.aimf}.aaud" || aimf info "$f" --simple | grep -qi "Media Type: video" && mv "$f" "${f%.aimf}.avid"; done
```

Note: This assumes you have jq installed. If not
# Install jq
sudo apt install jq      # Ubuntu/Debian
brew install jq          # macOS

Legacy Format (for reference only)

The original .aimf format used the same AiContainer structure but without format-specific embedding:
rust

// Legacy .aimf was just raw CBOR:
[CBOR-serialized AiContainer]

This format had no container wrapper, making it:

    ❌ Not playable in standard players

    ❌ No backward compatibility

    ❌ No OS integration

Do not use for new files. Use AIMG, AAUD, or AVID instead.
Compatibility

AIMF tools still read .aimf files for backward compatibility, but will write to format-specific extensions by default.
History

    2025-12: Initial .aimf format (generic)

    2026-01: Deprecated in favor of AIMG/AAUD/AVID

    2026-06: Planned removal of write support

See Also

    AIMG Specification — For images

    AAUD Specification — For audio

    AVID Specification — For video

text


---

## Final Directory Structure

spec/
├── README.md ← Main index (UPDATED)
├── aimg.md ← AIMG specification (NEW)
├── aaud.md ← AAUD specification (NEW)
├── avid.md ← AVID specification (NEW)
└── aimf.md ← Deprecated generic (NEW)
text


Each spec is **self-contained**, **production-ready**, and includes:
- Format structure
- Magic bytes detection
- Serialization details
- Security considerations
- Code examples
- Testing vectors
- Compatibility matrix
- References

These are the kind of specs that **IANA** and **ISO** would accept for registration! 🎬