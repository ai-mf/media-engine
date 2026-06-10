// media-engine/commands/src/batch.rs
use crate::traits::*;
use crate::common::*;
use crate::utils::ProgressTracker;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::PathBuf;
use async_trait::async_trait;
use aimf_core::{debug_print};

pub struct BatchCommand;

#[async_trait]
impl CommandExecutor for BatchCommand {
    type Args = BatchArgs;

    async fn execute(args: Self::Args, ctx: &CommandContext) -> Result<()> {
        let progress = ProgressTracker::new(ctx.show_progress, "Collecting files...");

        // Collect input files
        let files = collect_files(&args.input, args.recursive)?;
        
        if files.is_empty() {
            anyhow::bail!("No files found matching pattern: {}", args.input);
        }

        progress.set_message(&format!("Found {} files to process", files.len()));

        // Create output directory
        std::fs::create_dir_all(&args.output_dir)
            .context("Failed to create output directory")?;

        if args.dry_run {
            debug_print!("\n📋 Dry Run - Would process {} files:", files.len());
            for (i, file) in files.iter().enumerate() {
                let output = get_output_path(file, &args.output_dir);
                debug_print!("  {}. {} → {}", i + 1, file.display(), output.display());
            }
            progress.finish_with_message("Dry run complete");
            return Ok(());
        }

        // Process files
        progress.set_message("Processing files...");
        
        let results = if args.parallel {
            process_files_parallel(&files, &args, ctx, &progress)?
        } else {
            process_files_sequential(&files, &args, ctx, &progress)?
        };

        // Print summary
        print_batch_summary(&results, &args);
        progress.finish_with_message("Batch processing complete");

        Ok(())
    }

    fn name() -> &'static str { "batch" }
    fn description() -> &'static str { "Process multiple files in batch mode" }
}

#[derive(Debug)]
#[allow(unused)]
struct BatchResult {
    file: PathBuf,
    output: PathBuf,
    success: bool,
    error: Option<String>,
    size_before: u64,
    size_after: u64,
}

fn collect_files(pattern: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    if pattern.contains('*') || pattern.contains('?') {
        // Glob pattern
        let paths: Vec<PathBuf> = glob::glob(pattern)
            .context("Invalid glob pattern")?
            .filter_map(|p| p.ok())
            .filter(|p| p.is_file())
            .collect();
        Ok(paths)
    } else {
        let path = PathBuf::from(pattern);
        if path.is_file() {
            Ok(vec![path])
        } else if path.is_dir() {
            let mut files = Vec::new();
            collect_dir_files(&path, recursive, &mut files)?;
            Ok(files)
        } else {
            anyhow::bail!("No files found: {}", pattern);
        }
    }
}

fn collect_dir_files(dir: &PathBuf, recursive: bool, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() && recursive {
            collect_dir_files(&path, recursive, files)?;
        }
    }
    Ok(())
}

fn get_output_path(input: &PathBuf, output_dir: &PathBuf) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default();
    let ext = input.extension().unwrap_or_default();
    output_dir.join(format!("{}.{}", stem.to_string_lossy(), ext.to_string_lossy()))
}

fn process_files_parallel(
    files: &[PathBuf],
    args: &BatchArgs,
    ctx: &CommandContext,
    progress: &ProgressTracker,
) -> Result<Vec<BatchResult>> {
    let total = files.len();
    
    let results: Vec<BatchResult> = files
        .par_iter()
        .enumerate()
        .map(|(i, file)| {
            progress.set_progress(i as u64 + 1, total as u64);
            progress.set_message(&format!("Processing: {}", file.file_name().unwrap_or_default().to_string_lossy()));
            
            match process_single_file(file, args, ctx) {
                Ok(result) => result,
                Err(e) => BatchResult {
                    file: file.clone(),
                    output: get_output_path(file, &args.output_dir),
                    success: false,
                    error: Some(e.to_string()),
                    size_before: 0,
                    size_after: 0,
                },
            }
        })
        .collect();

    Ok(results)
}

fn process_files_sequential(
    files: &[PathBuf],
    args: &BatchArgs,
    ctx: &CommandContext,
    progress: &ProgressTracker,
) -> Result<Vec<BatchResult>> {
    let total = files.len();
    let mut results = Vec::with_capacity(total);

    for (i, file) in files.iter().enumerate() {
        progress.set_progress(i as u64 + 1, total as u64);
        progress.set_message(&format!("Processing: {}", file.file_name().unwrap_or_default().to_string_lossy()));

        let result = match process_single_file(file, args, ctx) {
            Ok(r) => r,
            Err(e) => {
                if !args.continue_on_error {
                    return Err(e);
                }
                BatchResult {
                    file: file.clone(),
                    output: get_output_path(file, &args.output_dir),
                    success: false,
                    error: Some(e.to_string()),
                    size_before: 0,
                    size_after: 0,
                }
            }
        };

        results.push(result);
    }

    Ok(results)
}

fn process_single_file(
    file: &PathBuf,
    args: &BatchArgs,
    ctx: &CommandContext,
) -> Result<BatchResult> {
    let size_before = std::fs::metadata(file)?.len();
    
    // Read file
    let data = std::fs::read(file)?;
    
    // Extract container
    let mut container = (ctx.extract_function)(&data)
        .context("Failed to extract AI container")?;
    
    // Update metadata
    container.metadata.model_name = args.processing_args.model.clone();
    container.metadata.model_version = args.processing_args.version.clone();
    
    // Sign if key provided
    if let Some(key_path) = &args.processing_args.key {
        let key_bytes = std::fs::read(key_path)?;
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            &key_bytes[..32].try_into().map_err(|_| anyhow::anyhow!("Invalid key"))?
        );
        container.sign(&signing_key)?;
    }
    
    // Re-embed
    let final_data = (ctx.embed_function)(&data, &container)?;
    
    // Write output
    let output = get_output_path(file, &args.output_dir);
    std::fs::write(&output, &final_data)?;
    
    let size_after = final_data.len() as u64;
    
    Ok(BatchResult {
        file: file.clone(),
        output,
        success: true,
        error: None,
        size_before,
        size_after,
    })
}

fn print_batch_summary(results: &[BatchResult], _args: &BatchArgs) {
    let successful: Vec<_> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();
    
    let total_size_before: u64 = successful.iter().map(|r| r.size_before).sum();
    let total_size_after: u64 = successful.iter().map(|r| r.size_after).sum();
    
    debug_print!("\n📊 Batch Processing Summary");
    debug_print!("═══════════════════════════════════════");
    debug_print!("Total files: {}", results.len());
    debug_print!("✅ Successful: {}", successful.len());
    debug_print!("❌ Failed: {}", failed.len());
    
    if !successful.is_empty() {
        debug_print!("\n📦 Total size: {} → {}", 
            human_bytes(total_size_before as usize),
            human_bytes(total_size_after as usize));
        if total_size_before > 0 {
            let change = ((total_size_after as f64 / total_size_before as f64) - 1.0) * 100.0;
            debug_print!("   Change: {:.1}%", change);
        }
    }
    
    if !failed.is_empty() {
        debug_print!("\n⚠️  Failed files:");
        for result in failed {
            debug_print!("   {}: {}", 
                result.file.display(),
                result.error.as_deref().unwrap_or("Unknown error"));
        }
    }
    
    debug_print!("═══════════════════════════════════════\n");
}

fn human_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", size, UNITS[unit])
}