from setuptools import setup

setup(
    name="aimf-cli",
    version="0.1.0",
    description="AI Media Format - CLI tool",
    author="Your Name",
    packages=["aimf"],
    entry_points={
        "console_scripts": [
            "aimf=aimf.__main__:main",
        ],
    },
    python_requires=">=3.6",
)
