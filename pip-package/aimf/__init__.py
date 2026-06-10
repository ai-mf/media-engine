"""
AIMF - AI Media Format for Python
"""

# Export main classes
from .core import AIMF, MediaType, DeprecationError
from .audio import AudioAI
from .image import ImageAI
from .video import VideoAI

__version__ = "1.0.0"
__all__ = ["AIMF", "MediaType", "AudioAI", "ImageAI", "VideoAI", "DeprecationError"]