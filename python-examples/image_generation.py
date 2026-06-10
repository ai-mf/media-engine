#!/usr/bin/env python3
"""Example: Generate AI image using AIMF Python wrapper"""

from aimf import ImageAI

def main():
    print("🤖 Simulating AI image generation...")
    
    # Create a simple pattern
    width = 100
    height = 100
    pixels = []
    
    # Create a colorful pattern
    for y in range(height):
        for x in range(width):
            r = (x * 85) % 256
            g = (y * 85) % 256
            b = 255 if (x + y) % 2 == 0 else 0
            pixels.append(r)
            pixels.append(g)
            pixels.append(b)
    
    # Generate signing key if needed
    import os
    key_path = "private.key"
    if not os.path.exists(key_path):
        from aimf import AIMF
        AIMF.generate_key(key_path)
    
    # Create image using the wrapper
    image = ImageAI.from_pixels(pixels, width=width, height=height)
    image.with_model("test-ai", "1.0")
    image.with_key(key_path)
    image.save("test_image.aimg")
    
    print("✅ Created test_image.aimg")
    print("📝 View with: aimf view test_image.aimg")

if __name__ == "__main__":
    main()