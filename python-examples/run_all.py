#!/usr/bin/env python3
"""Run all AIMF Python examples"""

import subprocess
import sys
from pathlib import Path

def run_example(name, script):
    print(f"\n{'='*60}")
    print(f"Running: {name}")
    print('='*60)
    
    result = subprocess.run([sys.executable, script], cwd=Path(script).parent)
    
    if result.returncode == 0:
        print(f"✅ {name} completed successfully")
    else:
        print(f"❌ {name} failed")
    
    return result.returncode == 0

def main():
    examples = [
        ("Audio Generation", "audio_example.py"),
        ("Image Generation", "image_example.py"),
        ("Video Generation", "video_example.py"),
        ("CSV Batch Processing", "csv_batch_example.py"),
        ("Batch Similar Files", "batch_similar_example.py"),
        ("Batch Mixed Media", "batch_mixed_example.py"),
    ]
    
    print("🎯 Running all AIMF Python examples")
    print("="*60)
    print("Note: Make sure aimf package is installed: pip install -e pip-package/")
    print("="*60)
    
    success_count = 0
    for name, script in examples:
        if run_example(name, script):
            success_count += 1
    
    print(f"\n{'='*60}")
    print(f"Summary: {success_count}/{len(examples)} examples passed")
    print('='*60)

if __name__ == "__main__":
    main()