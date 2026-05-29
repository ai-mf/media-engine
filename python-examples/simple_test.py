#!/usr/bin/env python3
"""Simplest possible test - minimal data"""

from aimf import AudioAI, ImageAI

def main():
    print("🧪 Testing AIMF Python wrapper (minimal)...")
    
    # Test 1: Tiny audio (1 sample!)
    print("\n1. Testing audio...")
    audio = AudioAI.from_samples([0.5], sample_rate=1000)
    audio.with_model("Test", "1.0")
    audio.save("test1.aaud")
    print("   ✅ test1.aaud created")
    
    # Test 2: Tiny image (1x1 pixel!)
    print("\n2. Testing image...")
    image = ImageAI.from_pixels([255, 0, 0], width=1, height=1)
    image.with_model("Test", "1.0")
    image.save("test2.aimg")
    print("   ✅ test2.aimg created")
    
    print("\n✅ All tests passed! Files created:")
    print("   - test1.aaud (1 sample)")
    print("   - test2.aimg (1x1 pixel)")

if __name__ == "__main__":
    main()