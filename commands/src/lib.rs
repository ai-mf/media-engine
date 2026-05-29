// media-engine/commands/src/lib.rs
pub mod traits;
pub mod common;
pub mod create;
pub mod raw;
pub mod json_input;
pub mod info;
pub mod verify;
pub mod extract;
pub mod view;
pub mod sign;
pub mod batch;
pub mod genkey;
pub mod parsers;
pub mod detectors;
pub mod utils;

// Re-export commonly used types
pub use traits::*;
pub use common::*;
pub use create::CreateCommand;
pub use raw::RawCreateCommand;
pub use json_input::JsonCreateCommand;
pub use info::InfoCommand;
pub use verify::VerifyCommand;
pub use extract::ExtractCommand;
pub use view::ViewCommand;
pub use sign::SignCommand;
pub use batch::BatchCommand;
pub use genkey::GenKeyCommand;
pub use parsers::*;
pub use detectors::*;
pub use utils::*;


#[cfg(test)]
mod tests {
    use super::*;
    use traits::ValidationRules;

    #[test]
    fn test_human_bytes() {
        assert_eq!(utils::human_bytes(0), "0 B");
        assert_eq!(utils::human_bytes(500), "500 B");
        assert_eq!(utils::human_bytes(1024), "1.00 KB");
        assert_eq!(utils::human_bytes(1536), "1.50 KB");
        assert_eq!(utils::human_bytes(1_048_576), "1.00 MB");
        assert_eq!(utils::human_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_human_duration() {
        assert_eq!(utils::human_duration(0.5), "500ms");
        assert_eq!(utils::human_duration(30.0), "30.0s");
        assert_eq!(utils::human_duration(90.0), "1m 30s");
        assert_eq!(utils::human_duration(3665.0), "1h 1m");
    }

    #[test]
    fn test_truncate_hex() {
        let hex = "abcdef1234567890";
        // The function keeps the first max_len-3 chars and adds "..."
        assert_eq!(utils::truncate_hex(hex, 10), "abcdef1...");
        assert_eq!(utils::truncate_hex(hex, 20), hex);
        assert_eq!(utils::truncate_hex("abc", 5), "abc");
    }

    #[test]
    fn test_validate_hex() {
        assert!(utils::validate_hex("1234abcd").is_ok());
        assert!(utils::validate_hex("123").is_err()); // odd length
        assert!(utils::validate_hex("123g").is_err()); // invalid char
        assert!(utils::validate_hex("").is_ok()); // empty is valid
    }

    #[test]
    fn test_temp_dir() {
        let temp_dir = utils::TempDir::new("test").unwrap();
        let path = temp_dir.path().clone();
        assert!(path.exists());
        
        // Can create files in temp dir
        let file_path = path.join("test.txt");
        std::fs::write(&file_path, b"test").unwrap();
        assert!(file_path.exists());
        
        // Check that path exists before dropping
        assert!(path.exists());
        
        // When temp_dir drops, it should clean up
        drop(temp_dir);
        assert!(!path.exists());
    }

    #[test]
    fn test_validation_rules_default() {
        let rules = ValidationRules::default();
        assert_eq!(rules.max_dimension, 16384);
        assert_eq!(rules.max_sample_rate, 384_000);
        assert_eq!(rules.max_audio_samples, 100_000_000);
        assert_eq!(rules.max_video_frames, 1_000_000);
        assert_eq!(rules.max_file_size, 10_000_000_000);
    }

    #[test]
    fn test_create_args() {
        use clap::Parser;
        
        #[derive(Parser, Debug)]
        struct TestArgs {
            #[command(flatten)]
            args: common::CreateArgs,
        }
        
        let args = TestArgs::try_parse_from(&[
            "test",
            "--output", "out.png",
            "--model", "TestModel",
            "--version", "1.0",
        ]).unwrap();
        
        assert_eq!(args.args.output.to_str().unwrap(), "out.png");
        assert_eq!(args.args.model, "TestModel");
        assert_eq!(args.args.version, "1.0");
        assert!(args.args.prompt_hash.is_none());
        assert_eq!(args.args.input_format, "auto");
    }

    #[test]
    fn test_info_args() {
        use clap::Parser;
        
        #[derive(Parser, Debug)]
        struct TestArgs {
            #[command(flatten)]
            args: common::InfoArgs,
        }
        
        let args = TestArgs::try_parse_from(&[
            "test",
            "file.png",
            "--detailed",
            "--output-format", "json",
        ]).unwrap();
        
        assert_eq!(args.args.file.to_str().unwrap(), "file.png");
        assert!(args.args.detailed);
        assert_eq!(args.args.output_format, "json");
    }

    #[test]
    fn test_verify_args() {
        use clap::Parser;
        
        #[derive(Parser, Debug)]
        struct TestArgs {
            #[command(flatten)]
            args: common::VerifyArgs,
        }
        
        let args = TestArgs::try_parse_from(&[
            "test",
            "container.aif",
            "--public-key", "key.pub",
            "--quiet",
        ]).unwrap();
        
        assert_eq!(args.args.file.to_str().unwrap(), "container.aif");
        assert_eq!(args.args.public_key.unwrap().to_str().unwrap(), "key.pub");
        assert!(args.args.quiet);
    }

    #[test]
    fn test_extract_args() {
        use clap::Parser;
        
        #[derive(Parser, Debug)]
        struct TestArgs {
            #[command(flatten)]
            args: common::ExtractArgs,
        }
        
        let args = TestArgs::try_parse_from(&[
            "test",
            "input.aif",
            "--output", "output.png",
            "--metadata-only",
        ]).unwrap();
        
        assert_eq!(args.args.file.to_str().unwrap(), "input.aif");
        assert_eq!(args.args.output.to_str().unwrap(), "output.png");
        assert!(args.args.metadata_only);
    }

    #[test]
    fn test_sign_args() {
        use clap::Parser;
        
        #[derive(Parser, Debug)]
        struct TestArgs {
            #[command(flatten)]
            args: common::SignArgs,
        }
        
        let args = TestArgs::try_parse_from(&[
            "test",
            "--input", "input.aif",
            "--key", "private.key",
            "--output", "output.aif",
            "--force",
        ]).unwrap();
        
        assert_eq!(args.args.input.to_str().unwrap(), "input.aif");
        assert_eq!(args.args.key.to_str().unwrap(), "private.key");
        assert_eq!(args.args.output.to_str().unwrap(), "output.aif");
        assert!(args.args.force);
    }

    #[test]
    fn test_batch_args() {
        use clap::Parser;
        
        #[derive(Parser, Debug)]
        struct TestArgs {
            #[command(flatten)]
            args: common::BatchArgs,
        }
        
        let args = TestArgs::try_parse_from(&[
            "test",
            "--input", "*.png",
            "--output-dir", "./output",
            "--parallel",
            "--recursive",
            "--model", "TestModel",
            "--version", "1.0",
            "--max-concurrent", "8",
        ]).unwrap();
        
        assert_eq!(args.args.input, "*.png");
        assert_eq!(args.args.output_dir.to_str().unwrap(), "./output");
        assert!(args.args.parallel);
        assert!(args.args.recursive);
        assert_eq!(args.args.max_concurrent, 8);
        assert_eq!(args.args.processing_args.model, "TestModel");
    }

    #[test]
    fn test_genkey_args() {
        use clap::Parser;
        
        #[derive(Parser, Debug)]
        struct TestArgs {
            #[command(flatten)]
            args: common::GenKeyArgs,
        }
        
        let args = TestArgs::try_parse_from(&[
            "test",
            "--output", "private.key",
            "--with-public",
        ]).unwrap();
        
        assert_eq!(args.args.output.to_str().unwrap(), "private.key");
        assert!(args.args.with_public);
    }

    #[test]
    fn test_global_options() {
        use clap::Parser;
        
        #[derive(Parser, Debug)]
        struct TestArgs {
            #[command(flatten)]
            args: common::GlobalOptions,
        }
        
        let args = TestArgs::try_parse_from(&[
            "test",
            "--verbose",
            "--no-progress",
            "--c2pa",
            "--color", "always",
        ]).unwrap();
        
        assert!(args.args.verbose);
        assert!(args.args.no_progress);
        assert!(args.args.c2pa);
        assert_eq!(args.args.color, "always");
    }

    #[test]
    fn test_media_info_struct() {
        let info = traits::MediaInfo {
            width: Some(1920),
            height: Some(1080),
            sample_rate: None,
            channels: None,
            fps: Some(30),
            duration_secs: Some(10.5),
            format: "mp4".to_string(),
            size_bytes: 1_000_000,
        };
        
        assert_eq!(info.width, Some(1920));
        assert_eq!(info.height, Some(1080));
        assert_eq!(info.fps, Some(30));
        assert_eq!(info.format, "mp4");
    }

    #[test]
    fn test_parsed_media_audio() {
        let audio = traits::AudioData {
            sample_rate: 44100,
            samples: vec![0.0, 0.5, -0.5, 1.0, -1.0],
            channels: 2,
            duration_secs: 0.113,
        };
        
        let media = traits::ParsedMedia::Audio(audio.clone());
        match media {
            traits::ParsedMedia::Audio(a) => {
                assert_eq!(a.sample_rate, 44100);
                assert_eq!(a.samples.len(), 5);
                assert_eq!(a.channels, 2);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_parsed_media_image() {
        let image = traits::ImageData {
            width: 100,
            height: 100,
            pixels: vec![0u8; 30000],
            channels: 3,
        };
        
        let media = traits::ParsedMedia::Image(image.clone());
        match media {
            traits::ParsedMedia::Image(img) => {
                assert_eq!(img.width, 100);
                assert_eq!(img.height, 100);
                assert_eq!(img.channels, 3);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_parsed_media_video() {
        let video = traits::VideoData {
            width: 1920,
            height: 1080,
            fps: 30,
            frames: vec![vec![0u8; 1920*1080*3]; 2],
            audio: None,
            frame_count: 2,
            duration_secs: 0.066,
        };
        
        let media = traits::ParsedMedia::Video(video.clone());
        match media {
            traits::ParsedMedia::Video(v) => {
                assert_eq!(v.width, 1920);
                assert_eq!(v.height, 1080);
                assert_eq!(v.fps, 30);
                assert_eq!(v.frame_count, 2);
            }
            _ => panic!("Wrong variant"),
        }
    }
}