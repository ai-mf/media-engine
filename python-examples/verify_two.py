#!/usr/bin/env python3
"""Check info and verify using VideoAI class"""

from aimf import VideoAI

file_path = "test_video_long.avid"

print("=" * 50)
print(f"Loading: {file_path}")
print("=" * 50)

# Load the file
video = VideoAI.from_file(file_path)

print(f"\n📋 METADATA:")
print(f"  Model: {video.model}")
print(f"  Version: {video.version}")
print(f"  Width: {video.video_width}")
print(f"  Height: {video.video_height}")
print(f"  FPS: {video.fps}")
print(f"  Frame Count: {video.frame_count}")

# Get detailed info
print("\n📋 DETAILED INFO:")
info = video.info()
if info.get("success"):
    print(f"  Media Type: {info.get('media_type')}")
    print(f"  Timestamp: {info.get('timestamp')}")
    print(f"  Signed: {info.get('signed')}")
    print(f"  Hash Valid: {info.get('hash_valid')}")

# Verify
print("\n🔍 VERIFICATION:")
verify = video.verify()
if verify.get("success"):
    print(f"  Verified: {verify.get('verified')}")
    print(f"  Hash Valid: {verify.get('hash_valid')}")
    print(f"  Signed: {verify.get('signed')}")
    if verify.get('signed'):
        print(f"  Signature Valid: {verify.get('signature_valid')}")

# Quick check
print("\n⚡ QUICK CHECK:")
if video.verify_simple():
    print("  PASSED ✓")
else:
    print("  FAILED ✗")

print("\n" + "=" * 50)