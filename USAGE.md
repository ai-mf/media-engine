//media-engine/USAGE.md
# AIMF - AI Media Format Tool Suite

A complete solution for embedding, verifying, and managing AI-generated content with cryptographic provenance across audio, image, and video formats.

## Overview

AIMF provides tools to:
- **Embed** AI metadata into media files (model info, parameters, timestamps)
- **Sign** files cryptographically for authenticity verification
- **Verify** file integrity and signature validity
- **Extract** original media from AI containers
- **View** media files with default system players

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                             AIMF                              │
├──────────────┬──────────────┬──────────────┬──────────────────┤
│   aimf       │    aaud      │    aimg      │   avid           │
│  (Universal) │   (Audio)    │   (Image)    │  (Video)         │
├──────────────┼──────────────┼──────────────┼──────────────────┤
│ Auto-detect  │ AAUD with    │ AIMG with    │ AVID UUID  with  │
│ any format   │ WAV marker   │ PNG marker   │ MP4              │
└──────────────┴──────────────┴──────────────┴──────────────────┘
```

## Installation

```bash
git clone https://github.com/ai-mf/media-engine
cd media-engine/ai
cargo build --release
```

The following binaries will be available:
- `aimf` - Universal tool (handles all formats)
- `aaud` - Audio-specific tool (AAUD format)
- `aimg` - Image-specific tool (AIMG format)
- `avid` - Video-specific tool (AVID format)

## Quick Start

### 1. Generate a Signing Key (Optional but Recommended)

```bash
# Generate key pair for signing
cargo run --bin aaud -- gen-key --output private.key

# This creates private.key - KEEP IT SAFE!
```

### 2. Create AI Media Files

#### Audio (AAUD)

```bash
# Create from JSON audio description
echo '{
  "sample_rate": 44100,
  "samples": [0.1, -0.2, 0.3, -0.1, 0.4],
  "channels": 1
}' | cargo run --bin aaud -- json \
  --output my_audio.aaud \
  --model "MusicGen" \
  --version "2.0" \
  --key private.key

# Create from raw PCM (16-bit signed, little-endian)
cat audio.raw | cargo run --bin aaud -- raw \
  --output my_audio.aaud \
  --sample-rate 44100 \
  --channels 2 \
  --model "AudioLDM" \
  --version "1.5"
```

#### Image (AIMG)

```bash
# Create from JSON image description
echo '{
  "width": 1024,
  "height": 768,
  "pixels": [255,0,0, 0,255,0, 0,0,255],
  "format": "rgb8"
}' | cargo run --bin aimg -- json \
  --output my_image.aimg \
  --model "StableDiffusion" \
  --version "3.0" \
  --key private.key

# Create from raw RGB data
cat image.rgb | cargo run --bin aimg -- raw \
  --width 1920 \
  --height 1080 \
  --output my_image.aimg \
  --model "DALL-E" \
  --version "2.0"
```

#### Video (AVID)

```bash
# Create from JSON video description
echo '{
  "width": 1920,
  "height": 1080,
  "fps": 30,
  "frames": [[255,0,0, 0,255,0], [0,0,255, 255,255,0]],
  "audio": {
    "sample_rate": 44100,
    "samples": [0.1, -0.2, 0.3]
  }
}' | cargo run --bin avid -- json \
  --output my_video.avid \
  --model "Sora" \
  --version "1.0" \
  --key private.key

# Create from raw video frames (RGB24)
cat video.raw | cargo run --bin avid -- raw \
  --width 1280 \
  --height 720 \
  --fps 24 \
  --output my_video.avid \
  --model "Runway" \
  --version "2.0"
```

### 3. View Media Information

```bash
# Show metadata (all tools)
cargo run --bin aaud -- info my_audio.aaud
cargo run --bin aimg -- info my_image.aimg
cargo run --bin avid -- info my_video.avid

# Universal tool (auto-detects format)
cargo run --bin aimf -- info my_audio.aaud
cargo run --bin aimf -- info my_image.aimg
```

### 4. Verify Integrity and Signatures

```bash
# Verify file integrity and signature
cargo run --bin aaud -- verify my_audio.aaud

# Output example:
# 🔍 Verification Results
# Integrity Check: ✅ PASS - File has not been modified
# Signature Verification: ✅ VALID
# Overall: ✅ FILE IS VALID
```

### 5. Extract Original Media

```bash
# Extract the original audio/image/video from AI container
cargo run --bin aaud -- extract my_audio.aaud --output extracted_audio.wav
cargo run --bin aimg -- extract my_image.aimg --output extracted_image.png
cargo run --bin avid -- extract my_video.avid --output extracted_video.mp4
```

### 6. View Media with Default Player

```bash
# Opens with system default player
cargo run --bin aaud -- view my_audio.aaud
cargo run --bin aimg -- view my_image.aimg
cargo run --bin avid -- view my_video.avid
```

### 7. Sign Existing Files

```bash
# Add signature to unsigned file
cargo run --bin aaud -- sign \
  --input unsigned.aaud \
  --key private.key \
  --output signed.aaud
```

## Universal Tool (aimf)

The `aimf` binary auto-detects file format (audio/image/video) and works with all media types:

```bash
# Works with any AIMF file
cargo run --bin aimf -- info my_audio.aaud
cargo run --bin aimf -- verify my_image.aimg
cargo run --bin aimf -- extract my_video.avid --output extracted.mp4
cargo run --bin aimf -- view my_audio.aaud

# Create with explicit type
echo '{"sample_rate":44100,"samples":[0.1,0.2]}' | \
  cargo run --bin aimf -- --type audio create \
  --output universal.aaud \
  --model "Universal" \
  --version "1.0"
```

## Complete Examples

### Example 1: Generate, Sign, and Verify Audio

```bash
# 1. Generate key
cargo run --bin aaud -- gen-key --output mykey.key

# 2. Create AI audio with signature
echo '{"sample_rate": 22050, "samples": [0.5, -0.3, 0.1, -0.2, 0.4]}' | \
  cargo run --bin aaud -- json \
  --output melody.aaud \
  --model "MusicGen" \
  --version "1.0" \
  --key mykey.key

# 3. Verify
cargo run --bin aaud -- verify melody.aaud

# 4. Extract and listen
cargo run --bin aaud -- extract melody.aaud --output melody.wav
cargo run --bin aaud -- view melody.aaud
```

### Example 2: Batch Process Images

```bash
# Process all JSON files in directory
for file in *.json; do
  cat "$file" | cargo run --bin aimg -- json \
    --output "${file%.json}.aimg" \
    --model "SDXL" \
    --version "1.0"
done

# Batch verify all .aimg files
cargo run --bin aimg -- batch --input "*.aimg" --verify
```

### Example 3: Video with Audio Track

```bash
# Create video with synchronized audio
echo '{
  "width": 640,
  "height": 480,
  "fps": 25,
  "frames": [...],
  "audio": {
    "sample_rate": 44100,
    "samples": [...]
  }
}' | cargo run --bin avid -- json \
  --output presentation.avid \
  --model "GenVideo" \
  --version "2.0"
```

## Command Reference

### AAUD (Audio)

| Command | Description |
|---------|-------------|
| `json` | Create from JSON audio description |
| `raw` | Create from raw PCM data |
| `info` | Show audio metadata |
| `verify` | Check integrity and signature |
| `extract` | Extract original audio |
| `view` | Play with default player |
| `sign` | Add signature to file |
| `gen-key` | Generate Ed25519 key pair |
| `batch` | Batch process multiple files |

### AIMG (Image)

| Command | Description |
|---------|-------------|
| `json` | Create from JSON image description |
| `raw` | Create from raw RGB data |
| `info` | Show image metadata |
| `verify` | Check integrity and signature |
| `extract` | Extract original image |
| `view` | Open with default viewer |
| `sign` | Add signature to file |
| `gen-key` | Generate Ed25519 key pair |
| `batch` | Batch process multiple files |

### AVID (Video)

| Command | Description |
|---------|-------------|
| `json` | Create from JSON video description |
| `raw` | Create from raw RGB frames |
| `info` | Show video metadata |
| `verify` | Check integrity and signature |
| `extract` | Extract original video |
| `view` | Play with default player |
| `sign` | Add signature to file |
| `gen-key` | Generate Ed25519 key pair |
| `batch` | Batch process multiple files |

### AIMF (Universal)

| Command | Description |
|---------|-------------|
| `create` | Auto-detect format and create |
| `info` | Show metadata (any format) |
| `verify` | Verify (any format) |
| `extract` | Extract (any format) |
| `view` | View (any format) |
| `--type` | Specify media type (audio/image/video) |
| `batch` | Batch process multiple files |

## JSON Input Formats

### Audio JSON
```json
{
  "sample_rate": 44100,
  "channels": 1,
  "samples": [0.1, -0.2, 0.3, -0.1, 0.4]
}
```

### Image JSON
```json
{
  "width": 1024,
  "height": 768,
  "format": "rgb8",
  "pixels": [255, 0, 0, 0, 255, 0, 0, 0, 255]
}
```

### Video JSON
```json
{
  "width": 1920,
  "height": 1080,
  "fps": 30,
  "frames": [
    [255,0,0, 0,255,0],
    [0,0,255, 255,255,0]
  ],
  "audio": {
    "sample_rate": 44100,
    "samples": [0.1, -0.2, 0.3]
  }
}
```

## File Formats

| Tool | Extension | Container | Marker | Purpose            |
|------|-----------|-----------|--------|--------------------|
| AAUD | `.aaud`   | WAV       | AAUD   | AI-generated audio |
| AIMG | `.aimg`   | PNG       | AIMG   | AI-generated image |
| AVID | `.avid`   | MP4       | AVID   | AI-generated video |

## Security Features

- **Ed25519 signatures** for cryptographic verification
- **Content hashing** to detect tampering
- **Timestamping** for creation time provenance
- **Public key extraction** for signer identification

## Troubleshooting

### "Not a valid AIMF file"
Ensure the file was created with the appropriate tool (aaud/aimg/avid) and contains the correct marker.

### Signature verification fails
- Check you're using the correct public key
- File may have been modified after signing
- Verify the signing key matches

### FFmpeg not found (video)
Install FFmpeg for video encoding:
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg

# Windows
choco install ffmpeg

winget install ffmpeg

```

## License

APACHE 2.0 License - See LICENSE file for details

## Contributing

Pull requests welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No warnings (`cargo clippy`)

## Support

- Documentation: [docs.rs/aimf](https://docs.rs/aimf)
- Issues: [GitHub Issues](https://github.com/ai-mf/media-engine/issues)
- Examples: `/examples` directory