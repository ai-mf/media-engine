#!/bin/bash
# Uninstall AIMF MIME types

sudo rm -f /usr/share/mime/packages/aimf.xml
sudo rm -f /usr/share/applications/aimf-viewer.desktop
sudo rm -rf /usr/share/icons/hicolor/*/mimetypes/video-avid.png
sudo rm -rf /usr/share/icons/hicolor/*/mimetypes/image-aimg.png
sudo rm -rf /usr/share/icons/hicolor/*/mimetypes/audio-aaud.png
sudo rm -f /usr/share/thumbnailers/aimf.thumbnailer

sudo update-mime-database /usr/share/mime
sudo update-desktop-database

echo "✅ AIMF MIME types uninstalled"