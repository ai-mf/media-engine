"""Audio-specific helper for AIMF - using RAW binary"""

from typing import List, Union, Optional
from pathlib import Path
from .core import AIMF, MediaType

class AudioAI(AIMF):
    """Audio-specific wrapper for AAUD files using RAW binary"""
    
    @classmethod
    def from_samples(cls, samples: List[float], sample_rate: int = 44100, channels: int = 1):
        """Create from raw audio samples"""
        return cls.from_audio_samples(samples, sample_rate, channels)
    
    @classmethod
    def from_bytes(cls, audio_bytes: bytes, sample_rate: int = 44100, channels: int = 1):
        """Create from raw PCM16 bytes"""
        return cls.from_audio_bytes(audio_bytes, sample_rate, channels)
    
    @classmethod
    def from_file(cls, path: Union[str, Path]):
        """Load existing AAUD file"""
        audio = super().from_file(path)
        if audio.media_type != MediaType.AUDIO:
            raise ValueError(f"File {path} is not an audio file")
        return audio