"""Image-specific helper for AIMF - using RAW binary"""

from typing import List, Union, Optional
from pathlib import Path
from .core import AIMF, MediaType

class ImageAI(AIMF):
    """Image-specific wrapper for AIMG files using RAW binary"""
    
    @classmethod
    def from_pixels(cls, pixels: List[int], width: int, height: int, pixel_format: str = "rgb8"):
        """Create from RGB8 pixel data"""
        return cls.from_image_pixels(pixels, width, height, pixel_format)
    
    @classmethod
    def from_bytes(cls, image_bytes: bytes, width: int, height: int, pixel_format: str = "rgb8"):
        """Create from raw RGB bytes"""
        return cls.from_image_bytes(image_bytes, width, height, pixel_format)
    
    @classmethod
    def from_file(cls, path: Union[str, Path]):
        """Load existing AIMG file"""
        image = super().from_file(path)
        if image.media_type != MediaType.IMAGE:
            raise ValueError(f"File {path} is not an image file")
        return image