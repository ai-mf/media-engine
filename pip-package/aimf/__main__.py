#!/usr/bin/env python3
"""AIMF Python wrapper - calls the actual aimf binary"""

import sys
import subprocess
from .binary import ensure_binary

def main():
    try:
        aimf_bin = ensure_binary()
    except Exception as e:
        print(f"❌ Failed to install AIMF binary: {e}", file=sys.stderr)
        return 1
    
    result = subprocess.run([aimf_bin] + sys.argv[1:])
    return result.returncode

if __name__ == "__main__":
    sys.exit(main())