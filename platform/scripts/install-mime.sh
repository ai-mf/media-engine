#!/bin/bash
# Install AIMF MIME types system-wide (Linux)

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
RESET='\033[0m'

echo "Installing AIMF MIME types..."

# Copy MIME definition
sudo mkdir -p /usr/share/mime/packages
sudo cp share/mime/packages/aimf.xml /usr/share/mime/packages/

# Copy desktop entry
sudo mkdir -p /usr/share/applications
sudo cp share/applications/aimf-viewer.desktop /usr/share/applications/

# Copy icons (if they exist)
if [ -d "share/icons" ]; then
    sudo cp -r share/icons/* /usr/share/icons/
fi

# Copy thumbnailer (if exists)
if [ -f "share/thumbnailers/aimf.thumbnailer" ]; then
    sudo mkdir -p /usr/share/thumbnailers
    sudo cp share/thumbnailers/aimf.thumbnailer /usr/share/thumbnailers/
fi

# Update MIME database
sudo update-mime-database /usr/share/mime

# Update desktop database
sudo update-desktop-database

echo -e "${GREEN}✅ AIMF MIME types installed${RESET}"
echo ""
echo "You can now:"
echo "  - Double-click .avid/.aimg/.aaud files"
echo "  - File managers will show proper icons"
echo "  - 'file' command will recognize AIMF files"