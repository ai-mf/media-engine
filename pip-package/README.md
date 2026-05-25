# AIMF CLI for Python

This package provides a Python wrapper for the AIMF CLI tool.

## Installation

```bash
pip install aimf-cli
Requirements

The actual AIMF binary will be installed automatically when you run:
bash

cargo install aimf-cli

Usage
bash

aimf --help
echo '{"width":2,"height":2,"pixels":[255,0,0]}' | aimf ingest --output test.aimg

