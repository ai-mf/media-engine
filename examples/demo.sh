
```bash
#!/bin/bash
# ai/examples/demo.sh

echo "=== AI Media Engine MVP Demo ==="

# Create test files if they don't exist
if [ ! -f test.png ]; then
    echo "Creating test.png..."
    convert -size 800x600 gradient:blue-green test.png
fi

if [ ! -f test.mp3 ]; then
    echo "Creating silent test.mp3..."
    ffmpeg -f lavfi -i anullsrc=r=44100:cl=mono -t 1 -q:a 9 -acodec libmp3lame test.mp3 -y 2>/dev/null
fi

if [ ! -f test.mp4 ]; then
    echo "Creating test.mp4..."
    ffmpeg -f lavfi -i testsrc=duration=1:size=320x240:rate=30 -c:v libx264 test.mp4 -y 2>/dev/null
fi

# Create AI media files
echo -e "\n📸 Creating AI image..."
aimg create test.png -o output.aimg --model "DALL-E 3" --version "3.0"

echo -e "\n🎵 Creating AI audio..."
aaud create test.mp3 -o output.aaud --model "MusicGen" --version "1.0"

echo -e "\n🎬 Creating AI video..."
avid create test.mp4 -o output.avid --model "Sora" --version "1.0"

# Inspect files
echo -e "\n📊 Image metadata:"
aimg info output.aimg

echo -e "\n📊 Audio metadata:"
aaud info output.aaud

echo -e "\n📊 Video metadata:"
avid info output.avid

# Verify integrity
echo -e "\n✅ Verification results:"
aimg verify output.aimg && echo "  Image: ✓ Verified"
aaud verify output.aaud && echo "  Audio: ✓ Verified"
avid verify output.avid && echo "  Video: ✓ Verified"

echo -e "\n🎉 Demo complete! Files created:"
ls -lh output.*