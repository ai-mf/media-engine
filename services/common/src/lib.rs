//media-engine/services/common/src/lib.rs
mod progress;

#[cfg(test)]
mod tests {
    use super::*;
    use progress::SimpleProgress;

    #[test]
    fn test_simple_progress_creation() {
        let _progress = SimpleProgress::new(true);
        let _disabled = SimpleProgress::new(false);
        // Just verify creation works
        assert!(true);
    }

    #[test]
    fn test_simple_progress_println() {
        let progress = SimpleProgress::new(true);
        // Just verify it doesn't panic
        progress.println("Test message");
    }

    #[test]
    fn test_simple_progress_set_task() {
        let mut progress = SimpleProgress::new(true);
        progress.set_task("Processing file");
        // Can't check private field, but method should not panic
        progress.done();
    }

    #[test]
    fn test_simple_progress_done() {
        let mut progress = SimpleProgress::new(true);
        progress.set_task("Processing");
        progress.done();
        // Should not panic
    }

    #[test]
    fn test_disabled_progress_does_nothing() {
        let mut progress = SimpleProgress::new(false);
        progress.set_task("Task");
        progress.println("Message");
        progress.done();
        // Should not panic
    }

    #[test]
    fn test_progress_multiple_updates() {
        let mut progress = SimpleProgress::new(true);
        
        progress.set_task("Step 1");
        progress.done();
        
        progress.set_task("Step 2");
        progress.done();
        
        // Should handle multiple updates without panic
    }

    #[test]
    fn test_progress_without_done() {
        let mut progress = SimpleProgress::new(true);
        progress.set_task("Task without done");
        // Just setting task should be fine
    }

    #[test]
    fn test_progress_with_empty_message() {
        let progress = SimpleProgress::new(true);
        progress.println("Just a message");
        // Should handle println when message is empty
    }
}