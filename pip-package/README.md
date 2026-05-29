# AIMF - AI Media Format for Python

Universal Python wrapper for AIMF - the ffmpeg for AI-generated content.

## Installation

```bash
pip install aimf

That's it! The AIMF binary is automatically downloaded for your platform.

All Commands Available
Universal AIMF (auto-detects format)

Usage
python

from aimf import AIMF, AudioAI, ImageAI, VideoAI, MediaType


# Get info
info = AIMF.info("file.aaud")

# Verify
result = AIMF.verify("file.aimg")

# Extract
AIMF.extract("file.avid", "output.mp4")

# View
AIMF.view("file.aaud")

# Sign existing file
AIMF.sign("unsigned.aaud", "private.key", "signed.aaud")

# Batch process
AIMF.batch("*.aaud", "./output", model="MyModel")

# Generate key
AIMF.generate_key("mykey.key")

Media-Specific Helpers
python

from aimf import AudioAI, ImageAI, VideoAI

# Audio
audio = AudioAI.from_samples([0.1, -0.2, 0.3], sample_rate=44100)
audio.with_model("ElevenLabs", "v2").save("speech.aaud")

# Image
image = ImageAI.from_pixels([255,0,0, 0,255,0], width=2, height=2)
image.with_model("Midjourney", "v6").save("art.aimg")

# Video
video = VideoAI.from_frames(frames, width=1920, height=1080, fps=30)
video.with_audio(samples, sample_rate=44100)
video.with_model("Runway", "gen2").save("video.avid")

# Verify
from aimf import AIMF
result = AIMF.verify("file.aaud")
print(f"Valid: {result['valid']}")

Command Line

After pip install, you can also use the CLI:
bash

aimf info file.aaud
aimf verify file.aimg
aimf view file.avid

Requirements

    Python 3.7+

    Internet connection (for first-time binary download)

License

Apache 2.0
text