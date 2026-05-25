// In tests/aimf_tests.rs
//cargo test
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invalid_image_json() {
        let bad_input = br#"{"width":"abc","height":600,"pixels":[255]}"#;
        let result = parse_json_image(bad_input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("width must be"));
    }
    
    #[test]
    fn test_memory_bomb_prevention() {
        let huge = br#"{"width":100000,"height":100000,"pixels":[]}"#;
        let result = parse_json_image(huge);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Dimensions too large"));
    }
}