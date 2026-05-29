"""Video-specific helper for AIMF"""

from typing import List, Optional, Any, Dict, Union
from pathlib import Path
from .core import AIMF, MediaType

class VideoAI(AIMF):
    """Video-specific wrapper for AVID files"""
    
    def __init__(self):
        super().__init__()
        self.data: Dict[str, Any] = {}
    
    @classmethod
    def from_frames(cls, frames: List[List[int]], width: int, height: int, fps: int = 30):
        """Create from RGB24 frame data"""
        video = cls()
        video.data = {
            "width": width,
            "height": height,
            "fps": fps,
            "frames": frames
        }
        video.media_type = MediaType.VIDEO
        return video
    
    @classmethod
    def from_file(cls, path: Union[str, Path]):
        """Load existing AVID file"""
        # Call parent class method
        video = super().from_file(path)
        # Ensure it's video type
        if video.media_type != MediaType.VIDEO:
            raise ValueError(f"File {path} is not a video file")
        return video
    
    def with_audio(self, samples: List[float], sample_rate: int = 44100):
        """Add audio track to video"""
        if self.data is None:
            self.data = {}
        self.data["audio"] = {
            "sample_rate": sample_rate,
            "samples": samples
        }
        return self