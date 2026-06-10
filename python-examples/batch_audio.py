#!/usr/bin/env python3
"""Example: Batch process multiple audio files using AIMF Python wrapper"""

from aimf import AudioAI, AIMF
from pathlib import Path

def main():
    print("🚀 Batch Processing: Creating multiple audio files")
    
    # Create output directory
    output_dir = Path("./batch_output/audio")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate signing key
    key_path = output_dir / "batch.key"
    AIMF.generate_key(key_path)
    
    # Sample audio data (name, sample_rate, samples pattern)
    audio_files = [
        ("melody1", 44100, [0.1, -0.2, 0.3, -0.1, 0.4]),
        ("melody2", 22050, [0.5, -0.3, 0.2, -0.4, 0.1]),
        ("melody3", 48000, [0.2, -0.1, 0.4, -0.3, 0.2]),
    ]
    
    print(f"\n📦 Processing {len(audio_files)} audio files...\n")
    
    for i, (name, sample_rate, pattern) in enumerate(audio_files, 1):
        print(f"[{i}/{len(audio_files)}] Processing: {name}")
        
        # Repeat pattern to make longer audio
        samples = pattern * 1000
        
        output_path = output_dir / f"{name}.aaud"
        
        audio = AudioAI.from_samples(samples, sample_rate=sample_rate, channels=1)
        audio.with_model("BatchModel", "1.0")
        audio.with_key(key_path)
        audio.save(output_path)
        
        print(f"   ✅ Created: {output_path}")
    
    # Verify all files
    print("\n🔍 Verifying all created files...\n")
    for name, _, _ in audio_files:
        file_path = output_dir / f"{name}.aaud"
        result = AIMF.verify(file_path)
        
        if result["valid"]:
            print(f"   ✅ {name} - VERIFIED")
        else:
            print(f"   ❌ {name} - VERIFICATION FAILED")
            print(f"      {result['error']}")
    
    print("\n✅ Batch processing complete!")
    print(f"📁 Output directory: {output_dir}")

if __name__ == "__main__":
    main()