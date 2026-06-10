"""Video-specific helper for AIMF - using RAW binary"""

from typing import List, Union, Optional
from pathlib import Path
from .core import AIMF, MediaType

class VideoAI(AIMF):
    """Video-specific wrapper for AVID files using RAW binary"""
    
    @classmethod
    def from_frames(cls, frames: List[List[int]], width: int, height: int, fps: int = 30):
        """Create from RGB24 frame data"""
        return cls.from_video_frames(frames, width, height, fps)
    
    @classmethod
    def from_bytes(cls, video_bytes: bytes, width: int, height: int, fps: int = 30, frame_count: int = 0):
        """Create from raw RGB bytes"""
        return cls.from_video_bytes(video_bytes, width, height, fps, frame_count)
    
    @classmethod
    def from_file(cls, path: Union[str, Path]):
        """Load existing AVID file"""
        video = super().from_file(path)
        if video.media_type != MediaType.VIDEO:
            raise ValueError(f"File {path} is not a video file")
        return video