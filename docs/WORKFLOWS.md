# Common Workflows

## AI Training Pipeline
1. Generate images with model
2. Wrap as AIMG with metadata
3. Verify integrity before training
4. Extract for training batches

## Content Distribution
1. Create AVID video
2. Distribute to users
3. Users verify authenticity
4. Play in standard players

## Audit Trail
1. Each file contains generation timestamp
2. Model version tracked
3. Optional prompt hash for reproducibility
4. Cryptographic verification

GitHub Release Checklist
markdown

## Release v0.1.0

### Pre-release
- [ ] All tests pass
- [ ] Examples work on clean install
- [ ] Documentation complete
- [ ] CHANGELOG.md updated

### Build artifacts
- [ ] Linux binary (x86_64)
- [ ] Linux binary (ARM64)
- [ ] macOS binary (Intel)
- [ ] macOS binary (Apple Silicon)
- [ ] Windows binary (x86_64)

### Release notes
- Initial release
- Support for AIMG, AAUD, AVID formats
- CLI tools for conversion and verification
- Backward compatible with standard players


## Quick Demo Workflow

```bash
# 1. Generate test files
cargo run --example ai_generate_video_simple
cargo run --example ai_generate_image
cargo run --example ai_generate_audio

# 2. View them directly
vlc test_video_10sec.avid
eog test_image.aimg
vlc test_audio.aaud

# 3. Inspect metadata
cargo run --bin aimf -- info test_video_10sec.avid

# 4. Verify integrity
cargo run --bin aimf -- verify test_video_10sec.avid

AI Training Pipeline
bash

# Generate images with your model
your_model --prompt "cat" --output cat.png

# Wrap with metadata
aimg create cat.png --output cat.aimg --model "your-model" --version "1.0"

# Verify before training
aimf verify cat.aimg

# Extract for training batch
aimf extract cat.aimg --output training/cat_001.png

Content Distribution
bash

# Create AI-generated video
cargo run --example ai_generate_video_simple

# Distribute .avid file to users
scp test_video_10sec.avid user@example.com:~/videos/

# Users can:
# - Play directly in VLC
vlc test_video_10sec.avid

# - Verify authenticity
aimf verify test_video_10sec.avid

# - Extract pure MP4
aimf extract test_video_10sec.avid --output video.mp4

Batch Processing
bash

# Convert all PNGs in directory to AIMG
for file in *.png; do
    aimg create "$file" --output "${file%.png}.aimg" \
        --model "batch-model" --version "1.0"
done

# Extract all AIMG files to PNG
for file in *.aimg; do
    aimf extract "$file" --output "${file%.aimg}.png"
done

Integrity Checking Pipeline
bash

# Generate with hash
cargo run --example ai_generate_video_simple

# Store hash in database
HASH=$(cargo run --bin aimf -- info test_video_10sec.avid | grep Hash | cut -d' ' -f2)
echo "$HASH test_video_10sec.avid" >> hashes.db

# Later, verify
cargo run --bin aimf -- verify test_video_10sec.avid
# Should output: ✅ Valid

Audit Trail
bash

# Check generation timestamp
cargo run --bin aimf -- info test_video_10sec.avid | grep Timestamp

# Track model versions
cargo run --bin aimf -- info test_video_10sec.avid | grep Model

# Export all metadata to JSON
cargo run --bin aimf -- info test_video_10sec.avid --json > metadata.json

text


## GitHub Repository Structure

When you push to GitHub, your repository should look like:

media-engine/
├── README.md (project overview - optional)
├── README.md (main documentation)
├── spec/
│ └── README.md (format specification)
├── docs/
│ ├── API.md
│ └── WORKFLOWS.md
├── media_engine_core/
├── codecs/
├── tools/
└── examples/
text


## Quick GitHub Setup Commands

```bash
# Initialize git repository
cd /home/ubuntu/Programs/media-engine/rust/media-engine
git init

# Add all documentation
git add media-engine/README.md
git add media-engine/spec/README.md
git add media-engine/docs/API.md
git add media-engine/docs/WORKFLOWS.md

# Add your code
git add media-engine/media_engine_core/
git add media-engine/codecs/
git add media-engine/tools/
git add media-engine/examples/

# Commit
git commit -m "Initial commit: AI Media Engine with AIMG, AAUD, AVID formats"

# Add remote (replace with your GitHub URL)
git remote add origin https://github.com/YOUR_USERNAME/media-engine.git

# Push
git push -u origin main