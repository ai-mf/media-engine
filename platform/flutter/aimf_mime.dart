// AIMF MIME type definitions for Flutter

class AIMFMimeTypes {
  static const String video = 'video/avid';
  static const String image = 'image/aimg';
  static const String audio = 'audio/aaud';
  
  static const Map<String, String> extensionToMime = {
    'avid': video,
    'aimg': image,
    'aaud': audio,
  };
  
  static const Map<String, List<String>> mimeToExtensions = {
    video: ['avid'],
    image: ['aimg'],
    audio: ['aaud'],
  };
  
  static String? detectFromBytes(Uint8List bytes) {
    // Check magic bytes
    if (bytes.length >= 4) {
      // PNG (AIMG)
      if (bytes[0] == 0x89 && bytes[1] == 0x50 && bytes[2] == 0x4E && bytes[3] == 0x47) {
        return image;
      }
      
      // WAV (AAUD)
      if (bytes[0] == 0x52 && bytes[1] == 0x49 && bytes[2] == 0x46 && bytes[3] == 0x46) {
        if (bytes.length >= 12 && bytes[8] == 0x57 && bytes[9] == 0x41 && bytes[10] == 0x56 && bytes[11] == 0x45) {
          return audio;
        }
      }
      
      // MP4 (AVID)
      if (bytes[4] == 0x66 && bytes[5] == 0x74 && bytes[6] == 0x79 && bytes[7] == 0x70) {
        return video;
      }
    }
    
    return null;
  }
}

// Android intent filter setup (in AndroidManifest.xml)
/*
<activity android:name=".MainActivity">
    <intent-filter>
        <action android:name="android.intent.action.VIEW" />
        <category android:name="android.intent.category.DEFAULT" />
        <category android:name="android.intent.category.BROWSABLE" />
        <data android:scheme="file" />
        <data android:scheme="content" />
        <data android:mimeType="video/avid" />
        <data android:mimeType="image/aimg" />
        <data android:mimeType="audio/aaud" />
    </intent-filter>
</activity>
*/

// iOS Info.plist additions
/*
<key>CFBundleDocumentTypes</key>
<array>
    <dict>
        <key>CFBundleTypeName</key>
        <string>AIMF File</string>
        <key>LSItemContentTypes</key>
        <array>
            <string>com.aimf.video</string>
            <string>com.aimf.image</string>
            <string>com.aimf.audio</string>
        </array>
        <key>LSHandlerRank</key>
        <string>Default</string>
    </dict>
</array>
*/