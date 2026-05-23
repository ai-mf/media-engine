
## File 3: `/media-engine/docs/API.md`

```markdown
# API Reference

## Core Types

### AiContainer

The main container structure holding media and metadata.

```rust
pub struct AiContainer {
    pub media_type: MediaType,
    pub encoding: String,
    pub payload_type: PayloadType,
    pub metadata: AiMetadata,
    pub payload: Vec<u8>,
    pub hash: [u8; 32],
}

AiMetadata

Provenance information about AI generation.
rust

pub struct AiMetadata {
    pub is_ai_generated: bool,
    pub model_name: String,
    pub model_version: String,
    pub prompt_hash: Option<[u8; 32]>,
    pub modality: String,
    pub format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub fps: Option<u32>,
    pub timestamp: u64,
}

Codec Functions
Video Codec (AVID)

    embed_avid_into_mp4(mp4_data: &[u8], container: &AiContainer) -> Result<Vec<u8>>
    Embeds AVID metadata into an MP4 file.

    extract_avid_from_mp4(mp4_data: &[u8]) -> Result<AiContainer>
    Extracts AVID metadata from an MP4 file.

Audio Codec (AAUD)

    embed_aaud_into_wav(wav_data: &[u8], container: &AiContainer) -> Result<Vec<u8>>
    Embeds AAUD metadata into a WAV file.

    extract_aaud_from_wav(wav_data: &[u8]) -> Result<AiContainer>
    Extracts AAUD metadata from a WAV file.

Image Codec (AIMG)

    embed_aimg_into_png(png_data: &[u8], container: &AiContainer) -> Result<Vec<u8>>
    Embeds AIMG metadata into a PNG file.

    extract_aimg_from_png(png_data: &[u8]) -> Result<AiContainer>
    Extracts AIMG metadata from a PNG file.

CLI Commands
aimf (Universal Tool)
bash

# Ingest JSON data
aimf ingest --output file.avid --model MODEL --version VERSION

# View file (extract and open)
aimf view file.avid

# Extract media without metadata
aimf extract file.avid --output output.mp4

# Show metadata
aimf info file.avid

# Verify integrity
aimf verify file.avid

Format-Specific Tools
bash

# Image
aimg create input.png --output output.aimg --model MODEL --version VERSION

# Audio
aaud create input.mp3 --output output.aaud --model MODEL --version VERSION

# Video
avid create input.mp4 --output output.avid --model MODEL --version VERSION