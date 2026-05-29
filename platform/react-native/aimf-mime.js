// AIMF MIME types for React Native

import { Platform } from 'react-native';
import RNFS from 'react-native-fs';
import { PlatformFile, pickSingle } from 'react-native-document-picker';

export const AIMF_MIME_TYPES = {
  VIDEO: 'video/avid',
  IMAGE: 'image/aimg',
  AUDIO: 'audio/aaud',
};

export const detectAIMFType = async (uri) => {
  try {
    // Read first 12 bytes to detect
    const buffer = await RNFS.read(uri, 0, 12);
    const bytes = new Uint8Array(buffer);
    
    // Check PNG (AIMG)
    if (bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4E && bytes[3] === 0x47) {
      return AIMF_MIME_TYPES.IMAGE;
    }
    
    // Check WAV (AAUD)
    if (bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46) {
      if (bytes[8] === 0x57 && bytes[9] === 0x41 && bytes[10] === 0x56 && bytes[11] === 0x45) {
        return AIMF_MIME_TYPES.AUDIO;
      }
    }
    
    // Check MP4 (AVID)
    if (bytes[4] === 0x66 && bytes[5] === 0x74 && bytes[6] === 0x79 && bytes[7] === 0x70) {
      return AIMF_MIME_TYPES.VIDEO;
    }
    
    return null;
  } catch (error) {
    console.error('Error detecting AIMF type:', error);
    return null;
  }
};

export const pickAIMFFile = async () => {
  try {
    const result = await pickSingle({
      type: [AIMF_MIME_TYPES.VIDEO, AIMF_MIME_TYPES.IMAGE, AIMF_MIME_TYPES.AUDIO],
      allowMultiSelection: false,
    });
    
    const mimeType = await detectAIMFType(result.uri);
    return { ...result, mimeType };
  } catch (err) {
    if (!isCancel(err)) {
      console.error('Error picking file:', err);
    }
    return null;
  }
};

// Android: Add to AndroidManifest.xml
// iOS: Add to Info.plist (see above)