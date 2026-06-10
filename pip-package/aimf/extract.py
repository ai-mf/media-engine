"""Extraction utilities"""

import subprocess
from pathlib import Path
from typing import Union

def extract(aimf_file: Union[str, Path], output_path: Union[str, Path]):
    """Extract original media from AIMF container"""
    cmd = ["aimf", "extract", str(aimf_file), "--output", str(output_path)]
    result = subprocess.run(cmd)
    if result.returncode != 0:
        raise RuntimeError(f"Extraction failed")
    return output_path