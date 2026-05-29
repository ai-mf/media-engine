"""Extraction utilities"""

import subprocess
from pathlib import Path
from typing import Union

def extract(aimf_file: Union[str, Path], output_path: Union[str, Path]):
    """
    Extract original media from AIMF container
    """
    aimf_file = Path(aimf_file)
    output_path = Path(output_path)
    
    if aimf_file.suffix == '.aaud':
        cmd = ["aaud", "extract", str(aimf_file), "--output", str(output_path)]
    elif aimf_file.suffix == '.aimg':
        cmd = ["aimg", "extract", str(aimf_file), "--output", str(output_path)]
    elif aimf_file.suffix == '.avid':
        cmd = ["avid", "extract", str(aimf_file), "--output", str(output_path)]
    else:
        cmd = ["aimf", "extract", str(aimf_file), "--output", str(output_path)]
    
    result = subprocess.run(cmd)
    if result.returncode != 0:
        raise RuntimeError(f"Extraction failed")
    
    return output_path