//! Unit tests for duration extraction functionality
//! 
//! These tests can be run with `cargo test` and are integrated into the standard Rust testing framework.

#[cfg(test)]
mod duration_extraction_tests {
    use crate::metadata::MetadataExtractor;

    #[test]
    fn test_extract_duration_from_html_basic_patterns() {
        let extractor = MetadataExtractor::new();
        
        // Test all supported duration patterns
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"duration_s":123}"#),
            "2:03 (123 seconds)"
        );
        
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"duration":75}"#),
            "1:15 (75 seconds)"
        );
        
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"length_seconds":360}"#),
            "6:00 (360 seconds)"
        );
        
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"video_duration":9}"#),
            "0:09 (9 seconds)"
        );
    }

    #[test]
    fn test_extract_duration_from_html_fallback_pattern() {
        let extractor = MetadataExtractor::new();
        
        // Test fallback 't' pattern with valid range
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"t":180}"#),
            "3:00 (180 seconds)"
        );
        
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"t":5}"#),
            "0:05 (5 seconds)"
        );
        
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"t":600}"#),
            "10:00 (600 seconds)"
        );
    }

    #[test]
    fn test_extract_duration_from_html_invalid_cases() {
        let extractor = MetadataExtractor::new();
        
        // Test cases that should return "Unknown duration"
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"no_duration":true}"#),
            "Unknown duration"
        );
        
        assert_eq!(
            extractor.extract_duration_from_html(""),
            "Unknown duration"
        );
        
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"duration_s":"invalid"}"#),
            "Unknown duration"
        );
        
        // Fallback pattern outside valid range
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"t":4}"#),
            "Unknown duration"
        );
        
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"t":601}"#),
            "Unknown duration"
        );
    }

    #[test]
    fn test_extract_duration_seconds_from_metadata() {
        let extractor = MetadataExtractor::new();
        
        // Test duration seconds extraction through metadata
        let metadata1 = extractor.extract_video_metadata(r#"{"duration_s":123}"#);
        assert_eq!(metadata1.duration_seconds, Some(123));
        
        let metadata2 = extractor.extract_video_metadata(r#"{"duration":75}"#);
        assert_eq!(metadata2.duration_seconds, Some(75));
        
        let metadata3 = extractor.extract_video_metadata(r#"{"no_duration":true}"#);
        assert_eq!(metadata3.duration_seconds, None);
        
        let metadata4 = extractor.extract_video_metadata(r#"{"t":300}"#);
        assert_eq!(metadata4.duration_seconds, Some(300));
    }

    #[test]
    fn test_duration_format_outputs() {
        let extractor = MetadataExtractor::new();
        
        // Test various duration format outputs
        let test_cases = vec![
            (1, "0:01 (1 seconds)"),
            (59, "0:59 (59 seconds)"),
            (60, "1:00 (60 seconds)"),
            (61, "1:01 (61 seconds)"),
            (3600, "60:00 (3600 seconds)"),
            (3661, "61:01 (3661 seconds)"),
            (7200, "120:00 (7200 seconds)"),
        ];
        
        for (seconds, expected) in test_cases {
            let html = format!(r#"{{"duration_s":{}}}"#, seconds);
            let result = extractor.extract_duration_from_html(&html);
            assert_eq!(result, expected, "Failed for {} seconds", seconds);
        }
    }

    #[test]
    fn test_multiple_duration_patterns() {
        let extractor = MetadataExtractor::new();
        
        // Test that first matching pattern is used
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"duration":45, "length_seconds":99}"#),
            "0:45 (45 seconds)"
        );
        
        // duration_s has priority over video_duration
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"video_duration":30, "duration_s":120}"#),
            "2:00 (120 seconds)"
        );
    }

    #[test]
    fn test_edge_cases() {
        let extractor = MetadataExtractor::new();
        
        // Zero duration
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"duration_s":0}"#),
            "0:00 (0 seconds)"
        );
        
        // Very large duration
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"duration_s":999999}"#),
            "16666:39 (999999 seconds)"
        );
        
        // Malformed JSON (regex still matches)
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"duration_s":123"#),
            "2:03 (123 seconds)"
        );
        
        // Nested JSON (regex can match)
        assert_eq!(
            extractor.extract_duration_from_html(r#"{"video":{"duration_s":90}}"#),
            "1:30 (90 seconds)"
        );
    }

    #[test]
    fn test_realistic_facebook_scenarios() {
        let extractor = MetadataExtractor::new();
        
        // Realistic Facebook JSON structure
        assert_eq!(
            extractor.extract_duration_from_html(
                r#"{"videoData":{"duration_s":142,"title":"Sample Video"},"otherData":{}}"#
            ),
            "2:22 (142 seconds)"
        );
        
        // Facebook page with embedded JSON
        assert_eq!(
            extractor.extract_duration_from_html(
                r#"<html><head><script>window.__INITIAL_DATA__={"video":{"duration_s":95}}</script></head></html>"#
            ),
            "1:35 (95 seconds)"
        );
        
        // Mixed duration units (seconds preferred)
        assert_eq!(
            extractor.extract_duration_from_html(
                r#"{"duration_minutes":5,"duration_s":300,"duration_hours":1}"#
            ),
            "5:00 (300 seconds)"
        );
    }

    #[tokio::test]
    async fn test_mp4_duration_probing_invalid_urls() {
        let extractor = MetadataExtractor::new();
        
        // Test with invalid URLs (should return None)
        let invalid_urls = vec![
            "https://invalid-url.com/video.mp4",
            "not-a-url",
            "",
        ];
        
        for url in invalid_urls {
            let result = extractor.probe_duration_from_video_url(url).await;
            assert!(result.is_none(), "Expected None for invalid URL: {}", url);
        }
    }

    #[test]
    fn test_boundary_conditions() {
        let extractor = MetadataExtractor::new();
        
        // Test boundary conditions for fallback pattern
        let metadata_min = extractor.extract_video_metadata(r#"{"t":5}"#);
        assert_eq!(metadata_min.duration_seconds, Some(5));
        
        let metadata_max = extractor.extract_video_metadata(r#"{"t":600}"#);
        assert_eq!(metadata_max.duration_seconds, Some(600));
        
        let metadata_below_min = extractor.extract_video_metadata(r#"{"t":4}"#);
        assert_eq!(metadata_below_min.duration_seconds, None);
        
        let metadata_above_max = extractor.extract_video_metadata(r#"{"t":601}"#);
        assert_eq!(metadata_above_max.duration_seconds, None);
    }

    #[test]
    fn test_comprehensive_metadata_with_duration() {
        let extractor = MetadataExtractor::new();
        
        let html = r#"{"duration_s":125,"title":"Test Video","author":"Test Author"}"#;
        let metadata = extractor.extract_video_metadata(html);
        
        assert_eq!(metadata.duration_seconds, Some(125));
        // Note: Other metadata fields may not be extracted from this simple JSON
        // as they require more complex HTML patterns
    }
}
