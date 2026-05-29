"""Verification utilities"""

import subprocess
from pathlib import Path
from typing import Union, Tuple

def verify(file_path: Union[str, Path]) -> Tuple[bool, str]:
    """
    Verify AIMF file integrity and signature
    
    Returns:
        (is_valid, message)
    """
    file_path = Path(file_path)
    
    # Detect file type by extension
    if file_path.suffix == '.aaud':
        cmd = ["aaud", "verify", str(file_path)]
    elif file_path.suffix == '.aimg':
        cmd = ["aimg", "verify", str(file_path)]
    elif file_path.suffix == '.avid':
        cmd = ["avid", "verify", str(file_path)]
    else:
        cmd = ["aimf", "verify", str(file_path)]
    
    result = subprocess.run(cmd, capture_output=True)
    is_valid = result.returncode == 0
    message = result.stdout.decode() + result.stderr.decode()
    
    return is_valid, message