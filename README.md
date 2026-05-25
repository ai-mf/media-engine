# AI Media Engine

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A verifiable AI media format system that embeds provenance metadata into existing media files while remaining backward compatible with standard media players (VLC, ffplay, etc.).

## Overview

AI Media Engine provides a family of container formats (AIMG, AAUD, AVID) that wrap standard media files with AI provenance metadata. The formats are designed to be:

- **Backward compatible**: `.avid` files play directly in VLC as normal MP4 files
- **Verifiable**: Each file contains cryptographic hashes for integrity checking
- **Cryptographically signed**: Optional Ed25519 signatures for authenticity verification
- **Extensible**: JSON metadata allows arbitrary AI model information
- **Streamable**: Header-first design enables quick metadata access

### Format Family

| Format | Extension | Media Type | Base Format | Plays in VLC |
|--------|-----------|------------|-------------|--------------|
| AIMG | `.aimg` | Image | PNG | ✅ Yes (as PNG) |
| AAUD | `.aaud` | Audio | WAV | ✅ Yes (as WAV) |
| AVID | `.avid` | Video | MP4 | ✅ Yes (as MP4) |

## Cryptographic Signatures

### What are keys used for?

The AI Media Engine uses **Ed25519 cryptographic signatures** to provide:

1. **Authenticity**: Prove the file was created by a specific AI system or organization
2. **Integrity**: Detect any tampering with the file after creation
3. **Non-repudiation**: The creator cannot deny having generated the file
4. **Trust chain**: Establish trust in AI-generated content provenance

### How it works

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│ AI Model    │────▶│ Create Hash  │────▶│ Sign with       │
│ Generates   │     │ of Content   │     │ Private Key     │
│ Media       │     │              │     │                 │
└─────────────┘     └──────────────┘     └────────┬────────┘
                                                    │
                                                    ▼
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│ Verify      │◀────│ Extract      │◀────│ Embed Signature │
│ with Public │     │ Signature    │     │ + Public Key    │
│ Key         │     │ & Hash       │     │ into File       │
└─────────────┘     └──────────────┘     └─────────────────┘
```

### Key Generation

Generate a key pair (private + public key):

```bash
# Generate a new key pair
cargo run --bin aimf -- gen-key --output private.key

# Output:
# ✅ Generated key pair
#    Private key saved to: private.key
#    Public key: a1b2c3d4e5f6...
```

**What gets generated:**
- `private.key` - Your secret signing key (keep this SAFE and SECURE!)
- Public key - Displayed in terminal, share this so others can verify your files

**⚠️ Security Warning:** 
- Never commit `private.key` to version control
- Never share your private key with anyone
- Back up your private key in a secure location
- The private key is 32 bytes of random data - treat it like a password

### Signing AI Media Files

Sign your AI-generated content to prove its origin:

```bash
# Sign an image
aimg create input.png --output signed.aimg \
    --model "stable-diffusion" --version "1.5" \
    --key private.key

# Sign audio
aaud create input.mp3 --output signed.aaud \
    --model "audio-gan" --version "2.0" \
    --key private.key

# Sign video
avid create input.mp4 --output signed.avid \
    --model "video-diffusion" --version "1.0" \
    --key private.key
```

### Verifying Signatures

Verify that a file is authentic and untampered:

```bash
# Verify an AI media file
aimf verify signed.aimg

# Output:
# 🔍 Verification Results:
#    Hash valid: true
#    ✅ Signature valid (cryptographically verified)
# 
# ✅ File is VALID and VERIFIED
```

If verification fails:

```bash
aimf verify tampered.aimg

# Output:
# 🔍 Verification Results:
#    Hash valid: false
#    ❌ Signature INVALID - File may be tampered!
# 
# ❌ File is CORRUPT or TAMPERED
```

### Checking Signature Status

View signature information without full verification:

```bash
aimf info signed.avid

# Output includes:
# Signature: Present ✓
#   Public Key: a1b2c3d4e5f6...
```

### Key Management Best Practices

#### For Individual Creators:
```bash
# Generate one key for all your AI creations
cargo run --bin aimf -- gen-key --output my_ai_identity.key

# Keep it safe
chmod 600 my_ai_identity.key
mkdir -p ~/.secure/keys/
mv my_ai_identity.key ~/.secure/keys/

# Share your public key with others
cargo run --bin aimf -- gen-key --output temp.key  # Just to get public key
# Then share the public key hex string
```

#### For Organizations:
```bash
# Generate separate keys for different models/departments
cargo run --bin aimf -- gen-key --output production_vision.key
cargo run --bin aimf -- gen-key --output production_audio.key
cargo run --bin aimf -- gen-key --output staging.key

# Store in hardware security module (HSM) or secure key management
# Use environment variables or secret managers in production
```

#### For Verification Systems:
```bash
# Store trusted public keys in a configuration
TRUSTED_KEYS="a1b2c3...,d4e5f6..."

# Batch verify multiple files
for file in *.aimg; do
    echo "Verifying $file..."
    aimf verify "$file" || echo "FAILED: $file"
done
```

### Cryptographic Details

- **Algorithm**: Ed25519 (elliptic curve signature scheme)
- **Key size**: 32 bytes (256 bits)
- **Signature size**: 64 bytes (512 bits)
- **Security level**: 128-bit security
- **Performance**: Extremely fast verification (~100,000 verifications/second)

### Use Cases

| Scenario | Without Signing | With Signing |
|----------|----------------|--------------|
| **News organization** publishes AI-generated image | Anyone can claim it's fake | Can verify it came from trusted source |
| **Research lab** releases AI model outputs | No way to prove authenticity | Signed files prove origin |
| **Legal evidence** from AI system | Tampering undetectable | Any modification invalidates signature |
| **Content marketplace** for AI art | Buyers can't verify creator | Signatures prove artist identity |

### Integration with Existing Systems

```rust
use media_engine_core::{AiContainer, AiMetadata, CryptoSignature};
use ed25519_dalek::SigningKey;

// Load your private key
let key_bytes = std::fs::read("private.key")?;
let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());

// Create and sign in one step
let mut container = AiContainer::new(media_type, encoding, payload_type, metadata, payload)?;
container.sign(&signing_key)?;

// Verify signature
let result = container.full_verify();
if result.hash_valid && result.signature_valid == Some(true) {
    println!("✅ Authentic and untampered!");
}
```

### FAQ

**Q: Can I use the same key for all file types?**  
A: Yes! Keys work across AIMG, AAUD, and AVID formats.

**Q: What happens if I lose my private key?**  
A: You can no longer sign new files as that identity. Already signed files remain verifiable.

**Q: Can someone forge my signature?**  
A: No, Ed25519 is cryptographically secure. Only someone with your private key can create valid signatures.

**Q: Does signing make files much larger?**  
A: Minimal overhead - only ~96 bytes for signature + public key.

**Q: Can I sign an already-signed file?**  
A: Yes, but the new signature replaces the old one. Use `aimf sign` command.

**Q: Are signatures required?**  
A: No, they're optional. Files can be created without signatures for testing or when provenance isn't needed.

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
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/media-engine.git
cd media-engine/ai

# Build all components in release mode
cargo build --release

# Generate your first key pair
cargo run --bin aimf -- gen-key --output private.key

# Build individual tools
cargo build --bin aimf   # Main tool (recommended)
cargo build --bin aimg   # Image-specific tool
cargo build --bin aaud   # Audio-specific tool
cargo build --bin avid   # Video-specific tool
```

### Install CLI Tools

```bash
# Install all tools to ~/.cargo/bin
cargo install --path tools/cli

# Now you can run from anywhere
aimf --help
aimg --help
aaud --help
avid --help
```

## Usage Examples

### Method 1: Using Examples (Quick Demo)

Generate AI media files with simulated content:

```bash
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
```

### Method 2: Creating AI Media from JSON

Feed JSON data to aimf ingest to create AI media files:

```bash
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
```

### Method 3: Converting Existing Media to AI Format

```bash
# Convert existing PNG to AIMG (embeds metadata)
aimg create input.png --output output.aimg --model "my-model" --version "1.0"

# Sign your creation
aimg create input.png --output signed.aimg \
    --model "my-model" --version "1.0" \
    --key private.key

# Convert existing MP3 to AAUD  
aaud create input.mp3 --output output.aaud --model "my-model" --version "1.0"

# Convert existing MP4 to AVID
avid create input.mp4 --output output.avid --model "my-model" --version "1.0"
```

### Method 4: Raw Data Conversion (from stdin)

```bash
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
```

### Method 5: Sign Existing AIMF Files

```bash
# Sign an unsigned AI media file
aimf sign --input unsigned.aimg --key private.key --output signed.aimg

# This preserves the original format (PNG, WAV, or MP4 container)
```

## Working with AI Media Files

### View and Play Files

```bash
# View file (extracts and opens in default player)
aimf view my_video.avid    # Opens in VLC/mpv
aimf view my_image.aimg    # Opens in image viewer
aimf view my_audio.aaud    # Opens in audio player

# VLC can play .avid files directly!
vlc test_video_10sec.avid  # Works like any MP4 file
```

### Extract Original Media

```bash
# Extract the original media without metadata
aimf extract my_video.avid --output extracted.mp4
aimf extract my_image.aimg --output extracted.png
aimf extract my_audio.aaud --output extracted.wav
```

### Inspect Metadata and Verify

```bash
# View AI provenance information
aimf info my_video.avid

# Verify file integrity and signature
aimf verify my_video.avid
```

## Complete Example Workflow with Signing

```bash
# 1. Generate key pair for your AI system
cargo run --bin aimf -- gen-key --output my_ai.key
PUBLIC_KEY=$(cargo run --bin aimf -- gen-key --output /dev/null 2>&1 | grep "Public key:" | awk '{print $3}')
echo "My public key: $PUBLIC_KEY"

# 2. Generate AI content (simulated)
cargo run --example ai_generate_video_simple

# 3. Sign the AI-generated video
cargo run --bin aimf -- sign \
    --input test_video_10sec.avid \
    --key my_ai.key \
    --output verified_video.avid

# 4. Check the metadata and signature
cargo run --bin aimf -- info verified_video.avid
# Output:
# Type: Video
# Encoding: mp4
# Model: test-ai v1.0
# Signature: Present ✓
#   Public Key: $PUBLIC_KEY

# 5. Verify integrity and authenticity
cargo run --bin aimf -- verify verified_video.avid
# Output: ✅ File is VALID and VERIFIED

# 6. Play directly in VLC
vlc verified_video.avid

# 7. Extract the pure MP4 (signature stays in metadata)
cargo run --bin aimf -- extract verified_video.avid --output pure.mp4

# 8. Check that tampering is detected
cp verified_video.avid tampered.avid
# Modify the file (e.g., with a hex editor)
cargo run --bin aimf -- verify tampered.avid
# Output: ❌ File is CORRUPT or TAMPERED
```

## API Integration (Rust)

```rust
use media_engine_core::{AiContainer, AiMetadata, MediaType, PayloadType, CryptoSignature};
use video_codec::embed_avid_into_mp4;
use ed25519_dalek::SigningKey;

// Load your signing key
let key_bytes = std::fs::read("private.key")?;
let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());

// Create metadata
let metadata = AiMetadata::new(
    "stable-diffusion".to_string(),
    "1.5".to_string(),
    None,  // No prompt hash
);

// Create container with your media
let mut container = AiContainer::new(
    MediaType::Video,
    "mp4".to_string(),
    PayloadType::Encoded,
    metadata,
    mp4_data,  // Your video bytes
)?;

// Sign the container
container.sign(&signing_key)?;

// Embed into MP4
let avid_data = embed_avid_into_mp4(&mp4_data, &container)?;

// Save to file
std::fs::write("output.avid", avid_data)?;

// Later: Verify the file
let result = container.full_verify();
if result.hash_valid && result.signature_valid == Some(true) {
    println!("File is authentic and untampered!");
}
```

## Architecture

```
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
```

## File Format Specifications

### AVID (AI Video)
- **Base format**: MP4
- **Metadata**: Embedded in UUID box
- **Compatibility**: Plays in any MP4 player
- **Use case**: AI-generated video with provenance

### AIMG (AI Image)
- **Base format**: PNG
- **Metadata**: Embedded in tEXt chunk
- **Compatibility**: Opens in any image viewer
- **Use case**: AI-generated images with model info

### AAUD (AI Audio)
- **Base format**: WAV
- **Metadata**: Embedded in custom "AAUD" chunk
- **Compatibility**: Plays in any audio player
- **Use case**: AI-generated audio/speech

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific example
cargo run --example ai_generate_video_simple

# Test the CLI
cargo run --bin aimf -- --help

# Generate key pair for testing
cargo run --bin aimf -- gen-key --output test.key
```

## Troubleshooting

### "Incomplete header" error
```bash
# Ensure you're using the correct file type
file myfile.avid  # Should show MP4 data
aimf info myfile.avid  # Should show metadata
```

### VLC not playing .avid files
```bash
# .avid files are standard MP4 - check with ffprobe
ffprobe test_video_10sec.avid

# If corrupted, re-extract
aimf extract test_video_10sec.avid --output test.mp4
vlc test.mp4
```

### GLIBCXX errors (Ubuntu snap)
```bash
# Use system VLC instead of snap
sudo apt install vlc
vlc test_video_10sec.avid

# Or use ffplay
ffplay test_video_10sec.avid
```

### Key permission errors
```bash
# Ensure private key has correct permissions
chmod 600 private.key

# Check key format
file private.key  # Should show data
```

## Security Considerations

- **Private key security**: Never expose your private key. Use environment variables or secret management in production.
- **Key rotation**: Generate new keys periodically and retire old ones.
- **Public key distribution**: Share public keys through secure channels (HTTPS, key servers, etc.).
- **Revocation**: Have a mechanism to revoke compromised keys.
- **Backup**: Always backup private keys in secure, encrypted storage.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

APACHE 2.0 License - see LICENSE file for details

## Acknowledgments

- Built with Rust
- Uses FFmpeg for encoding
- Ed25519 signatures via ed25519-dalek
- Inspired by EXIF, ID3, and other metadata standards

## Related Projects

- COCO Annotations - AI training data format
- ID3 Tags - Audio metadata standard
- EXIF - Image metadata standard
- C2PA - Content authenticity initiative
```