#!/usr/bin/env python3
"""Example: Batch process from CSV data using AIMF Python wrapper"""

from aimf import AudioAI, ImageAI, VideoAI, AIMF
from pathlib import Path
import math

def main():
    print("📊 Batch Processing from CSV data")
    
    # Create output directory
    output_dir = Path("./batch_output/csv")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate signing key
    key_path = output_dir / "batch.key"
    AIMF.generate_key(key_path)
    
    # Sample CSV data (in real scenario, you'd read from a file)
    csv_data = [
        ("audio", "voice_note", {"sample_rate": 44100}, [0.1, -0.2, 0.3]),
        ("image", "profile_pic", {"width": 100, "height": 75}, None),
        ("video", "short_clip", {"width": 64, "height": 48, "fps": 10, "frame_count": 5}, None),
    ]
    
    print(f"\n📦 Processing {len(csv_data)} items from CSV...\n")
    
    for i, (media_type, name, params, _) in enumerate(csv_data, 1):
        print(f"[{i}/{len(csv_data)}] Processing: {name} ({media_type})")
        
        output_path = output_dir / f"{name}.{get_extension(media_type)}"
        
        if media_type == "audio":
            sample_rate = params["sample_rate"]
            # Repeat pattern to make longer audio
            samples = [0.1, -0.2, 0.3, -0.1, 0.4] * 1000
            
            audio = AudioAI.from_samples(samples, sample_rate=sample_rate, channels=1)
            audio.with_model("CSV-Batch", "1.0")
            audio.with_key(key_path)
            audio.save(output_path)
            
        elif media_type == "image":
            width = params["width"]
            height = params["height"]
            pixels = generate_pattern(width, height)
            
            image = ImageAI.from_pixels(pixels, width=width, height=height)
            image.with_model("CSV-Batch", "1.0")
            image.with_key(key_path)
            image.save(output_path)
            
        elif media_type == "video":
            width = params["width"]
            height = params["height"]
            fps = params["fps"]
            frame_count = params["frame_count"]
            
            frames = generate_frames(frame_count, width, height)
            
            video = VideoAI.from_frames(frames, width=width, height=height, fps=fps)
            video.with_model("CSV-Batch", "1.0")
            video.with_key(key_path)
            video.save(output_path)
        
        print(f"   ✅ Created: {output_path}")
    
    print("\n✅ Batch CSV processing complete!")
    print(f"📁 Output directory: {output_dir}")

def get_extension(media_type):
    return {"audio": "aaud", "image": "aimg", "video": "avid"}.get(media_type, "bin")

def generate_pattern(width, height):
    pixels = []
    for y in range(height):
        for x in range(width):
            pixels.append(x % 256)
            pixels.append(y % 256)
            pixels.append((x + y) % 256)
    return pixels

def generate_frames(count, width, height):
    frames = []
    for frame_num in range(count):
        frame = []
        for y in range(height):
            for x in range(width):
                frame.append((x + frame_num) % 256)
                frame.append((y + frame_num * 2) % 256)
                frame.append((x + y + frame_num * 3) % 256)
        frames.append(frame)
    return frames

if __name__ == "__main__":
    main()