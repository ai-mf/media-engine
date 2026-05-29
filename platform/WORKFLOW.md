media-engine/
├── platform/
│   ├── android/
│   │   ├── AndroidManifest.xml           ← Intent filters
│   │   └── res/xml/file_paths.xml        ← File provider
│   ├── ios/
│   │   ├── Info.plist                    ← UTI declarations
│   │   └── AIMF.uttype                   ← UTI definition
│   ├── web/
│   │   ├── aimf-mime.ts                  ← TS definitions
│   │   ├── sw.js                         ← Service worker
│   │   └── manifest.json                 ← PWA manifest
│   ├── flutter/
│   │   └── aimf_mime.dart                ← Dart definitions
│   ├── react-native/
│   │   └── aimf-mime.js                  ← RN definitions
│   └── electron/
│       └── aimf-mime.js                  ← Electron handlers
├── share/
│   ├── mime/packages/aimf.xml            ← Linux desktop
│   └── applications/aimf-viewer.desktop  ← Linux desktop
└── docs/
    └── PLATFORM_INTEGRATION.md           ← This documentation