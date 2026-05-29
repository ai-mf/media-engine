"""
AIMF - AI Media Format for Python
"""

import subprocess
import sys
import platform
import urllib.request
import os
from pathlib import Path

__version__ = "0.1.0"

def _ensure_binary():
    """Ensure AIMF binary is installed"""
    
    # Check if binary exists
    aimf_bin = _find_binary()
    if aimf_bin:
        return aimf_bin
    
    # Not found, try to download
    print("🔧 AIMF binary not found. Downloading...")
    
    system = platform.system()
    arch = platform.machine()
    
    # Determine download URL
    if system == "Linux":
        if arch in ["x86_64", "amd64"]:
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-linux-x86_64"
        elif arch in ["aarch64", "arm64"]:
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-linux-arm64"
        else:
            raise RuntimeError(f"Unsupported Linux architecture: {arch}")
    elif system == "Darwin":
        if arch == "x86_64":
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-macos-x86_64"
        else:
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-macos-arm64"
    elif system == "Windows":
        url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-windows-x86_64.exe"
    else:
        raise RuntimeError(f"Unsupported platform: {system}")
    
    # Download to user directory
    if system == "Windows":
        install_dir = Path.home() / "AppData/Local/Programs/aimf"
        binary_path = install_dir / "aimf.exe"
    else:
        install_dir = Path.home() / ".local/bin"
        binary_path = install_dir / "aimf"
    
    install_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"   Downloading from: {url}")
    urllib.request.urlretrieve(url, binary_path)
    
    if system != "Windows":
        binary_path.chmod(0o755)
    
    print(f"✅ AIMF binary installed to: {binary_path}")
    
    # Add to PATH reminder
    if system == "Windows":
        print(f"   Add {install_dir} to your PATH to use 'aimf' command")
    
    return str(binary_path)

def _find_binary():
    """Find aimf binary in common locations"""
    
    # Check if it's in PATH
    import shutil
    aimf = shutil.which('aimf')
    if aimf:
        return aimf
    
    # Check common install locations
    locations = [
        Path.home() / ".local/bin/aimf",
        Path.home() / ".cargo/bin/aimf",
        Path("/usr/local/bin/aimf"),
    ]
    
    for loc in locations:
        if loc.exists():
            return str(loc)
    
    return None

"""
AIMF - AI Media Format for Python
Universal wrapper for all AIMF commands (audio, image, video)
"""

# Export main classes
from .core import AIMF, MediaType
from .audio import AudioAI
from .image import ImageAI
from .video import VideoAI

__all__ = ["AIMF", "MediaType", "AudioAI", "ImageAI", "VideoAI"]
