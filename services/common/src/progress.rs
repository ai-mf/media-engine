// media-engine/services/common/src/progress.rs
use std::io::{self, Write};

/// Simple progress reporter - doesn't break anything
pub struct SimpleProgress {
    enabled: bool,
    message: String,
}

#[allow(unused)]
impl SimpleProgress {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            message: String::new(),
        }
    }
    
    pub fn println(&self, msg: &str) {
        if self.enabled {
            eprintln!("{} {}", if self.message.is_empty() { "→" } else { "  " }, msg);
        }
    }
    
    pub fn set_task(&mut self, task: &str) {
        self.message = task.to_string();
        if self.enabled {
            eprint!("\r⏳ {}...", task);
            io::stderr().flush().ok();
        }
    }
    
    pub fn done(&self) {
        if self.enabled {
            eprintln!("\r✅ {}", self.message);
        }
    }
}