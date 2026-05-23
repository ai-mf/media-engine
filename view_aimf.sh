#!/bin/bash
FILE="$1"
cd /home/ubuntu/Programs/ai/rust/media-engine/ai
cargo run --bin aimf -- view "$FILE"
