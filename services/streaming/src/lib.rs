// media-engine/services/streaming/src/lib.rs
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub struct StreamingService;

impl StreamingService {
    /// Stream-process a large file in chunks
    pub fn process_in_chunks<F>(
        input_path: &Path,
        output_path: &Path,
        chunk_size: usize,
        mut processor: F,
    ) -> Result<()>
    where
        F: FnMut(&[u8]) -> Result<Vec<u8>>,
    {
        let input_file = File::open(input_path)
            .with_context(|| format!("Failed to open {}", input_path.display()))?;
        
        let file_size = input_file.metadata()?.len();
        let mut reader = BufReader::with_capacity(chunk_size, input_file);
        let mut writer = BufWriter::new(
            File::create(output_path)
                .with_context(|| format!("Failed to create {}", output_path.display()))?
        );
        
        let mut buffer = vec![0u8; chunk_size];
        let mut total_read = 0u64;
        
        loop {
            let bytes_read = reader.read(&mut buffer)
                .context("Failed to read from input")?;
            
            if bytes_read == 0 {
                break;
            }
            
            let processed = processor(&buffer[..bytes_read])
                .context("Failed to process chunk")?;
            
            writer.write_all(&processed)
                .context("Failed to write to output")?;
            
            total_read += bytes_read as u64;
            
            // Optional: report progress
            if file_size > 0 && total_read % (file_size / 100).max(1) == 0 {
                let progress = (total_read as f64 / file_size as f64 * 100.0) as u32;
                eprint!("\rProcessing: {}%", progress);
            }
        }
        
        writer.flush().context("Failed to flush output")?;
        
        if file_size > 0 {
            eprintln!("\rProcessing: 100%");
        }
        
        Ok(())
    }
    
    /// Stream video frames individually
    pub fn process_video_frames<F>(
        input_path: &Path,
        width: u32,
        height: u32,
        mut frame_processor: F,
    ) -> Result<Vec<Vec<u8>>>
    where
        F: FnMut(&[u8], usize) -> Result<Vec<u8>>,
    {
        let frame_size = (width * height * 3) as usize;
        let input_file = File::open(input_path)
            .with_context(|| format!("Failed to open {}", input_path.display()))?;
        
        let file_size = input_file.metadata()?.len() as usize;
        let total_frames = file_size / frame_size;
        
        if file_size % frame_size != 0 {
            anyhow::bail!(
                "File size {} is not a multiple of frame size {}",
                file_size, frame_size
            );
        }
        
        let mut reader = BufReader::new(input_file);
        let mut frame_buffer = vec![0u8; frame_size];
        let mut frames = Vec::with_capacity(total_frames);
        
        for frame_num in 0..total_frames {
            reader.read_exact(&mut frame_buffer)
                .with_context(|| format!("Failed to read frame {}", frame_num))?;
            
            let processed = frame_processor(&frame_buffer, frame_num)
                .with_context(|| format!("Failed to process frame {}", frame_num))?;
            
            frames.push(processed);
            
            if total_frames > 0 && frame_num % (total_frames / 10).max(1) == 0 {
                let progress = (frame_num as f64 / total_frames as f64 * 100.0) as u32;
                eprint!("\rProcessing frames: {}%", progress);
            }
        }
        
        eprintln!("\rProcessing frames: 100%");
        
        Ok(frames)
    }
    
    /// Create a temporary file for large intermediate data
    pub fn create_temp_file(prefix: &str) -> Result<tempfile::NamedTempFile> {
        tempfile::Builder::new()
            .prefix(prefix)
            .tempfile()
            .context("Failed to create temporary file")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use std::io::Write;

    #[test]
    fn test_process_in_chunks_small_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let output_path = dir.path().join("output.txt");
        
        // Create test file
        let test_data = b"Hello, World! This is test data for chunk processing.";
        fs::write(&input_path, test_data).unwrap();
        
        // Process in small chunks
        let result = StreamingService::process_in_chunks(
            &input_path,
            &output_path,
            10, // 10 byte chunks
            |chunk| {
                // Simple uppercase transformation
                Ok(chunk.iter().map(|&b| b.to_ascii_uppercase()).collect())
            },
        );
        
        assert!(result.is_ok());
        
        // Verify output
        let output_data = fs::read(&output_path).unwrap();
        let expected: Vec<u8> = test_data.iter().map(|&b| b.to_ascii_uppercase()).collect();
        assert_eq!(output_data, expected);
    }

    #[test]
    fn test_process_in_chunks_empty_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("empty.txt");
        let output_path = dir.path().join("output.txt");
        
        fs::write(&input_path, b"").unwrap();
        
        let result = StreamingService::process_in_chunks(
            &input_path,
            &output_path,
            1024,
            |chunk| Ok(chunk.to_vec()),
        );
        
        assert!(result.is_ok());
        
        let output_data = fs::read(&output_path).unwrap();
        assert_eq!(output_data, b"");
    }

    #[test]
    fn test_process_in_chunks_large_chunk() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let output_path = dir.path().join("output.txt");
        
        let test_data = vec![b'A'; 10000];
        fs::write(&input_path, &test_data).unwrap();
        
        let result = StreamingService::process_in_chunks(
            &input_path,
            &output_path,
            5000, // Chunk larger than file
            |chunk| Ok(chunk.to_vec()),
        );
        
        assert!(result.is_ok());
        
        let output_data = fs::read(&output_path).unwrap();
        assert_eq!(output_data, test_data);
    }

    #[test]
    fn test_process_in_chunks_exact_chunks() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let output_path = dir.path().join("output.txt");
        
        let test_data = vec![b'X'; 1000];
        fs::write(&input_path, &test_data).unwrap();
        
        let result = StreamingService::process_in_chunks(
            &input_path,
            &output_path,
            100, // Exactly divides 1000
            |chunk| Ok(chunk.to_vec()),
        );
        
        assert!(result.is_ok());
        
        let output_data = fs::read(&output_path).unwrap();
        assert_eq!(output_data, test_data);
    }

    #[test]
    fn test_process_in_chunks_with_transformation() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.bin");
        let output_path = dir.path().join("output.bin");
        
        let test_data: Vec<u8> = (0..100).collect();
        fs::write(&input_path, &test_data).unwrap();
        
        let result = StreamingService::process_in_chunks(
            &input_path,
            &output_path,
            20,
            |chunk| {
                // Double each byte
                Ok(chunk.iter().flat_map(|&b| vec![b, b]).collect())
            },
        );
        
        assert!(result.is_ok());
        
        let output_data = fs::read(&output_path).unwrap();
        let expected: Vec<u8> = test_data.iter().flat_map(|&b| vec![b, b]).collect();
        assert_eq!(output_data, expected);
    }

    #[test]
    fn test_process_video_frames() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("video.raw");
        
        // Create test video frames (3x3 RGB, 2 frames)
        let width = 3;
        let height = 3;
        let frame_size = (width * height * 3) as usize;
        
        let frame1: Vec<u8> = (0..frame_size).map(|i| (i % 255) as u8).collect();
        let frame2: Vec<u8> = (0..frame_size).map(|i| ((i + 100) % 255) as u8).collect();
        
        let mut video_data = Vec::new();
        video_data.extend_from_slice(&frame1);
        video_data.extend_from_slice(&frame2);
        
        fs::write(&input_path, &video_data).unwrap();
        
        let result = StreamingService::process_video_frames(
            &input_path,
            width,
            height,
            |frame, _frame_num| {
                // Simple: invert each frame
                let inverted: Vec<u8> = frame.iter().map(|&p| 255 - p).collect();
                Ok(inverted)
            },
        );
        
        assert!(result.is_ok());
        let frames = result.unwrap();
        
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].len(), frame_size);
        assert_eq!(frames[1].len(), frame_size);
        
        // Verify inversion
        for i in 0..frame_size {
            assert_eq!(frames[0][i], 255 - frame1[i]);
            assert_eq!(frames[1][i], 255 - frame2[i]);
        }
    }

    #[test]
    fn test_process_video_frames_invalid_size() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("invalid.raw");
        
        // Create data that's not a multiple of frame size
        let data = vec![0u8; 100]; // Not divisible by 27 (3x3x3)
        fs::write(&input_path, &data).unwrap();
        
        let result = StreamingService::process_video_frames(
            &input_path,
            3,
            3,
            |frame, _| Ok(frame.to_vec()),
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a multiple"));
    }

    #[test]
    fn test_process_video_frames_single_frame() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("single.raw");
        
        let width = 2;
        let height = 2;
        let frame_size = (width * height * 3) as usize;
        let frame: Vec<u8> = (0..frame_size).map(|i| i as u8).collect();
        
        fs::write(&input_path, &frame).unwrap();
        
        let result = StreamingService::process_video_frames(
            &input_path,
            width,
            height,
            |frame, _| Ok(frame.to_vec()),
        );
        
        assert!(result.is_ok());
        let frames = result.unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], frame);
    }

    #[test]
    fn test_create_temp_file() {
        let temp_file = StreamingService::create_temp_file("test_prefix").unwrap();
        let path = temp_file.path().to_path_buf();
        
        // Verify file exists
        assert!(path.exists());
        
        // Can write to it
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"test data").unwrap();
        
        // Verify content
        let content = fs::read(&path).unwrap();
        assert_eq!(content, b"test data");
    }

    #[test]
    fn test_create_multiple_temp_files() {
        let temp1 = StreamingService::create_temp_file("test1").unwrap();
        let temp2 = StreamingService::create_temp_file("test2").unwrap();
        
        // Should be different files
        assert_ne!(temp1.path(), temp2.path());
        
        // Both should exist
        assert!(temp1.path().exists());
        assert!(temp2.path().exists());
    }

    #[test]
    fn test_process_in_chunks_with_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.txt");
        let output_path = dir.path().join("output.txt");
        
        fs::write(&input_path, b"test data").unwrap();
        
        let result = StreamingService::process_in_chunks(
            &input_path,
            &output_path,
            10,
            |_chunk| {
                Err(anyhow::anyhow!("Simulated processing error"))
            },
        );
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // The error might be wrapped with context
        assert!(err.contains("processing") || err.contains("Failed"));
    }

    #[test]
    fn test_process_video_frames_with_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("video.raw");
        
        let frame: Vec<u8> = vec![0u8; 27]; // 3x3x3
        fs::write(&input_path, &frame).unwrap();
        
        let result = StreamingService::process_video_frames(
            &input_path,
            3,
            3,
            |_, frame_num| {
                if frame_num == 0 {
                    Err(anyhow::anyhow!("Error processing frame 0"))
                } else {
                    Ok(frame.to_vec())
                }
            },
        );
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // The error might be wrapped with context
        assert!(err.contains("frame") || err.contains("Failed"));
    }

    #[test]
    fn test_process_in_chunks_nonexistent_input() {
        let result = StreamingService::process_in_chunks(
            Path::new("/nonexistent/path/file.txt"),
            Path::new("/tmp/output.txt"),
            1024,
            |chunk| Ok(chunk.to_vec()),
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to open"));
    }
}