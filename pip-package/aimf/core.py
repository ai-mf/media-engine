"""Universal AIMF wrapper - handles all media types using RAW binary"""

import json
import subprocess
import tempfile
import struct
import os
from pathlib import Path
from typing import Optional, Union, Dict, Any, List
from enum import Enum
from .binary import ensure_binary

class DeprecationError(Exception):
    pass

class MediaType(Enum):
    AUDIO = "audio"
    IMAGE = "image"
    VIDEO = "video"
    AUTO = "auto"

class AIMF:
    """Universal AIMF wrapper - auto-detects media type"""
    
    def __init__(self):
        self.media_type: Optional[MediaType] = None
        self.model: str = ""
        self.version: str = ""
        self.key_path: Optional[str] = None
        self._file_path: Optional[Path] = None  # Store path for verify/info
        
        # RAW binary data
        self.audio_bytes: Optional[bytes] = None
        self.sample_rate: Optional[int] = None
        self.channels: int = 1
        
        self.image_bytes: Optional[bytes] = None
        self.width: Optional[int] = None
        self.height: Optional[int] = None
        self.pixel_format: str = "rgb8"
        
        self.video_bytes: Optional[bytes] = None
        self.video_width: Optional[int] = None
        self.video_height: Optional[int] = None
        self.fps: Optional[int] = None
        self.frame_count: int = 0
    
    # ========== Factory Methods ==========
    
    @classmethod
    def from_audio_samples(cls, samples: List[float], sample_rate: int = 44100, channels: int = 1):
        """Create from raw audio samples"""
        obj = cls()
        obj.media_type = MediaType.AUDIO
        obj.sample_rate = sample_rate
        obj.channels = channels
        
        audio_bytes = bytearray()
        for s in samples:
            s = max(-1.0, min(1.0, s))
            sample_i16 = int(s * 32767)
            audio_bytes.extend(struct.pack('<h', sample_i16))
        obj.audio_bytes = bytes(audio_bytes)
        return obj
    
    @classmethod
    def from_audio_bytes(cls, audio_bytes: bytes, sample_rate: int = 44100, channels: int = 1):
        """Create from raw PCM16 bytes"""
        obj = cls()
        obj.media_type = MediaType.AUDIO
        obj.sample_rate = sample_rate
        obj.channels = channels
        obj.audio_bytes = audio_bytes
        return obj
    
    @classmethod
    def from_image_pixels(cls, pixels: List[int], width: int, height: int, pixel_format: str = "rgb8"):
        """Create from RGB8 pixel data"""
        obj = cls()
        obj.media_type = MediaType.IMAGE
        obj.width = width
        obj.height = height
        obj.pixel_format = pixel_format
        obj.image_bytes = bytes(pixels)
        return obj
    
    @classmethod
    def from_image_bytes(cls, image_bytes: bytes, width: int, height: int, pixel_format: str = "rgb8"):
        """Create from raw RGB bytes"""
        obj = cls()
        obj.media_type = MediaType.IMAGE
        obj.width = width
        obj.height = height
        obj.pixel_format = pixel_format
        obj.image_bytes = image_bytes
        return obj
    
    @classmethod
    def from_video_frames(cls, frames: List[List[int]], width: int, height: int, fps: int = 30):
        """Create from RGB24 frame data"""
        obj = cls()
        obj.media_type = MediaType.VIDEO
        obj.video_width = width
        obj.video_height = height
        obj.fps = fps
        obj.frame_count = len(frames)
        
        video_bytes = bytearray()
        for frame in frames:
            video_bytes.extend(bytes(frame))
        obj.video_bytes = bytes(video_bytes)
        return obj
    
    @classmethod
    def from_video_bytes(cls, video_bytes: bytes, width: int, height: int, fps: int = 30, frame_count: int = 0):
        """Create from raw RGB bytes"""
        obj = cls()
        obj.media_type = MediaType.VIDEO
        obj.video_width = width
        obj.video_height = height
        obj.fps = fps
        obj.video_bytes = video_bytes
        if frame_count == 0:
            frame_size = width * height * 3
            obj.frame_count = len(video_bytes) // frame_size
        else:
            obj.frame_count = frame_count
        return obj
    
    @classmethod
    def from_file(cls, path: Union[str, Path]):
        """Load any AIMF file (auto-detects format using JSON output)"""
        path = Path(path)
        obj = cls()
        obj._file_path = path
        
        # Detect type from extension
        if path.suffix == '.aaud':
            obj.media_type = MediaType.AUDIO
        elif path.suffix == '.aimg':
            obj.media_type = MediaType.IMAGE
        elif path.suffix == '.avid':
            obj.media_type = MediaType.VIDEO
        
        # Parse JSON output from aimf info
        result = subprocess.run(
            ["aimf", "info", "--json", str(path)],
            capture_output=True, text=True
        )
        
        if result.returncode == 0:
            try:
                data = json.loads(result.stdout)
                obj.model = data.get("model", "")
                obj.version = data.get("version", "")
                
                # Parse media-specific fields
                if obj.media_type == MediaType.AUDIO:
                    obj.sample_rate = data.get("sample_rate")
                    obj.channels = data.get("channels", 1)
                elif obj.media_type == MediaType.IMAGE:
                    obj.width = data.get("width")
                    obj.height = data.get("height")
                elif obj.media_type == MediaType.VIDEO:
                    obj.video_width = data.get("width")
                    obj.video_height = data.get("height")
                    obj.fps = data.get("fps")
            except json.JSONDecodeError:
                pass
        
        return obj
    
    @classmethod
    def from_json(cls, json_data: Dict[str, Any], media_type: Optional[MediaType] = None):
        """DEPRECATED: Use from_audio_samples/from_image_pixels/from_video_frames instead"""
        raise DeprecationError("from_json is deprecated. Use specific from_* methods with RAW binary")
    
    # ========== Chainable Setters ==========
    
    def with_model(self, model: str, version: str = "1.0"):
        """Set AI model metadata"""
        self.model = model
        self.version = version
        return self
    
    def with_key(self, key_path: Union[str, Path]):
        """Sign with private key"""
        self.key_path = str(key_path)
        return self
    
    def with_audio(self, samples: List[float], sample_rate: int = 44100):
        """Add audio to video"""
        self.sample_rate = sample_rate
        audio_bytes = bytearray()
        for s in samples:
            s = max(-1.0, min(1.0, s))
            sample_i16 = int(s * 32767)
            audio_bytes.extend(struct.pack('<h', sample_i16))
        self.audio_bytes = bytes(audio_bytes)
        return self
    
    # ========== Save Method ==========
    
    def save(self, output_path: Union[str, Path]):
        """Save as AIMF file using RAW binary format"""
        output_path = Path(output_path)
        
        if not self.model or not self.version:
            raise ValueError("Missing model name or version. Call with_model() first")
        
        # Build command based on media type
        if self.media_type == MediaType.AUDIO:
            if self.audio_bytes is None:
                raise ValueError("No audio data to save")
            if self.sample_rate is None:
                raise ValueError("Missing sample_rate")
            
            cmd = [
                "aimf", "raw",
                "--output", str(output_path),
                "--model", self.model,
                "--version", self.version,
                "--type", "audio",
                "--sample-rate", str(self.sample_rate),
                "--channels", str(self.channels)
            ]
            data = self.audio_bytes
            
        elif self.media_type == MediaType.IMAGE:
            if self.image_bytes is None:
                raise ValueError("No image data to save")
            if self.width is None or self.height is None:
                raise ValueError("Missing width/height")
            
            cmd = [
                "aimf", "raw",
                "--output", str(output_path),
                "--model", self.model,
                "--version", self.version,
                "--type", "image",
                "--width", str(self.width),
                "--height", str(self.height),
                "--format", self.pixel_format
            ]
            data = self.image_bytes
            
        elif self.media_type == MediaType.VIDEO:
            if self.video_bytes is None:
                raise ValueError("No video data to save")
            if self.video_width is None or self.video_height is None or self.fps is None:
                raise ValueError("Missing video metadata (width, height, fps)")
            
            combined = bytearray(self.video_bytes)
            if self.audio_bytes:
                combined.extend(self.audio_bytes)
            data = bytes(combined)
            
            cmd = [
                "aimf", "raw",
                "--output", str(output_path),
                "--model", self.model,
                "--version", self.version,
                "--type", "video",
                "--width", str(self.video_width),
                "--height", str(self.video_height),
                "--fps", str(self.fps),
                "--frame-count", str(self.frame_count)
            ]
            
            if self.audio_bytes:
                cmd.extend([
                    "--sample-rate", str(self.sample_rate or 44100),
                    "--channels", str(self.channels)
                ])
        else:
            raise ValueError("Unknown media type")
        
        if self.key_path:
            cmd.extend(["--key", self.key_path])
        
        # Write to temp file and pipe to command
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as f:
            f.write(data)
            temp_path = f.name
        
        try:
            with open(temp_path, 'rb') as f:
                result = subprocess.run(cmd, stdin=f, capture_output=True, text=True)
            
            if result.returncode != 0:
                raise RuntimeError(f"AIMF failed: {result.stderr}")
        finally:
            os.unlink(temp_path)
        
        return output_path
    
    # ========== Verification & Info Methods ==========
    
    def verify(self) -> Dict[str, Any]:
        """Verify this file (if loaded via from_file)"""
        if not self._file_path:
            raise ValueError("No file loaded. Use from_file() first.")
        return self.__class__.verify_file(self._file_path)
    
    def verify_simple(self) -> bool:
        """Quick verification - returns True/False"""
        if not self._file_path:
            raise ValueError("No file loaded. Use from_file() first.")
        return self.__class__.verify_simple_file(self._file_path)
    
    def info(self) -> Dict[str, Any]:
        """Get metadata for this file"""
        if not self._file_path:
            raise ValueError("No file loaded. Use from_file() first.")
        return self.__class__.info_file(self._file_path)
    
    # ========== Static Methods ==========
    
    @staticmethod
    def info_file(file_path: Union[str, Path]) -> Dict[str, Any]:
        """Get metadata from any AIMF file (parses JSON output)"""
        ensure_binary()  # Make sure binary exists
        cmd = ["aimf", "info", "--json", str(file_path)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            return {
                "success": False,
                "error": result.stderr,
                "raw_output": result.stdout
            }
        
        try:
            data = json.loads(result.stdout)
            data["success"] = True
            return data
        except json.JSONDecodeError:
            return {
                "success": True,
                "raw_output": result.stdout,
                "error": None
            }
    
    @staticmethod
    def verify_file(file_path: Union[str, Path]) -> Dict[str, Any]:
        """Verify any AIMF file (parses JSON output)"""
        cmd = ["aimf", "verify", "--json", str(file_path)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            return {
                "verified": False,
                "success": False,
                "error": result.stderr,
                "raw_output": result.stdout
            }
        
        try:
            data = json.loads(result.stdout)
            data["success"] = True
            return data
        except json.JSONDecodeError:
            return {
                "verified": result.returncode == 0,
                "success": True,
                "raw_output": result.stdout,
                "error": None
            }
    
    @staticmethod
    def verify_simple_file(file_path: Union[str, Path]) -> bool:
        """Quick verification - returns just PASSED/FAILED"""
        cmd = ["aimf", "verify", "--simple", str(file_path)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.returncode == 0 and "PASSED" in result.stdout
    
    @staticmethod
    def extract(file_path: Union[str, Path], output_path: Union[str, Path]):
        """Extract original media from any AIMF file"""
        cmd = ["aimf", "extract", str(file_path), "--output", str(output_path)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Extraction failed: {result.stderr}")
        return output_path
    
    @staticmethod
    def view(file_path: Union[str, Path]):
        """View any AIMF file with default player"""
        subprocess.run(["aimf", "view", str(file_path)])
    
    @staticmethod
    def sign(input_path: Union[str, Path], key_path: Union[str, Path], output_path: Union[str, Path]):
        """Sign an existing AIMF file"""
        cmd = ["aimf", "sign", "--input", str(input_path), 
               "--key", str(key_path), "--output", str(output_path)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Signing failed: {result.stderr}")
        return output_path
    
    @staticmethod
    def generate_key(output_path: Union[str, Path]):
        """Generate Ed25519 key pair"""
        cmd = ["aimf", "gen-key", "--output", str(output_path)]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Key generation failed: {result.stderr}")
        return output_path