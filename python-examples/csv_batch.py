#!/usr/bin/env python3
"""Example: Batch process from CSV using AIMF Python wrapper"""

from aimf import AudioAI, ImageAI, VideoAI
from pathlib import Path
import math

def main():
    print("📊 Batch Processing from CSV data")
    
    # Create output directory
    output_dir = Path("./batch_output/csv")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Sample CSV data (in real scenario, you'd read from a file)
    csv_data = [
        ("audio", "voice_note", "44100", [0.1, -0.2, 0.3]),
        ("image", "profile_pic", "100x75", None),
        ("video", "short_clip", "64x48@10", None),
    ]
    
    print(f"\n📦 Processing {len(csv_data)} items from CSV...\n")
    
    for i, (media_type, name, params, _) in enumerate(csv_data, 1):
        print(f"[{i}/{len(csv_data)}] Processing: {name} ({media_type})")
        
        output_path = output_dir / f"{name}.{get_extension(media_type)}"
        
        if media_type == "audio":
            sample_rate = int(params)
            samples = [0.1, -0.2, 0.3, -0.1, 0.4] * 1000  # Longer sample
            audio = AudioAI.from_samples(samples, sample_rate=sample_rate)
            audio.with_model("CSV-Batch", "1.0")
            audio.save(output_path)
            
        elif media_type == "image":
            dimensions = params.split('x')
            width = int(dimensions[0])
            height = int(dimensions[1])
            pixels = generate_pattern(width, height)
            image = ImageAI.from_pixels(pixels, width=width, height=height)
            image.with_model("CSV-Batch", "1.0")
            image.save(output_path)
            
        elif media_type == "video":
            parts = params.split('@')
            dimensions = parts[0].split('x')
            width = int(dimensions[0])
            height = int(dimensions[1])
            fps = int(parts[1])
            frames = generate_frames(5, width, height)  # Short video
            video = VideoAI.from_frames(frames, width=width, height=height, fps=fps)
            video.with_model("CSV-Batch", "1.0")
            video.save(output_path)
        
        print(f"   ✅ Created: {output_path}")
    
    print("\n✅ Batch CSV processing complete!")
    print(f"📁 Output directory: {output_dir}")

def get_extension(media_type):
    return {
        "audio": "aaud",
        "image": "aimg", 
        "video": "avid"
    }.get(media_type, "bin")

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
    for _ in range(count):
        frames.append(generate_pattern(width, height))
    return frames

if __name__ == "__main__":
    main()