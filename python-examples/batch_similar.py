#!/usr/bin/env python3
"""Example: Batch process similar audio files using AIMF Python wrapper"""

from aimf import AudioAI
from pathlib import Path

def main():
    print("🚀 Batch Processing: Creating multiple audio files")
    
    # Create output directory
    output_dir = Path("./batch_output/audio")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Sample audio data
    audio_files = [
        ("melody1", 44100, [0.1, -0.2, 0.3, -0.1, 0.4]),
        ("melody2", 22050, [0.5, -0.3, 0.2, -0.4, 0.1]),
        ("melody3", 48000, [0.2, -0.1, 0.4, -0.3, 0.2]),
    ]
    
    print(f"\n📦 Processing {len(audio_files)} audio files...\n")
    
    for i, (name, sample_rate, samples) in enumerate(audio_files, 1):
        print(f"[{i}/{len(audio_files)}] Processing: {name}")
        
        output_path = output_dir / f"{name}.aaud"
        
        # Repeat samples to make longer audio
        long_samples = samples * 1000
        
        audio = AudioAI.from_samples(long_samples, sample_rate=sample_rate)
        audio.with_model("BatchModel", "1.0")
        audio.save(output_path)
        
        print(f"   ✅ Created: {output_path}")
    
    print("\n✅ Batch processing complete!")
    print(f"📁 Output directory: {output_dir}")

if __name__ == "__main__":
    main()