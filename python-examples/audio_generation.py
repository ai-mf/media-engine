#!/usr/bin/env python3
"""Example: Generate AI audio using AIMF Python wrapper"""

from aimf import AudioAI
import math

def main():
    print("🤖 Simulating AI audio generation...")
    
    # Create a simple sine wave (0.1 seconds at 44.1kHz)
    sample_rate = 44100
    duration = 0.1
    num_samples = int(sample_rate * duration)
    
    samples = []
    for i in range(num_samples):
        t = i / sample_rate
        sample = math.sin(2 * math.pi * 440 * t)
        samples.append(sample)
    
    # Use AIMF Python wrapper
    audio = AudioAI.from_samples(samples, sample_rate=sample_rate)
    audio.with_model("test-ai", "1.0")
    audio.save("test_audio.aaud")
    
    print("✅ Created test_audio.aaud")
    print("🔊 View with: aimf view test_audio.aaud")

if __name__ == "__main__":
    main()