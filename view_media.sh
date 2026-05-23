#!/bin/bash

FILE="$1"
EXT="${FILE##*.}"

# Create temp file
TEMP_FILE="/tmp/aimf_view_$$"

case "$EXT" in
    aimg)
        # Extract image and convert to PNG if needed
        TEMP_FILE="${TEMP_FILE}.png"
        cargo run --bin aimf -- extract "$FILE" --output "$TEMP_FILE" 2>/dev/null
        # Try different viewers
        if command -v eog &>/dev/null; then
            eog "$TEMP_FILE" &
        elif command -v display &>/dev/null; then
            display "$TEMP_FILE" &
        elif command -v feh &>/dev/null; then
            feh "$TEMP_FILE" &
        else
            echo "No image viewer found. File extracted to: $TEMP_FILE"
        fi
        ;;
    
    aaud)
        # Extract audio
        TEMP_FILE="${TEMP_FILE}.wav"
        cargo run --bin aimf -- extract "$FILE" --output "$TEMP_FILE" 2>/dev/null
        # Try different audio players
        if command -v aplay &>/dev/null; then
            aplay "$TEMP_FILE" &
        elif command -v play &>/dev/null; then
            play "$TEMP_FILE" &
        elif command -v vlc &>/dev/null; then
            vlc --play-and-exit "$TEMP_FILE" &
        else
            echo "No audio player found. File extracted to: $TEMP_FILE"
        fi
        ;;
    
    avid)
        # Extract video
        TEMP_FILE="${TEMP_FILE}.mp4"
        cargo run --bin aimf -- extract "$FILE" --output "$TEMP_FILE" 2>/dev/null
        # Try different video players
        if command -v vlc &>/dev/null; then
            vlc "$TEMP_FILE" &
        elif command -v mpv &>/dev/null; then
            mpv "$TEMP_FILE" &
        elif command -v mplayer &>/dev/null; then
            mplayer "$TEMP_FILE" &
        else
            echo "No video player found. File extracted to: $TEMP_FILE"
        fi
        ;;
esac

# Clean up temp file after 30 seconds (adjust as needed)
(sleep 30; rm -f "$TEMP_FILE") &
