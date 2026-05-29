#!/usr/bin/env python3
"""Ultra-lightweight image example - tiny 8x8 image"""

from aimf import ImageAI

def main():
    print("🤖 Simulating AI image generation (tiny 8x8)...")
    
    # TINY image - only 8x8 pixels!
    width = 8
    height = 8
    pixels = []
    
    # Create a simple pattern
    for y in range(height):
        for x in range(width):
            r = (x * 32) % 256
            g = (y * 32) % 256
            b = 255 if (x + y) % 2 == 0 else 0
            pixels.append(r)
            pixels.append(g)
            pixels.append(b)
    
    print(f"   Creating {width}x{height} image ({len(pixels)} pixels)...")
    
    # Use AIMF Python wrapper
    image = ImageAI.from_pixels(pixels, width=width, height=height)
    image.with_model("test-ai", "1.0")
    image.save("test_image.aimg")
    
    print("✅ Created test_image.aimg (tiny 8x8 image)")
    print(f"📊 Size: {width}x{height} pixels")

if __name__ == "__main__":
    main()