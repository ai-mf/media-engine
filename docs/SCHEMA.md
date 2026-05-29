//media-engine/docs/SCHEMA.md

# AIMF JSON Input Schema

## Image Input
```json
{
  "width": 1024,           // Required: positive integer
  "height": 1024,          // Required: positive integer  
  "pixels": [255,0,0,...], // Required: array of RGB bytes (length = width*height*3)
  "model": "model-name",   // Optional: AI model identifier
  "version": "1.0",         // Optional: model version
  "key": "private.key"         // Optional: sign key
}

Audio Input
json

{
  "sample_rate": 44100,    // Required: positive integer (1-384000)
  "samples": [0.5, -0.3],  // Required: array of floats (-1.0 to 1.0)
  "model": "model-name",   // Optional
  "version": "1.0" ,        // Optional
  "key": "private.key"        // Optional
}

Video Input
json

{
  "width": 1920,           // Required
  "height": 1080,          // Required
  "fps": 30,               // Required (1-240)
  "frames": [[R,G,B,...]], // Required: array of frame pixel arrays
  "audio": {               // Optional
    "sample_rate": 44100,
    "samples": [0.5, -0.3]
  },
  "key": "private.key"        // Optional
}