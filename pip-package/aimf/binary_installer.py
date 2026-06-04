"""Auto-install AIMF binary if missing"""

import subprocess
import sys
import platform
import urllib.request
import tarfile
import zipfile
from pathlib import Path

def install_aimf_binary():
    """Download and install pre-built AIMF binary"""
    
    system = platform.system()
    arch = platform.machine()
    
    # Map to release filenames
    if system == "Linux":
        if arch in ["x86_64", "amd64"]:
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-linux-x86_64.tar.gz"
        elif arch in ["aarch64", "arm64"]:
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-linux-arm64.tar.gz"
    elif system == "Darwin":  # macOS
        if arch == "x86_64":
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-macos-x86_64.tar.gz"
        else:  # M1/M2
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-macos-arm64.tar.gz"
    elif system == "Windows":
        url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-windows-x86_64.zip"
    else:
        raise RuntimeError(f"Unsupported platform: {system}")
    
    # Download and install to ~/.local/bin or %USERPROFILE%\.local\bin
    if system == "Windows":
        install_dir = Path.home() / ".local/bin"
    else:
        install_dir = Path.home() / ".local/bin"
    
    install_dir.mkdir(parents=True, exist_ok=True)
    
    # Download
    print(f"Downloading AIMF binary for {system}...")
    with urllib.request.urlopen(url) as response:
        data = response.read()
    
    # Extract and install
    if url.endswith(".tar.gz"):
        with tarfile.open(fileobj=response, mode='r|gz') as tar:
            tar.extractall(install_dir)
    elif url.endswith(".zip"):
        with zipfile.ZipFile(response) as zipf:
            zipf.extractall(install_dir)
    
    # Make executable on Unix
    if system != "Windows":
        binary = install_dir / "aimf"
        binary.chmod(0o755)
    
    print(f"✅ AIMF binary installed to {install_dir}")