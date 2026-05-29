#!/usr/bin/env python3
"""Ultra-lightweight video example - tiny 4x4, 2 frames"""

from aimf import VideoAI
import math

def main():
    print("🤖 Generating ultra-tiny video...")
    
    # SUPER TINY video
    width = 4
    height = 4
    fps = 1  # 1 frame per second
    num_frames = 2  # Only 2 frames!
    
    print(f"   Creating {num_frames} frames at {width}x{height}...")
    
    # Generate simple frames
    frames = []
    for frame_num in range(num_frames):
        frame_data = []
        for y in range(height):
            for x in range(width):
                # Simple color pattern
                color = (x + y + frame_num * 50) % 256
                frame_data.append(color)
                frame_data.append(color)
                frame_data.append(color)
        frames.append(frame_data)
    
    # Generate tiny audio (0.1 seconds)
    sample_rate = 4000  # Very low
    duration = 0.1
    samples = []
    for i in range(int(sample_rate * duration)):
        t = i / sample_rate
        sample = math.sin(2 * math.pi * 440 * t) * 0.3
        samples.append(sample)
    
    print(f"   Adding {len(samples)} audio samples...")
    
    # Use AIMF Python wrapper
    video = VideoAI.from_frames(frames, width=width, height=height, fps=fps)
    video.with_audio(samples, sample_rate=sample_rate)
    video.with_model("test-ai", "1.0")
    video.save("test_video.avid")
    
    print("✅ Created test_video.avid (tiny video)")
    print(f"📊 Size: {width}x{height}, {num_frames} frames, {fps} fps")

if __name__ == "__main__":
    main()