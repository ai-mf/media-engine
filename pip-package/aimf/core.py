"""Universal AIMF wrapper - handles all media types"""

import json
import subprocess
import tempfile
from pathlib import Path
from typing import Optional, Union, Dict, Any, List
from enum import Enum

class MediaType(Enum):
    AUDIO = "audio"
    IMAGE = "image"
    VIDEO = "video"
    AUTO = "auto"

class AIMF:
    """
    Universal AIMF wrapper - auto-detects media type
    Works with all formats: AAUD (audio), AIMG (image), AVID (video)
    """
    
    def __init__(self):
        self.media_type: Optional[MediaType] = None
        self.data: Dict[str, Any] = {}
        self.metadata: Dict[str, str] = {}
        self.key_path: Optional[str] = None
    
    @classmethod
    def from_file(cls, path: Union[str, Path]):
        """Load any AIMF file (auto-detects format)"""
        aimf = cls()
        path_str = str(path)
        # TODO: Implement actual file parsing
        # For now, just detect type from extension
        if path_str.endswith('.aaud'):
            aimf.media_type = MediaType.AUDIO
        elif path_str.endswith('.aimg'):
            aimf.media_type = MediaType.IMAGE
        elif path_str.endswith('.avid'):
            aimf.media_type = MediaType.VIDEO
        
        # Load the actual data from file
        # This would extract and parse the AIMF container
        aimf.data = {}  # Placeholder
        
        return aimf
    
    @classmethod
    def from_json(cls, json_data: Dict[str, Any], media_type: Optional[MediaType] = None):
        """Create from JSON data, optionally specify media type"""
        aimf = cls()
        aimf.data = json_data
        
        # Auto-detect if not specified
        if media_type is None:
            if "samples" in json_data or "sample_rate" in json_data:
                aimf.media_type = MediaType.AUDIO
            elif "pixels" in json_data or "width" in json_data:
                aimf.media_type = MediaType.IMAGE
            elif "frames" in json_data:
                aimf.media_type = MediaType.VIDEO
        else:
            aimf.media_type = media_type
        
        return aimf
    
    def with_model(self, model: str, version: str = "1.0"):
        """Set AI model metadata"""
        self.metadata["model"] = model
        self.metadata["version"] = version
        return self
    
    def with_key(self, key_path: Union[str, Path]):
        """Sign with private key"""
        self.key_path = str(key_path)
        return self
    
    def save(self, output_path: Union[str, Path]):
        """Save as AIMF file (extension determines format)"""
        output_path = Path(output_path)
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(self.data, f)
            f.flush()
            temp_path = f.name
        
        try:
            # Build universal aimf command
            cmd = ["aimf", "json", "--output", str(output_path)]
            
            if self.metadata.get("model"):
                cmd.extend(["--model", self.metadata["model"]])
            if self.metadata.get("version"):
                cmd.extend(["--version", self.metadata["version"]])
            if self.key_path:
                cmd.extend(["--key", self.key_path])
            if self.media_type and self.media_type != MediaType.AUTO:
                cmd.extend(["--type", self.media_type.value])
            
            # Run command with stdin from temp file
            with open(temp_path, 'r') as json_file:
                result = subprocess.run(
                    cmd, 
                    stdin=json_file, 
                    capture_output=True,
                    text=True
                )
            
            if result.returncode != 0:
                raise RuntimeError(f"AIMF failed: {result.stderr}")
        
        finally:
            # Clean up temp file
            Path(temp_path).unlink(missing_ok=True)
        
        return output_path
    
    @staticmethod
    def info(file_path: Union[str, Path]) -> Dict[str, Any]:
        """Get metadata from any AIMF file"""
        file_path = Path(file_path)
        cmd = ["aimf", "info", str(file_path)]
        
        result = subprocess.run(
            cmd, 
            capture_output=True, 
            text=True
        )
        
        return {
            "raw_output": result.stdout,
            "success": result.returncode == 0,
            "error": result.stderr if result.returncode != 0 else None
        }
    
    @staticmethod
    def verify(file_path: Union[str, Path]) -> Dict[str, Any]:
        """Verify any AIMF file"""
        file_path = Path(file_path)
        cmd = ["aimf", "verify", str(file_path)]
        
        result = subprocess.run(
            cmd, 
            capture_output=True, 
            text=True
        )
        
        return {
            "valid": result.returncode == 0,
            "output": result.stdout,
            "error": result.stderr if result.returncode != 0 else None
        }
    
    @staticmethod
    def extract(file_path: Union[str, Path], output_path: Union[str, Path]):
        """Extract original media from any AIMF file"""
        file_path = Path(file_path)
        output_path = Path(output_path)
        cmd = ["aimf", "extract", str(file_path), "--output", str(output_path)]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Extraction failed: {result.stderr}")
        
        return output_path
    
    @staticmethod
    def view(file_path: Union[str, Path]):
        """View any AIMF file with default player"""
        file_path = Path(file_path)
        cmd = ["aimf", "view", str(file_path)]
        subprocess.run(cmd)
    
    @staticmethod
    def sign(input_path: Union[str, Path], key_path: Union[str, Path], output_path: Union[str, Path]):
        """Sign an existing AIMF file"""
        input_path = Path(input_path)
        output_path = Path(output_path)
        cmd = ["aimf", "sign", "--input", str(input_path), 
               "--key", str(key_path), "--output", str(output_path)]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Signing failed: {result.stderr}")
        
        return output_path
    
    @staticmethod
    def batch(input_pattern: str, output_dir: Union[str, Path], **kwargs):
        """Batch process multiple files"""
        output_dir = Path(output_dir)
        cmd = ["aimf", "batch", "--input", input_pattern, "--output-dir", str(output_dir)]
        
        for key, value in kwargs.items():
            cmd.extend([f"--{key}", str(value)])
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.returncode == 0
    
    @staticmethod
    def generate_key(output_path: Union[str, Path]):
        """Generate Ed25519 key pair"""
        output_path = Path(output_path)
        cmd = ["aimf", "gen-key", "--output", str(output_path)]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Key generation failed: {result.stderr}")
        
        return output_path