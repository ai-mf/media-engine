#!/usr/bin/env python3
"""Just verify files - doesn't create anything new"""

from aimf import AIMF
from pathlib import Path

def main():
    print("🔍 Verifying AIMF files...")
    
    # Check if test files exist
    test_files = ["test_audio.aaud", "test_image.aimg", "test_video.avid"]
    
    for file in test_files:
        if Path(file).exists():
            print(f"\n📁 Checking: {file}")
            result = AIMF.verify(file)
            print(f"   Valid: {result['valid']}")
            
            info = AIMF.info(file)
            print(f"   Info success: {info['success']}")
        else:
            print(f"\n⚠️ {file} not found - run examples first")

if __name__ == "__main__":
    main()