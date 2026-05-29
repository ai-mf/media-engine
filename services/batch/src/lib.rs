// media-engine/services/batch/src/lib.rs
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug)]
pub struct BatchConfig {
    pub input_pattern: String,
    pub output_dir: PathBuf,
    pub recursive: bool,
    pub continue_on_error: bool,
}

#[derive(Debug)]
pub struct BatchResult {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<(PathBuf, String)>,
    pub output_files: Vec<PathBuf>,
}

pub struct BatchService;

impl BatchService {
    /// Process multiple files with a single handler
    pub fn process_batch<F>(
        config: &BatchConfig,
        mut file_handler: F,
    ) -> Result<BatchResult>
    where
        F: FnMut(&Path, &Path) -> Result<()>,
    {
        let input_files = Self::collect_files(config)?;
        let total = input_files.len();
        
        println!("📁 Found {} files to process", total);
        
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        let mut output_files = Vec::new();
        
        for (index, input_path) in input_files.iter().enumerate() {
            let output_path = Self::get_output_path(input_path, &config.output_dir);
            
            print!("[{}/{}] {} ... ", index + 1, total, input_path.display());
            
            match file_handler(input_path, &output_path) {
                Ok(()) => {
                    println!("✅");
                    successful += 1;
                    output_files.push(output_path);
                }
                Err(e) => {
                    println!("❌ {}", e);
                    failed += 1;
                    errors.push((input_path.clone(), e.to_string()));
                    
                    if !config.continue_on_error {
                        anyhow::bail!("Aborting due to error (use --continue-on-error to skip)");
                    }
                }
            }
        }
        
        Ok(BatchResult {
            total_files: total,
            successful,
            failed,
            errors,
            output_files,
        })
    }
    
    /// Collect all input files based on config
    fn collect_files(config: &BatchConfig) -> Result<Vec<PathBuf>> {
        let path = Path::new(&config.input_pattern);
        
        if path.is_file() {
            return Ok(vec![path.to_path_buf()]);
        }
        
        if path.is_dir() {
            let mut files = Vec::new();
            Self::scan_directory(path, config.recursive, &mut files)?;
            
            if files.is_empty() {
                anyhow::bail!("No files found in directory: {}", config.input_pattern);
            }
            
            return Ok(files);
        }
        
        // Try glob pattern
        if config.input_pattern.contains('*') || config.input_pattern.contains('?') {
            let paths = glob::glob(&config.input_pattern)
                .map_err(|e| anyhow!("Invalid glob pattern: {}", e))?;
            
            let files: Vec<PathBuf> = paths
                .filter_map(|p| p.ok())
                .filter(|p| p.is_file())
                .collect();
            
            if files.is_empty() {
                anyhow::bail!("No files match pattern: {}", config.input_pattern);
            }
            
            return Ok(files);
        }
        
        anyhow::bail!("No files found matching: {}", config.input_pattern)
    }
    
    /// Recursively scan directory for files
    fn scan_directory(dir: &Path, recursive: bool, files: &mut Vec<PathBuf>) -> Result<()> {
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() && recursive {
                Self::scan_directory(&path, recursive, files)?;
            }
        }
        
        Ok(())
    }
    
    /// Generate output path for a given input file
    fn get_output_path(input: &Path, output_dir: &Path) -> PathBuf {
        let stem = input.file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        
        let ext = input.extension()
            .unwrap_or_default()
            .to_string_lossy();
        
        let filename = format!("{}_processed.{}", stem, ext);
        output_dir.join(filename)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_file(dir: &std::path::Path, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_collect_files_single_file() {
        let dir = tempdir().unwrap();
        let test_file = create_test_file(dir.path(), "test.txt", b"content");
        
        let config = BatchConfig {
            input_pattern: test_file.to_str().unwrap().to_string(),
            output_dir: dir.path().to_path_buf(),
            recursive: false,
            continue_on_error: false,
        };
        
        let files = BatchService::collect_files(&config).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], test_file);
    }

    #[test]
    fn test_collect_files_directory_non_recursive() {
        let dir = tempdir().unwrap();
        create_test_file(dir.path(), "file1.txt", b"content1");
        create_test_file(dir.path(), "file2.txt", b"content2");
        
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_file(&subdir, "file3.txt", b"content3");
        
        let config = BatchConfig {
            input_pattern: dir.path().to_str().unwrap().to_string(),
            output_dir: dir.path().to_path_buf(),
            recursive: false,
            continue_on_error: false,
        };
        
        let files = BatchService::collect_files(&config).unwrap();
        // Should only find files in root, not subdirectory
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_collect_files_directory_recursive() {
        let dir = tempdir().unwrap();
        create_test_file(dir.path(), "file1.txt", b"content1");
        
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_file(&subdir, "file2.txt", b"content2");
        
        let config = BatchConfig {
            input_pattern: dir.path().to_str().unwrap().to_string(),
            output_dir: dir.path().to_path_buf(),
            recursive: true,
            continue_on_error: false,
        };
        
        let files = BatchService::collect_files(&config).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_collect_files_glob_pattern() {
        let dir = tempdir().unwrap();
        create_test_file(dir.path(), "file1.txt", b"content1");
        create_test_file(dir.path(), "file2.txt", b"content2");
        create_test_file(dir.path(), "file3.log", b"content3");
        
        let pattern = format!("{}/*.txt", dir.path().display());
        let config = BatchConfig {
            input_pattern: pattern,
            output_dir: dir.path().to_path_buf(),
            recursive: false,
            continue_on_error: false,
        };
        
        let files = BatchService::collect_files(&config).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_collect_files_no_matches() {
        let dir = tempdir().unwrap();
        let pattern = format!("{}/nonexistent*.txt", dir.path().display());
        
        let config = BatchConfig {
            input_pattern: pattern,
            output_dir: dir.path().to_path_buf(),
            recursive: false,
            continue_on_error: false,
        };
        
        let result = BatchService::collect_files(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No files match"));
    }

    #[test]
    fn test_get_output_path() {
        let input = std::path::Path::new("/input/dir/testfile.txt");
        let output_dir = std::path::Path::new("/output/dir");
        
        let output = BatchService::get_output_path(input, output_dir);
        assert_eq!(output, std::path::PathBuf::from("/output/dir/testfile_processed.txt"));
    }

    #[test]
    fn test_get_output_path_no_extension() {
        let input = std::path::Path::new("/input/dir/testfile");
        let output_dir = std::path::Path::new("/output/dir");
        
        let output = BatchService::get_output_path(input, output_dir);
        assert_eq!(output, std::path::PathBuf::from("/output/dir/testfile_processed."));
    }

    #[test]
    fn test_process_batch_all_successful() {
        let dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        
        create_test_file(dir.path(), "file1.txt", b"content1");
        create_test_file(dir.path(), "file2.txt", b"content2");
        
        let config = BatchConfig {
            input_pattern: dir.path().to_str().unwrap().to_string(),
            output_dir: output_dir.path().to_path_buf(),
            recursive: false,
            continue_on_error: false,
        };
        
        let result = BatchService::process_batch(&config, |_input, output| {
            // Simulate successful processing
            let mut file = fs::File::create(output)?;
            file.write_all(b"processed")?;
            Ok(())
        }).unwrap();
        
        assert_eq!(result.total_files, 2);
        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 0);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.output_files.len(), 2);
    }

    #[test]
    fn test_process_batch_with_failures_continue() {
        let dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        
        create_test_file(dir.path(), "file1.txt", b"content1");
        create_test_file(dir.path(), "file2.txt", b"content2");
        
        let config = BatchConfig {
            input_pattern: dir.path().to_str().unwrap().to_string(),
            output_dir: output_dir.path().to_path_buf(),
            recursive: false,
            continue_on_error: true,
        };
        
        let result = BatchService::process_batch(&config, |input, output| {
            if input.to_str().unwrap().contains("file1") {
                Err(anyhow::anyhow!("Simulated failure on file1"))
            } else {
                let mut file = fs::File::create(output)?;
                file.write_all(b"processed")?;
                Ok(())
            }
        }).unwrap();
        
        assert_eq!(result.total_files, 2);
        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].1.contains("Simulated failure"));
    }

    #[test]
    fn test_process_batch_with_failures_stop_on_error() {
        let dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        
        create_test_file(dir.path(), "file1.txt", b"content1");
        create_test_file(dir.path(), "file2.txt", b"content2");
        
        let config = BatchConfig {
            input_pattern: dir.path().to_str().unwrap().to_string(),
            output_dir: output_dir.path().to_path_buf(),
            recursive: false,
            continue_on_error: false,
        };
        
        let result = BatchService::process_batch(&config, |input, _output| {
            Err(anyhow::anyhow!("Simulated failure on {}", input.display()))
        });
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Aborting due to error"));
    }

    #[test]
    fn test_batch_result_debug() {
        let result = BatchResult {
            total_files: 10,
            successful: 8,
            failed: 2,
            errors: vec![
                (std::path::PathBuf::from("file1.txt"), "Error 1".to_string()),
                (std::path::PathBuf::from("file2.txt"), "Error 2".to_string()),
            ],
            output_files: vec![
                std::path::PathBuf::from("output1.txt"),
                std::path::PathBuf::from("output2.txt"),
            ],
        };
        
        // Verify Debug doesn't panic
        let _ = format!("{:?}", result);
    }

    #[test]
    fn test_process_empty_directory() {
        let dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        
        let config = BatchConfig {
            input_pattern: dir.path().to_str().unwrap().to_string(),
            output_dir: output_dir.path().to_path_buf(),
            recursive: true,
            continue_on_error: false,
        };
        
        let result = BatchService::process_batch(&config, |_, _| Ok(()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No files found"));
    }

    #[test]
    fn test_scan_directory_nested() {
        let dir = tempdir().unwrap();
        
        // Create nested directory structure
        let subdir1 = dir.path().join("sub1");
        let subdir2 = dir.path().join("sub2");
        fs::create_dir(&subdir1).unwrap();
        fs::create_dir(&subdir2).unwrap();
        
        create_test_file(&subdir1, "file1.txt", b"content");
        create_test_file(&subdir2, "file2.txt", b"content");
        create_test_file(dir.path(), "root.txt", b"content");
        
        let mut files = Vec::new();
        BatchService::scan_directory(dir.path(), true, &mut files).unwrap();
        
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_scan_directory_non_recursive() {
        let dir = tempdir().unwrap();
        
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_file(&subdir, "file.txt", b"content");
        create_test_file(dir.path(), "root.txt", b"content");
        
        let mut files = Vec::new();
        BatchService::scan_directory(dir.path(), false, &mut files).unwrap();
        
        // Should only get root.txt, not file in subdirectory
        assert_eq!(files.len(), 1);
        assert!(files[0].to_str().unwrap().contains("root.txt"));
    }
}