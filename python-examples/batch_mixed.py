#!/usr/bin/env python3
"""Example: Process mixed media types using AIMF Python wrapper"""

from aimf import AudioAI, ImageAI, VideoAI, AIMF
from pathlib import Path
import math

def main():
    print("🎯 Processing mixed media formats")
    
    # Create output directory
    output_dir = Path("./batch_output/mixed")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate signing key
    key_path = output_dir / "master.key"
    AIMF.generate_key(key_path)
    
    # 1. Audio job
    print("\n🎵 Creating audio...")
    audio_samples = []
    for i in range(int(44100 * 0.5)):
        t = i / 44100
        audio_samples.append(math.sin(2 * math.pi * 440 * t) * 0.5)
    
    audio = AudioAI.from_samples(audio_samples, sample_rate=44100, channels=1)
    audio.with_model("MusicGen", "1.0")
    audio.with_key(key_path)
    audio.save(output_dir / "piano_melody.aaud")
    print("   ✅ Created: piano_melody.aaud")
    
    # 2. Image job
    print("\n🎨 Creating image...")
    width, height = 200, 150
    pixels = []
    for y in range(height):
        for x in range(width):
            pixels.append((x * 255) // width)           # R gradient
            pixels.append((y * 255) // height)          # G gradient
            pixels.append(((x + y) * 255) // (width + height))  # B blend
    
    image = ImageAI.from_pixels(pixels, width=width, height=height)
    image.with_model("StableDiffusion", "1.5")
    image.with_key(key_path)
    image.save(output_dir / "sunset_scene.aimg")
    print("   ✅ Created: sunset_scene.aimg")
    
    # 3. Video job
    print("\n🎬 Creating video...")
    video_width, video_height = 64, 48
    fps = 10
    frames = []
    
    for frame_num in range(30):
        frame = []
        for y in range(video_height):
            for x in range(video_width):
                frame.append((x + frame_num) % 256)
                frame.append((y + frame_num * 2) % 256)
                frame.append((x + y + frame_num * 3) % 256)
        frames.append(frame)
    
    video = VideoAI.from_frames(frames, width=video_width, height=video_height, fps=fps)
    video.with_model("GenVideo", "1.0")
    video.with_key(key_path)
    video.save(output_dir / "animated_logo.avid")
    print("   ✅ Created: animated_logo.avid")
    
    # Verify all files
    print("\n🔍 Verifying all created files...\n")
    for filename in ["piano_melody.aaud", "sunset_scene.aimg", "animated_logo.avid"]:
        file_path = output_dir / filename
        result = AIMF.verify(file_path)
        
        if result["valid"]:
            # Get info
            info = AIMF.info(file_path)
            print(f"   ✅ {filename} - VERIFIED")
            
            # Extract model name from output
            for line in info["raw_output"].split('\n'):
                if "Model:" in line:
                    print(f"      {line.strip()}")
        else:
            print(f"   ❌ {filename} - VERIFICATION FAILED")
    
    print("\n✅ Mixed media processing complete!")
    print(f"📁 Output directory: {output_dir}")

if __name__ == "__main__":
    main()