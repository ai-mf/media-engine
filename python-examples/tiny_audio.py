#!/usr/bin/env python3
"""Lightweight audio example - won't crash"""

from aimf import AudioAI
import math

def main():
    print("🤖 Simulating AI audio generation (lightweight)...")
    
    # SUPER SMALL - only 0.01 seconds
    sample_rate = 8000  # Low sample rate
    duration = 0.01     # 10 milliseconds only!
    num_samples = int(sample_rate * duration)  # Only 80 samples
    
    samples = []
    for i in range(num_samples):
        t = i / sample_rate
        sample = math.sin(2 * math.pi * 440 * t)
        samples.append(sample)
    
    print(f"   Creating audio with {num_samples} samples...")
    
    # Use AIMF Python wrapper
    audio = AudioAI.from_samples(samples, sample_rate=sample_rate)
    audio.with_model("test-ai", "1.0")
    audio.save("test_audio.aaud")
    
    print("✅ Created test_audio.aaud (tiny file)")
    print(f"📊 Size: {num_samples} samples, {sample_rate} Hz")

if __name__ == "__main__":
    main()