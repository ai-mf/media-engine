#!/usr/bin/env python3
"""Example: Generate AI video using AIMF Python wrapper"""

from aimf import VideoAI
import math

def main():
    print("🤖 Generating short video with audio...")
    
    # Small video for testing
    width = 32
    height = 24
    fps = 10
    duration = 1  # 1 second
    num_frames = fps * duration
    
    # Generate simple frames (moving pattern)
    frames = []
    for frame_num in range(num_frames):
        frame_data = []
        for y in range(height):
            for x in range(width):
                color = (x + y + frame_num * 10) % 256
                frame_data.append(color)
                frame_data.append(color)
                frame_data.append(color)
        frames.append(frame_data)
    
    # Generate simple audio (short beep)
    sample_rate = 8000  # Lower sample rate for testing
    samples = []
    for i in range(sample_rate * duration):
        t = i / sample_rate
        sample = math.sin(2 * math.pi * 440 * t) * 0.3
        samples.append(sample)
    
    # Use AIMF Python wrapper
    video = VideoAI.from_frames(frames, width=width, height=height, fps=fps)
    video.with_audio(samples, sample_rate=sample_rate)
    video.with_model("test-ai", "1.0")
    video.save("test_video.avid")
    
    print("✅ Created test_video.avid")
    print("🎬 View with: aimf view test_video.avid")

if __name__ == "__main__":
    main()