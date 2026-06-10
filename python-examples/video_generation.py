#!/usr/bin/env python3
"""Example: Generate AI video using AIMF Python wrapper"""

from aimf import VideoAI
import math

def main():
    print("🤖 Simulating AI video generation...")
    
    width = 64
    height = 48
    fps = 10
    duration_secs = 1.0
    frame_count = int(fps * duration_secs)
    
    print(f"Generating {frame_count} frames ({duration_secs} seconds at {fps} fps)...")
    
    # Generate frames
    frames = []
    for frame_num in range(frame_count):
        frame = []
        for y in range(height):
            for x in range(width):
                # Moving dot effect
                dot_x = int((frame_num * 5) % width)
                dot_y = int((frame_num * 3) % height)
                
                if abs(x - dot_x) < 3 and abs(y - dot_y) < 3:
                    r, g, b = 255, 255, 255  # White dot
                else:
                    r = (x + frame_num) % 256
                    g = (y + frame_num * 2) % 256
                    b = (x + y + frame_num * 3) % 256
                
                frame.append(r)
                frame.append(g)
                frame.append(b)
        frames.append(frame)
        
        if frame_num % 5 == 0:
            print(f"Progress: {frame_num + 1}/{frame_count} frames")
    
    # Generate audio track
    sample_rate = 22050
    num_samples = int(sample_rate * duration_secs)
    audio_samples = []
    
    for i in range(num_samples):
        t = i / sample_rate
        sample = math.sin(2 * math.pi * 440 * t) * 0.5  # 440Hz tone
        audio_samples.append(sample)
    
    print("Generating audio track...")
    
    # Create video using the wrapper
    video = VideoAI.from_frames(frames, width=width, height=height, fps=fps)
    video.with_model("test-ai", "1.0")
    video.with_audio(audio_samples, sample_rate=sample_rate)
    video.save("test_video.avid")
    
    print(f"✅ Created test_video.avid ({frame_count} frames)")
    print("🎬 View with: aimf view test_video.avid")

if __name__ == "__main__":
    main()