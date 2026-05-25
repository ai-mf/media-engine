#!/usr/bin/env python3
"""
AIMF Python wrapper - calls the actual aimf binary
Location: /home/ubuntu/Programs/ai/rust/media-engine/ai/pip-package/aimf/__main__.py
"""

import sys
import subprocess
import os
import shutil

def find_aimf_binary():
    """Find the aimf binary in common locations"""
    # Check if it's in PATH
    aimf = shutil.which('aimf')
    if aimf:
        return aimf
    
    # Check in ~/.cargo/bin
    cargo_bin = os.path.expanduser("~/.cargo/bin/aimf")
    if os.path.exists(cargo_bin):
        return cargo_bin
    
    # Check in /usr/local/bin
    if os.path.exists("/usr/local/bin/aimf"):
        return "/usr/local/bin/aimf"
    
    return None

def main():
    aimf_bin = find_aimf_binary()
    
    if not aimf_bin:
        print("❌ AIMF binary not found!", file=sys.stderr)
        print("Please install AIMF first:", file=sys.stderr)
        print("  cargo install aimf-cli", file=sys.stderr)
        print("  or download from: https://github.com/yourusername/aimf", file=sys.stderr)
        sys.exit(1)
    
    # Forward all arguments to the real aimf binary
    result = subprocess.run([aimf_bin] + sys.argv[1:])
    sys.exit(result.returncode)

if __name__ == "__main__":
    main()
