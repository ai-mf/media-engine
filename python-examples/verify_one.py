#!/usr/bin/env python3
"""Check info and verify AVID file"""

from aimf import AIMF

file_path = "test_video_long.avid"

print("=" * 50)
print(f"Checking: {file_path}")
print("=" * 50)

# Get info with JSON output
print("\n📋 FILE INFO:")
info = AIMF.info_file(file_path)

if info.get("success"):
    print(f"  Media Type: {info.get('media_type')}")
    print(f"  Model: {info.get('model')}")
    print(f"  Version: {info.get('version')}")
    print(f"  Width: {info.get('width')}")
    print(f"  Height: {info.get('height')}")
    print(f"  FPS: {info.get('fps')}")
    print(f"  Timestamp: {info.get('timestamp')}")
    print(f"  Signed: {info.get('signed')}")
    print(f"  Hash Valid: {info.get('hash_valid')}")

    pub_key = info.get('public_key')

    if pub_key:
        print(f"  Public Key: {pub_key[:16]}...")
    else:
        print("  Public Key: N/A")
else:
    print(f"  Error: {info.get('error')}")
    print(f"  Raw: {info.get('raw_output')}")

# Verify the file
print("\n🔍 VERIFICATION:")
verify = AIMF.verify_file(file_path)

if verify.get("success"):
    print(f"  Verified: {verify.get('verified')}")
    print(f"  Hash Valid: {verify.get('hash_valid')}")
    print(f"  Signed: {verify.get('signed')}")
    print(f"  Signature Valid: {verify.get('signature_valid')}")

    pub_key = verify.get('public_key')

    if pub_key:
        print(f"  Public Key: {pub_key[:16]}...")
    else:
        print("  Public Key: N/A")
else:
    print(f"  Error: {verify.get('error')}")

# Quick simple check
print("\n⚡ QUICK CHECK:")
if AIMF.verify_simple_file(file_path):
    print("  PASSED ✓")
else:
    print("  FAILED ✗")

print("\n" + "=" * 50)