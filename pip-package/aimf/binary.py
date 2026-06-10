"""AIMF binary management - download and locate the binary"""

import subprocess
import sys
import platform
import urllib.request
import os
from pathlib import Path

def _find_binary():
    """Find aimf binary in common locations"""
    import shutil
    
    # Check if it's in PATH
    aimf = shutil.which('aimf')
    if aimf:
        return aimf
    
    # Check common install locations
    locations = [
        Path.home() / ".local/bin/aimf",
        Path.home() / ".cargo/bin/aimf",
        Path("/usr/local/bin/aimf"),
    ]
    
    # Check Windows-specific locations
    if platform.system() == "Windows":
        locations.append(Path.home() / "AppData/Local/Programs/aimf/aimf.exe")
    
    for loc in locations:
        if loc.exists():
            return str(loc)
    
    return None

def ensure_binary():
    """Ensure AIMF binary is installed, download if missing"""
    aimf_bin = _find_binary()
    if aimf_bin:
        return aimf_bin
    
    # Not found, try to download
    print("🔧 AIMF binary not found. Downloading...", file=sys.stderr)
    
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
    
    print(f"   Downloading from: {url}", file=sys.stderr)
    urllib.request.urlretrieve(url, binary_path)
    
    if system != "Windows":
        binary_path.chmod(0o755)
    
    print(f"✅ AIMF binary installed to: {binary_path}", file=sys.stderr)
    
    # Add to PATH reminder for Windows
    if system == "Windows":
        print(f"   Add {install_dir} to your PATH to use 'aimf' command", file=sys.stderr)
    
    return str(binary_path)