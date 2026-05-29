"""Audio-specific helper for AIMF"""

from typing import List, Dict, Any, Union
from pathlib import Path
from .core import AIMF, MediaType

class AudioAI(AIMF):
    """Audio-specific wrapper for AAUD files"""
    
    def __init__(self):
        super().__init__()
        self.data: Dict[str, Any] = {}
    
    @classmethod
    def from_samples(cls, samples: List[float], sample_rate: int = 44100, channels: int = 1):
        """Create from raw audio samples"""
        audio = cls()
        audio.data = {
            "sample_rate": sample_rate,
            "channels": channels,
            "samples": samples
        }
        audio.media_type = MediaType.AUDIO
        return audio
    
    @classmethod
    def from_file(cls, path: Union[str, Path]):
        """Load existing AAUD file"""
        # Call parent class method
        audio = super().from_file(path)
        # Ensure it's audio type
        if audio.media_type != MediaType.AUDIO:
            raise ValueError(f"File {path} is not an audio file")
        return audio