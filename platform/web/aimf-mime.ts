// AIMF MIME type definitions for web applications

export const AIMF_MIME_TYPES = {
  VIDEO: 'video/avid',
  IMAGE: 'image/aimg',
  AUDIO: 'audio/aaud',
} as const;

export const AIMF_EXTENSIONS = {
  VIDEO: '.avid',
  IMAGE: '.aimg',
  AUDIO: '.aaud',
} as const;

export const AIMF_MAGIC_BYTES = {
  AVID: { offset: 4, bytes: [0x66, 0x74, 0x79, 0x70] }, // 'ftyp'
  AIMG: { offset: 0, bytes: [0x89, 0x50, 0x4E, 0x47] }, // PNG
  AAUD: { offset: 0, bytes: [0x52, 0x49, 0x46, 0x46] }, // 'RIFF'
} as const;

// Detect AIMF type from file
export function detectAIMFType(buffer: ArrayBuffer): string | null {
  const bytes = new Uint8Array(buffer);
  
  // Check for PNG (AIMG)
  if (bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4E && bytes[3] === 0x47) {
    // Look for AIMG text chunk
    if (hasAIMGTEXtChunk(buffer)) {
      return AIMF_MIME_TYPES.IMAGE;
    }
  }
  
  // Check for WAV (AAUD)
  if (bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46) {
    if (bytes[8] === 0x57 && bytes[9] === 0x41 && bytes[10] === 0x56 && bytes[11] === 0x45) {
      // Check for AAUD LIST chunk
      if (hasAAUDListChunk(buffer)) {
        return AIMF_MIME_TYPES.AUDIO;
      }
    }
  }
  
  // Check for MP4 (AVID)
  if (bytes[4] === 0x66 && bytes[5] === 0x74 && bytes[6] === 0x79 && bytes[7] === 0x70) {
    if (hasAVIDUUIDBox(buffer)) {
      return AIMF_MIME_TYPES.VIDEO;
    }
  }
  
  return null;
}

function hasAIMGTEXtChunk(buffer: ArrayBuffer): boolean {
  // Implementation would scan PNG chunks for 'AIMG' keyword
  // Simplified for example
  const text = new TextDecoder().decode(buffer);
  return text.includes('AIMG');
}

function hasAAUDListChunk(buffer: ArrayBuffer): boolean {
  const text = new TextDecoder().decode(buffer);
  return text.includes('AAUD');
}

function hasAVIDUUIDBox(buffer: ArrayBuffer): boolean {
  const text = new TextDecoder().decode(buffer);
  return text.includes('AVID');
}

// Web API for file handling
export async function registerAIMFFileHandlers() {
  if ('showOpenFilePicker' in window) {
    const opts = {
      types: [
        {
          description: 'AIMF Video Files',
          accept: { 'video/avid': ['.avid'] }
        },
        {
          description: 'AIMF Image Files',
          accept: { 'image/aimg': ['.aimg'] }
        },
        {
          description: 'AIMF Audio Files',
          accept: { 'audio/aaud': ['.aaud'] }
        }
      ],
      excludeAcceptAllOption: true,
      multiple: false,
    };
    
    try {
      const [handle] = await (window as any).showOpenFilePicker(opts);
      const file = await handle.getFile();
      const buffer = await file.arrayBuffer();
      const mimeType = detectAIMFType(buffer);
      console.log(`Detected: ${mimeType}`);
      return { file, mimeType };
    } catch (err) {
      console.error('File picker cancelled or failed', err);
      return null;
    }
  }
}