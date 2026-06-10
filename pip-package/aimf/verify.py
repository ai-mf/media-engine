"""Verification utilities"""

import subprocess
from pathlib import Path
from typing import Union, Tuple

def verify(file_path: Union[str, Path]) -> Tuple[bool, str]:
    """Verify AIMF file integrity and signature"""
    cmd = ["aimf", "verify", str(file_path), "--json"]
    result = subprocess.run(cmd, capture_output=True)
    is_valid = result.returncode == 0
    message = result.stdout.decode() + result.stderr.decode()
    return is_valid, message