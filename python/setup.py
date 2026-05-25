from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="aimf",
    version="0.1.0",
    author="AI Media Format Team",
    description="Verifiable AI media format for provenance tracking",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/ai-mf/media-engine",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Multimedia :: Graphics",
        "Topic :: Multimedia :: Sound/Audio",
        "Topic :: Multimedia :: Video",
    ],
    python_requires=">=3.8",
    install_requires=[
        "requests>=2.25.0",  # For API client
    ],
    extras_require={
        "dev": ["pytest", "black", "mypy"],
        "server": ["fastapi", "uvicorn"],
    },
    entry_points={
        "console_scripts": [
            "aimf-python=aimf.cli:main",
        ],
    },
)