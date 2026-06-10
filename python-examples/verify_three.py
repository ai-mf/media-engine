#!/usr/bin/env python3
"""Quick check - minimal version"""

from aimf import AIMF
import json

file_path = "test_video_long.avid"

# Info
print("INFO:")
info = AIMF.info_file(file_path)
if info.get("success"):
    print(f"  Model: {info['model']} v{info['version']}")
    print(f"  Size: {info['width']}x{info['height']} @ {info['fps']}fps")
    print(f"  Signed: {info['signed']}")
else:
    print(f"  Error: {info.get('error')}")

# Verify
print("\nVERIFY:")
verify = AIMF.verify_file(file_path)
if verify.get("success"):
    print(f"  Result: {'✅ PASSED' if verify['verified'] else '❌ FAILED'}")
    print(f"  Hash: {'✅' if verify['hash_valid'] else '❌'}")
    if verify['signed']:
        print(f"  Signature: {'✅' if verify['signature_valid'] else '❌'}")
else:
    print(f"  Error: {verify.get('error')}")