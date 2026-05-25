"""
AIMF - AI Media Format
Production-ready Python bindings for verifiable AI-generated media
"""

import json
import subprocess
import tempfile
from pathlib import Path
from typing import Dict, Any, Optional, Union, BinaryIO
from dataclasses import dataclass
import os

@dataclass
class AIMFMetadata:
    """AI provenance metadata"""
    model: str
    version: str
    timestamp: str
    organization: Optional[str] = None
    prompt_hash: Optional[str] = None
    
    def to_dict(self) -> Dict:
        return {k: v for k, v in self.__dict__.items() if v is not None}

class AIMFContainer:
    """Base class for AIMF containers"""
    
    def __init__(self, path: Union[str, Path]):
        self.path = Path(path)
        if not self.path.exists():
            raise FileNotFoundError(f"{self.path} not found")
    
    def verify(self) -> bool:
        """Verify cryptographic integrity"""
        result = subprocess.run(
            ["aimf", "verify", str(self.path)],
            capture_output=True,
            text=True
        )
        return result.returncode == 0
    
    def info(self) -> Dict[str, Any]:
        """Extract metadata"""
        result = subprocess.run(
            ["aimf", "info", str(self.path), "--json"],
            capture_output=True,
            text=True
        )
        if result.returncode != 0:
            raise RuntimeError(f"Failed to extract info: {result.stderr}")
        return json.loads(result.stdout)
    
    def extract(self, output_path: Union[str, Path]) -> None:
        """Extract original media without metadata"""
        subprocess.run(
            ["aimf", "extract", str(self.path), "--output", str(output_path)],
            check=True
        )

class AIMFImage(AIMFContainer):
    """AIMG (AI Image) container"""
    pass

class AIMFAudio(AIMFContainer):
    """AAUD (AI Audio) container"""
    pass

class AIMFVideo(AIMFContainer):
    """AVID (AI Video) container"""
    pass

def create_aimf(
    media_data: Union[bytes, BinaryIO, Path],
    metadata: AIMFMetadata,
    output_path: Union[str, Path],
    media_type: str = "image"  # image, audio, video
) -> None:
    """
    Create an AIMF file from raw media data
    
    Example:
        >>> from aimf import create_aimf, AIMFMetadata
        >>> with open("image.png", "rb") as f:
        ...     create_aimf(
        ...         f.read(),
        ...         AIMFMetadata(model="stable-diffusion", version="1.5"),
        ...         "output.aimg"
        ...     )
    """
    
    # Read bytes if path or file object
    if isinstance(media_data, Path):
        media_data = media_data.read_bytes()
    elif hasattr(media_data, 'read'):
        media_data = media_data.read()
    
    # Create temp file for input
    with tempfile.NamedTemporaryFile(suffix=f".{media_type}") as tmp_input:
        tmp_input.write(media_data)
        tmp_input.flush()
        
        # Build command
        cmd = [
            "aimf", "ingest",
            "--input", tmp_input.name,
            "--output", str(output_path),
            "--model", metadata.model,
            "--version", metadata.version
        ]
        
        if metadata.organization:
            cmd.extend(["--organization", metadata.organization])
        if metadata.prompt_hash:
            cmd.extend(["--prompt-hash", metadata.prompt_hash])
        
        subprocess.run(cmd, check=True)

# Easy-to-use functions for AI pipelines
def aimg_from_json(json_data: Dict[str, Any], output_path: Union[str, Path]) -> None:
    """
    Create AIMG from JSON output of AI model
    
    Example:
        >>> ai_output = {
        ...     "type": "image",
        ...     "width": 512,
        ...     "height": 512,
        ...     "pixels": [...],  # RGB values
        ...     "model": "sdxl",
        ...     "version": "1.0"
        ... }
        >>> aimg_from_json(ai_output, "generated.aimg")
    """
    cmd = [
        "aimf", "ingest",
        "--output", str(output_path),
        "--model", json_data.get("model", "unknown"),
        "--version", json_data.get("version", "1.0")
    ]
    
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    stdout, stderr = proc.communicate(json.dumps(json_data))
    
    if proc.returncode != 0:
        raise RuntimeError(f"Failed to create AIMG: {stderr}")

def aa_from_json(json_data: Dict[str, Any], output_path: Union[str, Path]) -> None:
    """Create AAUD from JSON audio data"""
    # Similar to aimg_from_json
    pass

def avid_from_json(json_data: Dict[str, Any], output_path: Union[str, Path]) -> None:
    """Create AVID from JSON video data with audio"""
    # Similar to aimg_from_json
    pass