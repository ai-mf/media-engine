// media-engine/commands/src/utils.rs
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use std::path::PathBuf;

/// Progress tracker for CLI operations
pub struct ProgressTracker {
    bar: Option<ProgressBar>,
}

impl ProgressTracker {
    pub fn new(enabled: bool, message: &str) -> Self {
        let bar = if enabled {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template("{spinner:.green} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            );
            pb.set_message(message.to_string());
            pb.enable_steady_tick(Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };
        Self { bar }
    }

    pub fn set_message(&self, msg: &str) {
        if let Some(ref bar) = self.bar {
            bar.set_message(msg.to_string());
        }
    }

    pub fn set_progress(&self, current: u64, total: u64) {
        if let Some(ref bar) = self.bar {
            if bar.length().is_none() {
                bar.set_length(total);
                bar.set_style(
                    ProgressStyle::with_template(
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})"
                    )
                    .unwrap()
                    .progress_chars("#>-")
                );
            }
            bar.set_position(current);
        }
    }

    pub fn finish_with_message(&self, msg: &str) {
        if let Some(ref bar) = self.bar {
            bar.finish_with_message(msg.to_string());
        }
    }

    pub fn finish_with_error(&self, msg: &str) {
        if let Some(ref bar) = self.bar {
            bar.finish_with_message(format!("❌ {}", msg));
        }
    }
}

impl Drop for ProgressTracker {
    fn drop(&mut self) {
        if let Some(ref bar) = self.bar {
            if !bar.is_finished() {
                bar.finish_and_clear();
            }
        }
    }
}

/// Format bytes to human readable string
pub fn human_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.2} {}", size, UNITS[unit])
    }
}

/// Format duration in seconds to human readable string
pub fn human_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.1}s", seconds)
    } else if seconds < 3600.0 {
        let mins = (seconds / 60.0) as u64;
        let secs = (seconds % 60.0) as u64;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = (seconds / 3600.0) as u64;
        let mins = ((seconds % 3600.0) / 60.0) as u64;
        format!("{}h {}m", hours, mins)
    }
}

/// Truncate a hex string for display
pub fn truncate_hex(hex_str: &str, max_len: usize) -> String {
    if hex_str.len() <= max_len {
        hex_str.to_string()
    } else {
        format!("{}...", &hex_str[..max_len - 3])
    }
}

/// Validate hex string
pub fn validate_hex(hex_str: &str) -> Result<(), String> {
    if hex_str.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid hex characters".to_string());
    }
    Ok(())
}

/// Create a temporary directory that's cleaned up on drop
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str) -> anyhow::Result<Self> {
        let path = std::env::temp_dir().join(format!("{}_{}", prefix, std::process::id()));
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}