

# AI Media Engine — Architecture Guide

## High-Level Architecture

┌─────────────────────────────────────────────────────────────────────┐
│ CLI LAYER (tools/) │
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ │
│ │ aimf │ │ aimg │ │ aaud │ │ avid │ │
│ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ │
│ └────────────┴────────────┴────────────┘ │
│ │ │
├──────────────────────────┼──────────────────────────────────────────┤
│ COMMANDS LAYER (commands/) │
│ ┌──────────┐ ┌─────────┐ ┌────────┐ ┌──────┐ ┌───────┐ │
│ │ create │ │ raw │ │ json │ │ info │ │verify │ ... │
│ └────┬─────┘ └────┬────┘ └───┬────┘ └──┬───┘ └───┬───┘ │
│ └────────────┴──────────┴─────────┴─────────┘ │
│ │ │
├──────────────────────────┼──────────────────────────────────────────┤
│ CODECS LAYER (codecs/) │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│ │image_codec │ │audio_codec │ │video_codec │ │
│ │ (PNG) │ │ (WAV) │ │ (MP4) │ │
│ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ │
│ └────────────────┼────────────────┘ │
│ │ │
├──────────────────────────┼──────────────────────────────────────────┤
│ CORE LAYER (aimf_core/) │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│ │container │ │ metadata │ │ hash │ │signature │ │
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
│ │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│ │ error │ │ frame │ │validation│ │signature │ │
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
└─────────────────────────────────────────────────────────────────────┘
text


## Data Flow: Creating an AI Media File

User Input (JSON or raw)
│
▼
┌─────────────────────┐
│ CLI Parser │ ← Validates arguments
│ (tools//main.rs) │
└─────────┬───────────┘
│
▼
┌─────────────────────┐
│ Command Handler │ ← Routes to specific command
│ (commands/src/.rs)│
└─────────┬───────────┘
│
▼
┌─────────────────────┐
│ Codec │ ← Embeds metadata into container
│ (codecs/*/lib.rs) │
└─────────┬───────────┘
│
▼
┌─────────────────────┐
│ Core Container │ ← Serializes + hashes + signs
│ (aimf_core/src/) │
└─────────────────────┘
text


## Data Flow: Verifying a File

AIMF File (.aimg/.aaud/.avid)
│
▼
┌─────────────────────┐
│ Codec Extractor │ ← Extracts AiContainer
│ (codecs/*/lib.rs) │
└─────────┬───────────┘
│
▼
┌─────────────────────┐
│ Deserialize CBOR │ ← Parses binary metadata
└─────────┬───────────┘
│
▼
┌─────────────────────┐
│ Verify Hash │ ← SHA-256(content + metadata)
└─────────┬───────────┘
│
▼
┌─────────────────────┐
│ Verify Signature │ ← Ed25519(signature, hash)
└─────────────────────┘
text


## Module Dependencies

aimf_core (no dependencies within project)
↑
├── codecs/image_codec
├── codecs/audio_codec
└── codecs/video_codec
↑
└── commands (uses all codecs)
↑
└── tools/* (thin wrappers)
text


## Core Structures

### AiContainer Serialization

```rust
// aimf_core/src/container.rs
pub struct AiContainer {
    pub media_type: MediaType,     // 0=Image,1=Audio,2=Video
    pub encoding: String,          // "png", "wav", "mp4"
    pub payload_type: PayloadType, // 0=Encoded,1=Raw
    pub metadata: AiMetadata,
    pub payload: Vec<u8>,          // Original media
    pub hash: [u8; 32],            // SHA-256
}

// Serialized as CBOR (Concise Binary Object Representation)
// Then embedded in container format (PNG tEXt, WAV LIST, MP4 UUID)

Metadata Structure
rust

// aimf_core/src/metadata.rs
pub struct AiMetadata {
    pub is_ai_generated: bool,      // Always true
    pub model_name: String,
    pub model_version: String,
    pub prompt_hash: Option<[u8; 32]>,
    pub modality: String,            // "image"/"audio"/"video"
    pub format: String,              // "image/png", etc.
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub fps: Option<u32>,
    pub timestamp: u64,              // Unix seconds
    pub signature: Option<Vec<u8>>,  // Ed25519 sig
    pub public_key: Option<Vec<u8>>,
}

Codec Details
Image Codec (PNG)
text

Input PNG bytes
    │
    ▼
Read existing PNG chunks
    │
    ▼
Insert tEXt chunk (keyword="AIMG")
    │
    ▼
┌─────────────────────┐
│ [AIMG]\0            │
│ + base64(cbor(...)) │
└─────────────────────┘
    │
    ▼
Output new PNG bytes

Audio Codec (WAV)
text

Input WAV bytes
    │
    ▼
Parse RIFF structure
    │
    ▼
Add LIST chunk (type="AAUD")
    │
    ▼
┌─────────────────────┐
│ "AAUD" (4 bytes)    │
│ + size (4 bytes)    │
│ + cbor(...)         │
└─────────────────────┘
    │
    ▼
Output new WAV bytes

Video Codec (MP4)
text

Input MP4 bytes
    │
    ▼
Parse ISO BMFF boxes
    │
    ▼
Add uuid box to moov
    │
    ▼
┌─────────────────────────────┐
│ uuid_type = "AVID"          │
│ uuid_value = A1B2C3D4-...   │
│ data = cbor(...)            │
└─────────────────────────────┘
    │
    ▼
Output new MP4 bytes

Cryptographic Flow
Signing
text

┌─────────────────────────────────────────────────────────────┐
│ 1. Serialize AiContainer (without sig + pubkey fields)     │
│    ↓                                                        │
│ 2. SHA-256 → hash [32 bytes]                               │
│    ↓                                                        │
│ 3. Sign hash with private_key → signature [64 bytes]       │
│    ↓                                                        │
│ 4. Store signature + public_key in AiContainer             │
│    ↓                                                        │
│ 5. Reserialize complete struct → CBOR                      │
│    ↓                                                        │
│ 6. Embed in container format                               │
└─────────────────────────────────────────────────────────────┘

Verification
text

┌─────────────────────────────────────────────────────────────┐
│ 1. Extract AiContainer from file                           │
│    ↓                                                        │
│ 2. Extract signature + public_key (save temporarily)       │
│    ↓                                                        │
│ 3. Remove signature + pubkey from struct                   │
│    ↓                                                        │
│ 4. Serialize + hash                                        │
│    ↓                                                        │
│ 5. Compare hash with stored hash                           │
│    ↓                                                        │
│ 6. Verify signature with public_key                        │
└─────────────────────────────────────────────────────────────┘

Error Handling Strategy
rust

// aimf_core/src/error.rs
pub enum AimfError {
    InvalidFormat(String),      // Not a valid AIMF file
    HashMismatch,               // Content changed
    SignatureInvalid,           // Signature doesn't verify
    MissingSignature,           // No signature when expected
    IoError(std::io::Error),    // File system error
    SerializationError,         // CBOR parse error
}

Performance Characteristics
Operation	Time (typical)	Memory
Create (small image)	~10ms	~2x file size
Create (1GB video)	~0.5s + hash time	~2x + streaming
Verify (small)	~5ms	~1.5x file size
Verify (1GB)	~0.3s + hash time	Streaming capable
Extract	~50ms + copy	~1x file size
Thread Safety

All core structures are Send + Sync. Codecs use Arc<Mutex<>> where needed for FFI (FFmpeg).
Testing Strategy
text

Unit tests (aimf_core/)
    ↓
Integration tests (tests/)
    ↓
Example programs (examples/)
    ↓
CLI tests (shell scripts)

Run all: cargo test --workspace
Future Architecture Considerations
Planned: Streaming API
rust

// Future: Process large files without loading entirely
pub trait StreamingContainer {
    fn verify_stream<R: Read>(reader: R) -> Result<VerificationStatus>;
    fn extract_stream<R: Read, W: Write>(reader: R, writer: W) -> Result<()>;
}

Planned: Plugin System
rust

// Future: Support custom codecs
pub trait CodecPlugin {
    fn name(&self) -> &str;
    fn can_handle(&self, data: &[u8]) -> bool;
    fn embed(&self, data: &[u8], container: &AiContainer) -> Result<Vec<u8>>;
    fn extract(&self, data: &[u8]) -> Result<AiContainer>;
}

Planned: WASM Support
javascript

// Future: Browser-based verification
import { verifyAimg } from '@aimf/wasm';

const result = await verifyAimg(fileBuffer);
console.log(result.valid, result.metadata);

