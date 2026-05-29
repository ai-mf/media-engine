"""Image-specific helper for AIMF"""

from typing import List, Dict, Any, Union
from pathlib import Path
from .core import AIMF, MediaType

class ImageAI(AIMF):
    """Image-specific wrapper for AIMG files"""
    
    def __init__(self):
        super().__init__()
        self.data: Dict[str, Any] = {}
    
    @classmethod
    def from_pixels(cls, pixels: List[int], width: int, height: int, format: str = "rgb8"):
        """Create from RGB8 pixel data"""
        image = cls()
        image.data = {
            "width": width,
            "height": height,
            "format": format,
            "pixels": pixels
        }
        image.media_type = MediaType.IMAGE
        return image
    
    @classmethod
    def from_file(cls, path: Union[str, Path]):
        """Load existing AIMG file"""
        # Call parent class method
        image = super().from_file(path)
        # Ensure it's image type
        if image.media_type != MediaType.IMAGE:
            raise ValueError(f"File {path} is not an image file")
        return image