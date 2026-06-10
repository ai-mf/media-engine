# AIMF Input Schema

> **⚠️ DEPRECATION NOTICE**  
> JSON input format was removed in v1.0. All input now uses RAW binary format.  
> This document describes the current RAW format. See `USAGE.md` for examples.

## Overview

AIMF tools accept raw binary data piped via stdin. No JSON wrapping, no text encoding — just pure bytes.

| Media Type | Format | Required Parameters |
|------------|--------|---------------------|
| Audio | 16-bit signed PCM (little-endian) | `--sample-rate`, `--channels` |
| Image | RGB24 (3 bytes per pixel, R,G,B order) | `--width`, `--height` |
| Video | RGB24 frames concatenated + optional PCM audio | `--width`, `--height`, `--fps`, `--frame-count` |

> **💡 Tip:** For quick testing with existing MP3/MP4 files, you can use `aimf create` which auto-detects input format. But for production workflows, RAW input is recommended.

---

## Audio Input (AAUD)

### RAW Format

**Specification:**
- **Encoding:** 16-bit signed integer, little-endian
- **Channels:** 1 (mono) or 2 (stereo) — interleaved
- **No header** — raw PCM only

**Required CLI parameters:**
- `--sample-rate`: Positive integer (typical: 22050, 44100, 48000)
- `--channels`: 1 or 2

**Optional CLI parameters:**
- `--model`: AI model identifier
- `--version`: Model version
- `--key`: Path to private key for signing

### Example

```bash
# Generate 1 second of 440Hz sine wave (44.1kHz, mono)
cat audio.pcm | cargo run --bin aimf -- raw \
  --output tone.aaud \
  --type audio \
  --sample-rate 44100 \
  --channels 1 \
  --model "SineWaveGen" \
  --version "1.0"
```

### Generate test audio (Rust)

```rust
let sample_rate = 44100;
let duration = 1.0;
let num_samples = (sample_rate as f64 * duration) as usize;

let mut audio_bytes = Vec::new();
for i in 0..num_samples {
    let t = i as f64 / sample_rate as f64;
    let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin();
    let sample_i16 = (sample * i16::MAX as f64) as i16;
    audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
}

// Pipe audio_bytes to aimf
```

---

## Image Input (AIMG)

### RAW Format

**Specification:**
- **Format:** RGB24 (3 bytes per pixel)
- **Order:** R, G, B sequentially (no alpha channel)
- **No header** — raw pixel data only
- **Row-major order:** Left to right, top to bottom

**Required CLI parameters:**
- `--width`: Positive integer (pixels per row)
- `--height`: Positive integer (number of rows)

**Optional CLI parameters:**
- `--model`: AI model identifier
- `--version`: Model version
- `--key`: Path to private key for signing

### Example

```bash
# Create a 800x600 gradient image
cat image.rgb | cargo run --bin aimf -- raw \
  --output gradient.aimg \
  --type image \
  --width 800 \
  --height 600 \
  --model "StableDiffusion" \
  --version "1.5"
```

### Generate test image (Rust)

```rust
let width = 800;
let height = 600;
let mut pixels = Vec::with_capacity((width * height * 3) as usize);

for y in 0..height {
    for x in 0..width {
        let r = ((x * 255) / width) as u8;
        let g = ((y * 255) / height) as u8;
        let b = (((x + y) * 255) / (width + height)) as u8;
        pixels.push(r);
        pixels.push(g);
        pixels.push(b);
    }
}

// Pipe pixels to aimf
```

---

## Video Input (AVID)

### RAW Format

**Specification:**
- **Video frames:** RGB24, same format as images
- **Frame order:** Sequential (frame 0, frame 1, ..., frame N-1)
- **Audio (optional):** 16-bit signed PCM, appended after all frames
- **No container header** — raw bytes only

**Required CLI parameters:**
- `--width`: Positive integer (pixels per row)
- `--height`: Positive integer (number of rows per frame)
- `--fps`: Positive integer (frames per second, 1-240)
- `--frame-count`: Number of frames

**Optional CLI parameters:**
- `--sample-rate`: Audio sample rate (if audio included)
- `--channels`: Audio channels (1 or 2, if audio included)
- `--model`: AI model identifier
- `--version`: Model version
- `--key`: Path to private key for signing

### Example (video only)

```bash
# Video without audio
cat frames.rgb | cargo run --bin aimf -- raw \
  --output video_only.avid \
  --type video \
  --width 640 \
  --height 480 \
  --fps 30 \
  --frame-count 300 \
  --model "GenVideo" \
  --version "1.0"
```

### Example (video + audio)

```bash
# Combine frames then audio, pipe both
cat frames.rgb audio.pcm | cargo run --bin aimf -- raw \
  --output video_with_audio.avid \
  --type video \
  --width 640 \
  --height 480 \
  --fps 30 \
  --frame-count 300 \
  --sample-rate 44100 \
  --channels 1 \
  --model "GenVideo" \
  --version "1.0"
```

### Generate test video (Rust)

```rust
let width = 320;
let height = 240;
let fps = 30;
let frame_count = 300;  // 10 seconds
let sample_rate = 44100;
let duration_secs = frame_count as f64 / fps as f64;

// Generate frames
let mut video_bytes = Vec::new();
for frame_num in 0..frame_count {
    for y in 0..height {
        for x in 0..width {
            let r = ((x + frame_num) % 256) as u8;
            let g = ((y + frame_num * 2) % 256) as u8;
            let b = ((x + y + frame_num * 3) % 256) as u8;
            video_bytes.push(r);
            video_bytes.push(g);
            video_bytes.push(b);
        }
    }
}

// Generate audio (optional)
let num_samples = (sample_rate as f64 * duration_secs) as usize;
let mut audio_bytes = Vec::new();
for i in 0..num_samples {
    let t = i as f64 / sample_rate as f64;
    let sample = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.5;
    let sample_i16 = (sample * i16::MAX as f64) as i16;
    audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
}

// Combine
let mut combined = video_bytes;
combined.extend(audio_bytes);

// Pipe combined to aimf
```

---

## Convenience: Converting Existing Media

For quick testing with MP3 or MP4 files, you can use `aimf create`:

```bash
# Convert MP3 to AAUD
cat input.mp3 | cargo run --bin aimf -- create --output output.aaud \
  --model "ModelName" --version "1.0"

# Convert MP4 to AVID
cat input.mp4 | cargo run --bin aimf -- create --output output.avid \
  --model "ModelName" --version "1.0"
```

> **Note:** For production workflows, generating RAW input directly is recommended over converting existing media.

---

## Field Reference (CLI Parameters)

| Parameter | Type | Required for | Description |
|-----------|------|--------------|-------------|
| `--type` | string | **Yes** (aimf only) | `audio`, `image`, or `video` |
| `--output` | path | **Yes** | Output file path (.aaud, .aimg, .avid) |
| `--model` | string | No | AI model identifier |
| `--version` | string | No | Model version string |
| `--key` | path | No | Private key for signing |
| `--width` | integer | Image, Video | Width in pixels |
| `--height` | integer | Image, Video | Height in pixels |
| `--fps` | integer | Video | Frames per second |
| `--frame-count` | integer | Video | Total number of frames |
| `--sample-rate` | integer | Audio, Video (with audio) | Sample rate in Hz |
| `--channels` | integer | Audio, Video (with audio) | 1 (mono) or 2 (stereo) |
| `--format` | string | Image (optional) | Always `rgb8` (default) |

---

## Size Calculations

### Audio
```
bytes = sample_rate * channels * duration_seconds * 2
```
Example: 44.1kHz, mono, 10 seconds = 44100 * 1 * 10 * 2 = 882,000 bytes

### Image
```
bytes = width * height * 3
```
Example: 1920×1080 = 1920 * 1080 * 3 = 6,220,800 bytes (~6.2MB)

### Video (no audio)
```
bytes = width * height * 3 * frame_count
```
Example: 1280×720, 300 frames (10s @30fps) = 1280 * 720 * 3 * 300 = 829,440,000 bytes (~791MB)

### Video (with audio)
```
bytes = (width * height * 3 * frame_count) + (sample_rate * channels * duration_seconds * 2)
```

---

## Common Mistakes

### ❌ Wrong endianness
Audio must be **little-endian**. Most systems are little-endian, but if you're on a big-endian system, convert first.

### ❌ Missing audio parameters for video
If you append audio bytes, you **must** provide `--sample-rate` and `--channels`. Without them, the audio bytes will be interpreted as video frames.

### ❌ Wrong pixel format
Only RGB24 is supported (not RGBA, not BGR, not YUV).

### ❌ Frame count mismatch
`--frame-count` must match the actual number of frames in the input. Extra or missing bytes will cause errors.

---

## See Also

- [USAGE.md](../USAGE.md) — Complete CLI guide with examples
- [WORKFLOWS.md](WORKFLOWS.md) — Real-world usage patterns
- [API.md](API.md) — Rust API reference
```