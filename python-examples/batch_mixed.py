#!/usr/bin/env python3
"""Example: Process mixed media types using AIMF Python wrapper"""

from aimf import AudioAI, ImageAI, VideoAI
from pathlib import Path
import math

def main():
    print("🎯 Processing mixed media formats")
    
    # Create output directory
    output_dir = Path("./batch_output/mixed")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Define different media jobs
    jobs = [
        {
            "name": "piano_melody",
            "type": "audio",
            "data": {
                "sample_rate": 44100,
                "samples": generate_audio_samples(44100, 0.5),
                "model": "MusicGen"
            }
        },
        {
            "name": "sunset_scene", 
            "type": "image",
            "data": {
                "width": 200,
                "height": 150,
                "pixels": generate_image_pattern(200, 150),
                "model": "StableDiffusion"
            }
        },
        {
            "name": "animated_logo",
            "type": "video",
            "data": {
                "width": 64,
                "height": 48,
                "fps": 10,
                "frames": generate_video_frames(10, 64, 48),
                "audio": {
                    "sample_rate": 22050,
                    "samples": generate_audio_samples(22050, 1.0)
                },
                "model": "GenVideo"
            }
        }
    ]
    
    print(f"\n📦 Processing {len(jobs)} mixed media files...\n")
    
    for job in jobs:
        print(f"🎬 Processing: {job['name']} ({job['type']})")
        
        output_path = output_dir / f"{job['name']}.{get_extension(job['type'])}"
        
        if job['type'] == "audio":
            audio = AudioAI.from_samples(
                job['data']['samples'], 
                sample_rate=job['data']['sample_rate']
            )
            audio.with_model(job['data']['model'], "1.0")
            audio.save(output_path)
            
        elif job['type'] == "image":
            image = ImageAI.from_pixels(
                job['data']['pixels'],
                width=job['data']['width'],
                height=job['data']['height']
            )
            image.with_model(job['data']['model'], "1.0")
            image.save(output_path)
            
        elif job['type'] == "video":
            video = VideoAI.from_frames(
                job['data']['frames'],
                width=job['data']['width'],
                height=job['data']['height'],
                fps=job['data']['fps']
            )
            if 'audio' in job['data']:
                video.with_audio(
                    job['data']['audio']['samples'],
                    sample_rate=job['data']['audio']['sample_rate']
                )
            video.with_model(job['data']['model'], "1.0")
            video.save(output_path)
        
        print(f"   ✅ Created: {output_path}")
    
    print("\n✅ Mixed media processing complete!")
    print(f"📁 Output directory: {output_dir}")

def get_extension(media_type):
    return {"audio": "aaud", "image": "aimg", "video": "avid"}.get(media_type, "bin")

def generate_audio_samples(sample_rate, duration):
    num_samples = int(sample_rate * duration)
    samples = []
    for i in range(num_samples):
        t = i / sample_rate
        sample = math.sin(2 * math.pi * 440 * t) * 0.5
        samples.append(sample)
    return samples

def generate_image_pattern(width, height):
    pixels = []
    for y in range(height):
        for x in range(width):
            r = int((x * 255) / width)
            g = int((y * 255) / height)
            b = int(((x + y) * 255) / (width + height))
            pixels.append(r)
            pixels.append(g)
            pixels.append(b)
    return pixels

def generate_video_frames(frame_count, width, height):
    frames = []
    for frame_num in range(frame_count):
        frame_data = []
        for y in range(height):
            for x in range(width):
                r = (x + frame_num) % 256
                g = (y + frame_num * 2) % 256
                b = (x + y + frame_num * 3) % 256
                frame_data.append(r)
                frame_data.append(g)
                frame_data.append(b)
        frames.append(frame_data)
    return frames

if __name__ == "__main__":
    main()