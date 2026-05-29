#!/usr/bin/env python3
"""
AIMF Python wrapper - calls the actual aimf binary
"""

import sys
import subprocess
import os
import shutil

def find_aimf_binary():
    """Find the aimf binary in common locations"""
    aimf = shutil.which('aimf')
    if aimf:
        return aimf
    
    cargo_bin = os.path.expanduser("~/.cargo/bin/aimf")
    if os.path.exists(cargo_bin):
        return cargo_bin
    
    if os.path.exists("/usr/local/bin/aimf"):
        return "/usr/local/bin/aimf"
    
    if os.path.exists("./aimf"):
        return "./aimf"
    
    return None

def main():
    aimf_bin = find_aimf_binary()
    
    if not aimf_bin:
        print("❌ AIMF binary not found!", file=sys.stderr)
        print("\nPlease install AIMF first:", file=sys.stderr)
        print("  cargo install aimf-cli", file=sys.stderr)
        sys.exit(1)
    
    result = subprocess.run([aimf_bin] + sys.argv[1:])
    sys.exit(result.returncode)

if __name__ == "__main__":
    main()
