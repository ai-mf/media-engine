#!/usr/bin/env python3
"""Run all tiny examples safely"""

import subprocess
import sys
import time

def run_example(name, script):
    print(f"\n{'='*50}")
    print(f"Running: {name}")
    print('='*50)
    
    start_time = time.time()
    result = subprocess.run([sys.executable, script], capture_output=True, text=True)
    elapsed = time.time() - start_time
    
    print(result.stdout)
    if result.stderr:
        print("STDERR:", result.stderr)
    
    if result.returncode == 0:
        print(f"✅ {name} completed in {elapsed:.2f}s")
    else:
        print(f"❌ {name} failed (exit code: {result.returncode})")
    
    return result.returncode == 0

def main():
    examples = [
        ("Simple Test (tiny)", "simple_test.py"),
        ("Audio (0.01 sec)", "audio_example.py"),
        ("Image (8x8)", "image_example.py"),
        ("Video (4x4, 2 frames)", "video_example.py"),
    ]
    
    print("🎯 Running TINY AIMF Python examples")
    print("="*50)
    print("These examples are designed to NOT crash your laptop")
    print("="*50)
    
    success_count = 0
    for name, script in examples:
        if run_example(name, script):
            success_count += 1
    
    print(f"\n{'='*50}")
    print(f"Summary: {success_count}/{len(examples)} examples passed")
    print('='*50)

if __name__ == "__main__":
    main()