#!/bin/bash
# Install AIMF MIME types on macOS using Launch Services

# Create a property list for Uniform Type Identifier
cat > ~/Library/Mobile\ Documents/com~apple~CloudDocs/aimf-uti.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>UTImportedTypeDeclarations</key>
    <array>
        <dict>
            <key>UTTypeIdentifier</key>
            <string>com.aimf.video</string>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>avid</string>
                </array>
                <key>public.mime-type</key>
                <string>video/avid</string>
            </dict>
        </dict>
        <dict>
            <key>UTTypeIdentifier</key>
            <string>com.aimf.image</string>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>aimg</string>
                </array>
                <key>public.mime-type</key>
                <string>image/aimg</string>
            </dict>
        </dict>
        <dict>
            <key>UTTypeIdentifier</key>
            <string>com.aimf.audio</string>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>aaud</string>
                </array>
                <key>public.mime-type</key>
                <string>audio/aaud</string>
            </dict>
        </dict>
    </array>
</dict>
</plist>
EOF

# Import the UTI
plutil ~/Library/Mobile\ Documents/com~apple~CloudDocs/aimf-uti.plist
echo "✅ AIMF UTIs registered on macOS"