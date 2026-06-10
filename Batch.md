////media-engine/BATCH.md
# Batch Processing with AIMF Suite

The `batch` command allows you to process multiple files at once, perfect for bulk operations on AI-generated media.

## Batch Command Syntax

```bash
cargo run --bin aaud -- batch [OPTIONS]
cargo run --bin aimg -- batch [OPTIONS]
cargo run --bin avid -- batch [OPTIONS]
cargo run --bin aimf -- batch [OPTIONS]
```

## Batch Options

| Option | Description |
|--------|-------------|
| `--input <PATTERN>` | Input file pattern (e.g., `*.aaud`, `*.raw`) |
| `--output-dir <DIR>` | Output directory for processed files |
| `--recursive` | Search subdirectories recursively |
| `--parallel` | Process files in parallel |
| `--jobs <N>` | Number of parallel jobs (default: CPU cores) |
| `--verify` | Verify files instead of creating |
| `--extract` | Extract original media from containers |
| `--info` | Show info for each file |
| `--sign` | Sign files with key |
| `--key <FILE>` | Private key for signing |
| `--model <NAME>` | Model name (for creation) |
| `--version <VER>` | Model version (for creation) |
| `--force` | Overwrite existing files |
| `--dry-run` | Show what would be processed |

## Examples

### 1. Batch Verify All Files

```bash
# Verify all AAUD files in current directory
cargo run --bin aaud -- batch --input "*.aaud" --verify

# Verify recursively with summary
cargo run --bin aaud -- batch --input "*.aaud" --verify --recursive

# Verify all AIMF files (any format)
cargo run --bin aimf -- batch --input "*.aaud" --verify
cargo run --bin aimf -- batch --input "*.aimg" --verify
cargo run --bin aimf -- batch --input "*.avid" --verify
```

### 2. Batch Extract Original Media

```bash
# Extract all audio files to WAV
cargo run --bin aaud -- batch --input "*.aaud" --extract --output-dir ./extracted

# Extract all images to PNG
cargo run --bin aimg -- batch --input "*.aimg" --extract --output-dir ./originals

# Extract all videos to MP4
cargo run --bin avid -- batch --input "*.avid" --extract --output-dir ./videos
```

### 3. Batch Sign Unsigned Files

```bash
# Sign all unsigned AAUD files
cargo run --bin aaud -- batch --input "*.aaud" --sign --key private.key --output-dir ./signed

# Sign all images recursively
cargo run --bin aimg -- batch --input "*.aimg" --sign --key mykey.key --recursive
```

### 5. Batch Create from Raw Files

```bash
# Batch create AAUD from raw PCM files
cargo run --bin aaud -- batch \
  --input "*.raw" \
  --output-dir ./aaud_files \
  --model "AudioLDM" \
  --version "1.5"

# Batch create AIMG from raw RGB files (must specify dimensions)
for file in *.rgb; do
  cargo run --bin aimg -- batch \
    --input "$file" \
    --output-dir ./aimg_files \
    --model "DALL-E" \
    --version "2.0"
done
```

### 6. Batch Info Display

```bash
# Show info for all media files
cargo run --bin aimf -- batch --input "*.aaud" --info
cargo run --bin aimf -- batch --input "*.aimg" --info
cargo run --bin aimf -- batch --input "*.avid" --info

# Show detailed info with progress
cargo run --bin aaud -- batch --input "*.aaud" --info --verbose
```

### 7. Complex Batch Operations

```bash
# Verify, then sign valid files
cargo run --bin aaud -- batch --input "*.aaud" --verify
cargo run --bin aaud -- batch --input "*.aaud" --sign --key private.key --output-dir ./signed

# Extract and then verify extraction
cargo run --bin aimg -- batch --input "*.aimg" --extract --output-dir ./extracted
cargo run --bin aimg -- batch --input "./extracted/*.png" --verify
```

### 8. Dry Run First

```bash
# See what would be processed without doing anything
cargo run --bin aaud -- batch --input "*.aaud" --verify --dry-run

# Check batch create operation
cargo run --bin aimg -- batch --input "*.raw" --output-dir ./out --dry-run
```

## Batch Script Examples

### Process All AI Media in Directory

```bash
#!/bin/bash
# process_all.sh - Verify and extract all AI media

echo "🔍 Processing all AIMF files..."

# Verify all files
cargo run --bin aimf -- batch --input "*.aaud" --verify --parallel
cargo run --bin aimf -- batch --input "*.aimg" --verify --parallel
cargo run --bin aimf -- batch --input "*.avid" --verify --parallel

# Extract originals
cargo run --bin aaud -- batch --input "*.aaud" --extract --output-dir ./wav_files
cargo run --bin aimg -- batch --input "*.aimg" --extract --output-dir ./png_files
cargo run --bin avid -- batch --input "*.avid" --extract --output-dir ./mp4_files

echo "✅ Done!"
```

### Sign All Unsigned Files

```bash
#!/bin/bash
# sign_all.sh - Sign all unsigned AAUD files

KEY="private.key"
OUTPUT_DIR="./signed"

mkdir -p "$OUTPUT_DIR"

echo "🔐 Signing all AAUD files..."

cargo run --bin aaud -- batch \
  --input "*.aaud" \
  --sign \
  --key "$KEY" \
  --output-dir "$OUTPUT_DIR" \
  --parallel \
  --force

echo "✅ Signed files saved to $OUTPUT_DIR"
```

## Performance Tips

### Parallel Processing
```bash
# Use all CPU cores
cargo run --bin aaud -- batch --input "*.aaud" --verify --parallel

# Use specific number of jobs
cargo run --bin aimg -- batch --input "*.aimg" --extract --parallel --jobs 4
```

### Large Directories
```bash
# Process recursively with progress
cargo run --bin aimf -- batch \
  --input "*.aaud" \
  --verify \
  --recursive \
  --parallel \
  --verbose
```

### Memory Management
```bash
# Process in chunks (using find + xargs)
find . -name "*.aaud" -print0 | xargs -0 -P 4 -I {} \
  cargo run --bin aaud -- batch --input "{}" --verify
```

## Output Examples

### Verify Batch Output
```
🔍 Batch Verifying 5 files...
[1/5] ✅ file1.aaud - VALID
[2/5] ✅ file2.aaud - VALID
[3/5] ❌ file3.aaud - CORRUPT (hash mismatch)
[4/5] ✅ file4.aaud - VALID
[5/5] ⚠️ file5.aaud - VALID but unsigned

Summary: 3 valid, 1 corrupted, 1 unsigned
```

## Error Handling

```bash
# Continue on errors
cargo run --bin aaud -- batch --input "*.aaud" --verify --force

# Verbose error output
cargo run --bin aimg -- batch --input "*.aimg" --extract --verbose 2>&1 | tee batch.log
```

The batch command makes it easy to process entire collections of AI-generated media files efficiently!