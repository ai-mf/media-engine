# AI Media Engine

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A verifiable AI media format system that embeds provenance metadata into existing media files while remaining backward compatible with standard media players (VLC, ffplay, etc.).

## Overview

AI Media Engine provides a family of container formats (AIMG, AAUD, AVID) that wrap standard media files with AI provenance metadata. The formats are designed to be:

- **Backward compatible**: `.avid` files play directly in VLC as normal MP4 files
- **Verifiable**: Each file contains cryptographic hashes for integrity checking
- **Extensible**: JSON metadata allows arbitrary AI model information
- **Streamable**: Header-first design enables quick metadata access

### Format Family

| Format | Extension | Media Type | Base Format | Plays in VLC |
|--------|-----------|------------|-------------|--------------|
| AIMG | `.aimg` | Image | PNG | ✅ Yes (as PNG) |
| AAUD | `.aaud` | Audio | WAV | ✅ Yes (as WAV) |
| AVID | `.avid` | Video | MP4 | ✅ Yes (as MP4) |

## Quick Start

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y ffmpeg libavcodec-dev libavformat-dev libavutil-dev

# For image support
sudo apt-get install -y libpng-dev libjpeg-dev

# For audio support  
sudo apt-get install -y libmp3lame-dev

Build from Source
bash

# Clone the repository
git clone https://github.com/yourusername/media-engine.git
cd media-engine/ai

# Build all components in release mode
cargo build --release

# Generate key pair
cargo run --bin aimf -- gen-key --output private.key


# Build individual tools
cargo build --bin aimf   # Main tool (recommended)
cargo build --bin aimg   # Image-specific tool
cargo build --bin aaud   # Audio-specific tool
cargo build --bin avid   # Video-specific tool

Install CLI Tools
bash

# Install all tools to ~/.cargo/bin
cargo install --path tools/cli

# Now you can run from anywhere
aimf --help
aimg --help
aaud --help
avid --help

Usage Examples
Method 1: Using Examples (Quick Demo)

Generate AI media files with simulated content:
bash

# Generate a 10-second test video (gradient + sine wave)
cargo run --example ai_generate_video_simple

# Generate a synthetic image
cargo run --example ai_generate_image

# Generate synthetic audio (440Hz sine wave)
cargo run --example ai_generate_audio

# View the generated files (opens in default player)
cargo run --bin aimf -- view test_video_10sec.avid
cargo run --bin aimf -- view test_image.aimg
cargo run --bin aimf -- view test_audio.aaud

Method 2: Creating AI Media from JSON

Feed JSON data to aimf ingest to create AI media files:
bash

# Create an image from JSON description
echo '{
  "width": 800,
  "height": 600,
  "pixels": [255,0,0, 0,255,0, 0,0,255]
}' | cargo run --bin aimf -- ingest \
    --output my_image.aimg \
    --model "stable-diffusion" \
    --version "1.5"

# Create audio from JSON samples
echo '{
  "sample_rate": 44100,
  "samples": [0.5, -0.3, 0.2, -0.1]
}' | cargo run --bin aimf -- ingest \
    --output my_audio.aaud \
    --model "audio-gan" \
    --version "2.0"

# Create video from JSON frames
echo '{
  "width": 320,
  "height": 240,
  "fps": 30,
  "frames": [[255,0,0, 0,255,0], [0,0,255, 255,255,0]]
}' | cargo run --bin aimf -- ingest \
    --output my_video.avid \
    --model "video-diffusion" \
    --version "1.0"

Method 3: Converting Existing Media to AI Format
bash

# Convert existing PNG to AIMG (embeds metadata)
aimg create input.png --output output.aimg --model "my-model" --version "1.0"

# Convert existing MP3 to AAUD  
aaud create input.mp3 --output output.aaud --model "my-model" --version "1.0"

# Convert existing MP4 to AVID
avid create input.mp4 --output output.avid --model "my-model" --version "1.0"

Method 4: Raw Data Conversion (from stdin)
bash

# Convert raw RGB frames to AVID
cat frames.raw | avid raw \
    --width 1920 --height 1080 --fps 30 \
    --output video.avid \
    --model "ai-generator" --version "1.0"

# Convert raw PCM audio to AAUD
cat audio.pcm | aaud raw \
    --rate 44100 --channels 1 \
    --output audio.aaud \
    --model "ai-generator" --version "1.0"

Working with AI Media Files
View and Play Files
bash

# View file (extracts and opens in default player)
aimf view my_video.avid    # Opens in VLC/mpv
aimf view my_image.aimg    # Opens in image viewer
aimf view my_audio.aaud    # Opens in audio player

# VLC can play .avid files directly!
vlc test_video_10sec.avid  # Works like any MP4 file

Extract Original Media
bash

# Extract the original media without metadata
aimf extract my_video.avid --output extracted.mp4
aimf extract my_image.aimg --output extracted.png
aimf extract my_audio.aaud --output extracted.wav

Inspect Metadata
bash

# View AI provenance information
aimf info my_video.avid

# Verify file integrity
aimf verify my_video.avid

Complete Example Workflow
bash

# 1. Generate AI content (simulated)
cargo run --example ai_generate_video_simple

# 2. Check the metadata
cargo run --bin aimf -- info test_video_10sec.avid

# Output:
# Type: Video
# Encoding: mp4
# Model: test-ai v1.0

# 3. Verify integrity
cargo run --bin aimf -- verify test_video_10sec.avid

# Output: ✅ Valid

# 4. Play directly in VLC
vlc test_video_10sec.avid

# 5. Extract the pure MP4
cargo run --bin aimf -- extract test_video_10sec.avid --output pure.mp4

# 6. Check file sizes
ls -lh test_video_10sec.avid pure.mp4
# The .avid file is slightly larger due to metadata

API Integration (Rust)
rust

use media_engine_core::{AiContainer, AiMetadata, MediaType, PayloadType};
use video_codec::embed_avid_into_mp4;

// Create metadata
let metadata = AiMetadata::new(
    "stable-diffusion".to_string(),
    "1.5".to_string(),
    None,  // No prompt hash
);

// Create container with your media
let container = AiContainer::new(
    MediaType::Video,
    "mp4".to_string(),
    PayloadType::Encoded,
    metadata,
    mp4_data,  // Your video bytes
)?;

// Embed into MP4
let avid_data = embed_avid_into_mp4(&mp4_data, &container)?;

// Save to file
std::fs::write("output.avid", avid_data)?;

Architecture
text

media-engine/
media-engine/
├── media_engine_core/      # Shared logic (serialization, crypto)
├── codecs/                 # Format-specific codecs
│   ├── image_codec/        # AIMG + PNG bridge
│   ├── audio_codec/        # AAUD + WAV bridge
│   └── video_codec/        # AVID + MP4 bridge
├── tools/                  # CLI tools
│   └── cli/               # aimf, aimg, aaud, avid
├── examples/               # Demo scripts
└── spec/                   # Format specifications

File Format Specifications
AVID (AI Video)

    Base format: MP4

    Metadata: Embedded in UUID box

    Compatibility: Plays in any MP4 player

    Use case: AI-generated video with provenance

AIMG (AI Image)

    Base format: PNG

    Metadata: Embedded in tEXt chunk

    Compatibility: Opens in any image viewer

    Use case: AI-generated images with model info

AAUD (AI Audio)

    Base format: WAV

    Metadata: Embedded in custom "AAUD" chunk

    Compatibility: Plays in any audio player

    Use case: AI-generated audio/speech

Testing
bash

# Run all tests
cargo test --workspace

# Run specific example
cargo run --example ai_generate_video_simple

# Test the CLI
cargo run --bin aimf -- --help

# Generate key pair
cargo run --bin aimf -- gen-key --output private.key

Troubleshooting
"Incomplete header" error
bash

# Ensure you're using the correct file type
file myfile.avid  # Should show MP4 data
aimf info myfile.avid  # Should show metadata

VLC not playing .avid files
bash

# .avid files are standard MP4 - check with ffprobe
ffprobe test_video_10sec.avid

# If corrupted, re-extract
aimf extract test_video_10sec.avid --output test.mp4
vlc test.mp4

GLIBCXX errors (Ubuntu snap)
bash

# Use system VLC instead of snap
sudo apt install vlc
vlc test_video_10sec.avid

# Or use ffplay
ffplay test_video_10sec.avid

Contributing

    Fork the repository

    Create your feature branch (git checkout -b feature/amazing-feature)

    Commit your changes (git commit -m 'Add amazing feature')

    Push to the branch (git push origin feature/amazing-feature)

    Open a Pull Request

License

MIT License - see LICENSE file for details
Acknowledgments

    Built with Rust

    Uses FFmpeg for encoding

    Inspired by EXIF, ID3, and other metadata standards

Related Projects

    COCO Annotations - AI training data format

    ID3 Tags - Audio metadata standard

    EXIF - Image metadata standard

text


## Additional Documentation Files

Create a `docs/` folder with these files:

### docs/API.md
```markdown
# API Reference

## Core Types

### AiContainer
The main container structure holding media and metadata.

### AiMetadata
Provenance information about AI generation.

## Codec Functions

### Video Codec
- `embed_avid_into_mp4()` - Embed AVID metadata into MP4
- `extract_avid_from_mp4()` - Extract AVID metadata from MP4

### Audio Codec  
- `embed_aaud_into_wav()` - Embed AAUD metadata into WAV
- `extract_aaud_from_wav()` - Extract AAUD metadata from WAV

### Image Codec
- `embed_aimg_into_png()` - Embed AIMG metadata into PNG
- `extract_aimg_from_png()` - Extract AIMG metadata from PNG