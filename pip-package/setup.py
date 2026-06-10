from setuptools import setup
from setuptools.command.install import install
import subprocess
import sys
import platform
import urllib.request
import tarfile
import zipfile
import os
from pathlib import Path

# Get version from environment variable (set by GitHub Actions)
version = os.environ.get('RELEASE_VERSION', '1.0.0')

# Read README
readme_path = Path(__file__).parent / "README.md"
long_description = readme_path.read_text() if readme_path.exists() else ""

class DownloadBinary(install):
    """Custom install command to download AIMF binary"""
    
    def run(self):
        # First, do normal Python package install
        install.run(self)
        
        # Then download and install the binary
        print("\n📦 Downloading AIMF binary...")
        
        system = platform.system()
        arch = platform.machine()
        
        # Determine download URL based on platform
        if system == "Linux":
            if arch in ["x86_64", "amd64"]:
                url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-linux-x86_64.tar.gz"
            elif arch in ["aarch64", "arm64"]:
                url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-linux-arm64.tar.gz"
            else:
                print(f"⚠️ Unsupported architecture: {arch}")
                return
        elif system == "Darwin":  # macOS
            if arch == "x86_64":
                url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-macos-x86_64.tar.gz"
            else:  # Apple Silicon
                url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-macos-arm64.tar.gz"
        elif system == "Windows":
            url = "https://github.com/ai-mf/media-engine/releases/download/v1.0.0/aimf-windows-x86_64.zip"
        else:
            print(f"⚠️ Unsupported OS: {system}")
            return
        
        # Install to user's local bin
        if system == "Windows":
            install_dir = Path.home() / "AppData/Local/Programs/aimf"
            bin_dir = install_dir
        else:
            install_dir = Path.home() / ".local"
            bin_dir = install_dir / "bin"
        
        bin_dir.mkdir(parents=True, exist_ok=True)
        
        # Download
        print(f"   Downloading from: {url}")
        try:
            with urllib.request.urlopen(url) as response:
                data = response.read()
        except Exception as e:
            print(f"   ❌ Download failed: {e}")
            print("   Please download manually from: https://github.com/ai-mf/media-engine/releases")
            return
        
        # Save temp file
        temp_file = install_dir / f"aimf_download{'.zip' if system == 'Windows' else '.tar.gz'}"
        with open(temp_file, 'wb') as f:
            f.write(data)
        
        # Extract
        print("   Extracting...")
        if system == "Windows":
            with zipfile.ZipFile(temp_file, 'r') as zipf:
                zipf.extractall(bin_dir)
        else:
            with tarfile.open(temp_file, 'r:gz') as tar:
                tar.extractall(bin_dir)
        
        # Make executable on Unix
        if system != "Windows":
            binary = bin_dir / "aimf"
            if binary.exists():
                binary.chmod(0o755)
        
        # Clean up
        temp_file.unlink()
        
        # Add to PATH reminder for Windows
        if system == "Windows":
            print(f"\n✅ AIMF binary installed to: {bin_dir}")
            print("   Add this to your PATH environment variable to use 'aimf' command")
        else:
            print(f"\n✅ AIMF binary installed to: {bin_dir}")
            print("   You can now use 'aimf' command")

setup(
    name="aimf",
    version="1.0.0",
    author="AIMF Contributors",
    author_email="aimediaformat@gmail.com",
    description="AI Media Format - Python wrapper for authenticating AI-generated audio, images, and video",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/ai-mf/media-engine",
    project_urls={
        "Bug Reports": "https://github.com/ai-mf/media-engine/issues",
        "Source": "https://github.com/ai-mf/media-engine",
        "Documentation": "https://github.com/ai-mf/media-engine",
    },
    packages=["aimf"],
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "Topic :: Multimedia :: Sound/Audio",
        "Topic :: Multimedia :: Graphics",
        "Topic :: Multimedia :: Video",
        "License :: OSI Approved :: Apache Software License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    python_requires=">=3.7",
    install_requires=[],
    cmdclass={'install': DownloadBinary},
    entry_points={
        "console_scripts": [
            "aimf=aimf.__main__:main",
        ],
    },
    include_package_data=True,
    zip_safe=False,
)