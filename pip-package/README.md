# AIMF - AI Media Format for Python

Universal Python wrapper for AIMF - verifiable AI-generated audio, images, and video.

## Installation

```bash
pip install aimf
```

That's it! The AIMF binary is automatically downloaded for your platform on first use.

## Quick Start

### Audio

```python
from aimf import AudioAI
import math

# Generate a simple sine wave
samples = []
for i in range(int(44100 * 0.5)):  # 0.5 seconds
    t = i / 44100
    samples.append(math.sin(2 * math.pi * 440 * t) * 0.5)

# Create and save
audio = AudioAI.from_samples(samples, sample_rate=44100, channels=1)
audio.with_model("MusicGen", "1.0").save("melody.aaud")

# Verify
from aimf import AIMF
result = AIMF.verify("melody.aaud")
print(f"Valid: {result['valid']}")
```

### Image

```python
from aimf import ImageAI

# Create a simple gradient (100x100 RGB)
pixels = []
for y in range(100):
    for x in range(100):
        pixels.append((x * 85) % 256)      # R
        pixels.append((y * 85) % 256)      # G
        pixels.append(255 if (x+y) % 2 else 0)  # B

image = ImageAI.from_pixels(pixels, width=100, height=100)
image.with_model("StableDiffusion", "1.5").save("art.aimg")
```

### Video

```python
from aimf import VideoAI

# Generate 30 frames (64x48 RGB)
frames = []
for frame_num in range(30):
    frame = []
    for y in range(48):
        for x in range(64):
            frame.append((x + frame_num) % 256)
            frame.append((y + frame_num * 2) % 256)
            frame.append((x + y + frame_num * 3) % 256)
    frames.append(frame)

video = VideoAI.from_frames(frames, width=64, height=48, fps=30)
video.with_model("GenVideo", "1.0").save("animation.avid")
```

## API Reference

### Universal AIMF

```python
from aimf import AIMF

# Get file information
info = AIMF.info("file.aaud")  # Returns dict with raw_output

# Verify integrity and signature
result = AIMF.verify("file.aimg")  # Returns {'valid': bool, 'output': str}

# Extract original media
AIMF.extract("file.avid", "output.mp4")

# View with system player
AIMF.view("file.aaud")

# Sign existing file
AIMF.sign("unsigned.aaud", "private.key", "signed.aaud")

# Generate signing key
AIMF.generate_key("mykey.key")
```

### AudioAI

```python
from aimf import AudioAI

# Create from f32 samples
audio = AudioAI.from_samples(samples, sample_rate=44100, channels=1)

# Create from raw PCM16 bytes
audio = AudioAI.from_raw(audio_bytes, sample_rate=44100, channels=1)

# Load existing file
audio = AudioAI.from_file("existing.aaud")

# Set metadata
audio.with_model("ModelName", "1.0")
audio.with_version("2.0")
audio.with_key("private.key")

# Save
audio.save("output.aaud")
```

### ImageAI

```python
from aimf import ImageAI

# Create from RGB pixel list
image = ImageAI.from_pixels(pixels, width=1920, height=1080)

# Create from raw RGB bytes
image = ImageAI.from_raw(image_bytes, width=1920, height=1080)

# Load existing file
image = ImageAI.from_file("existing.aimg")

# Set metadata
image.with_model("StableDiffusion", "1.5")
image.with_key("private.key")

# Save
image.save("output.aimg")
```

### VideoAI

```python
from aimf import VideoAI

# Create from frames
video = VideoAI.from_frames(frames, width=1920, height=1080, fps=30)

# Create from raw bytes
video = VideoAI.from_raw(video_bytes, width=1920, height=1080, fps=30)

# Load existing file
video = VideoAI.from_file("existing.avid")

# Add audio track
video.with_audio(samples, sample_rate=44100)

# Set metadata
video.with_model("Sora", "1.0")
video.with_key("private.key")

# Save
video.save("output.avid")
```

## Raw Data Helpers

For processing raw media data programmatically:

```python
from aimf import AudioAI

# Create audio from raw PCM16 bytes
with open("audio.pcm", "rb") as f:
    audio_bytes = f.read()

audio = AudioAI.from_raw(
    audio_bytes, 
    sample_rate=44100, 
    channels=1
)
audio.with_model("Model", "1.0").save("output.aaud")
```

### Raw Format Specifications

| Media Type | Input Format | Required Parameters |
|------------|--------------|---------------------|
| Audio | 16-bit signed PCM (little-endian) | `sample_rate`, `channels` |
| Image | RGB24 (3 bytes per pixel) | `width`, `height` |
| Video | RGB24 frames concatenated + optional PCM16 audio | `width`, `height`, `fps`, `frame_count` |

## Requirements

- Python 3.7+
- Internet connection (for first-time binary download)

## License

Apache 2.0

## See Also

- [GitHub Repository](https://github.com/ai-mf/media-engine)
- [AIMF Specification](spec/ai-media-format.md)
- [Examples](python-examples/)
```