//! # Duration Extraction Test
//!
//! ## Purpose
//! Comprehensive testing of video duration extraction functionality from Facebook videos
//!
//! ## Category
//! Content Parsing - Duration Extraction
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_duration_extraction
//! ```
//!
//! ## Dependencies
//! - facebook-extractor-core: Core extraction functionality
//! - tokio: Async runtime for MP4 probing tests
//!
//! ## Setup Requirements
//! - No external dependencies for HTML parsing tests
//! - Internet connection required for MP4 probing tests (optional)

use facebook_extractor_core::metadata::MetadataExtractor;

#[tokio::main]
async fn main() {
    println!("🧪 Testing Duration Extraction Functionality");
    println!("============================================");
    
    let extractor = MetadataExtractor::new();
    
    // Test HTML duration extraction with various formats
    test_html_duration_extraction(&extractor);
    
    // Test duration seconds extraction
    test_duration_seconds_extraction(&extractor);
    
    // Test edge cases and error handling
    test_edge_cases(&extractor);
    
    // Test different duration formats and ranges
    test_duration_formats(&extractor);
    
    // Test realistic Facebook HTML scenarios
    test_realistic_facebook_scenarios(&extractor);

    // Test MP4 duration probing (if network available)
    test_mp4_duration_probing(&extractor).await;

    // Test duration parsing with real video examples
    test_real_video_duration_parsing(&extractor);

    println!("\n✅ All duration extraction tests completed!");
}

/// Test HTML duration extraction with various input patterns
fn test_html_duration_extraction(extractor: &MetadataExtractor) {
    println!("\n📝 Testing HTML Duration Extraction");
    println!("===================================");
    
    let test_cases = vec![
        // Standard duration patterns
        (r#"{"duration_s":123}"#, "2:03 (123 seconds)", "duration_s pattern"),
        (r#"{"duration":75}"#, "1:15 (75 seconds)", "duration pattern"),
        (r#"{"length_seconds":360}"#, "6:00 (360 seconds)", "length_seconds pattern"),
        (r#"{"video_duration":9}"#, "0:09 (9 seconds)", "video_duration pattern"),
        
        // Different duration ranges
        (r#"{"duration_s":5}"#, "0:05 (5 seconds)", "minimum duration (5s)"),
        (r#"{"duration_s":59}"#, "0:59 (59 seconds)", "under 1 minute"),
        (r#"{"duration_s":60}"#, "1:00 (60 seconds)", "exactly 1 minute"),
        (r#"{"duration_s":3661}"#, "61:01 (3661 seconds)", "over 1 hour"),
        (r#"{"duration_s":7200}"#, "120:00 (7200 seconds)", "exactly 2 hours"),
        
        // Multiple patterns (should match first pattern found in order)
        (r#"{"duration":45, "length_seconds":99}"#, "0:45 (45 seconds)", "multiple patterns"),
        (r#"{"video_duration":30, "duration_s":120}"#, "2:00 (120 seconds)", "multiple patterns priority (duration_s first)"),
        
        // Fallback pattern
        (r#"{"t":180}"#, "3:00 (180 seconds)", "fallback 't' pattern"),
        (r#"{"t":300}"#, "5:00 (300 seconds)", "fallback 't' pattern valid range"),
        
        // No match cases
        (r#"{"no_duration":true}"#, "Unknown duration", "no duration found"),
        (r#"{"other_field":123}"#, "Unknown duration", "unrelated fields"),
        ("", "Unknown duration", "empty HTML"),
    ];
    
    for (html, expected, description) in test_cases {
        println!("\n🔍 Test: {}", description);
        println!("   Input: {}", html);
        
        let result = extractor.extract_duration_from_html(html);
        println!("   Result: {}", result);
        
        if result == expected {
            println!("   ✅ PASS");
        } else {
            println!("   ❌ FAIL - Expected: {}", expected);
        }
    }
}

/// Test duration seconds extraction (returns Option<u32>)
fn test_duration_seconds_extraction(extractor: &MetadataExtractor) {
    println!("\n📝 Testing Duration Seconds Extraction");
    println!("======================================");
    
    let test_cases = vec![
        // Valid duration patterns
        (r#"{"duration_s":123}"#, Some(123), "duration_s pattern"),
        (r#"{"duration":75}"#, Some(75), "duration pattern"),
        (r#"{"length_seconds":360}"#, Some(360), "length_seconds pattern"),
        (r#"{"video_duration":9}"#, Some(9), "video_duration pattern"),
        
        // Boundary conditions for fallback pattern
        (r#"{"t":5}"#, Some(5), "fallback minimum (5s)"),
        (r#"{"t":600}"#, Some(600), "fallback maximum (600s)"),
        (r#"{"t":4}"#, None, "fallback below minimum"),
        (r#"{"t":601}"#, None, "fallback above maximum"),
        
        // Invalid cases
        (r#"{"duration_s":"invalid"}"#, None, "non-numeric duration"),
        (r#"{"no_duration":true}"#, None, "no duration field"),
        ("", None, "empty HTML"),
        (r#"{"duration_s":-5}"#, None, "negative duration"),
    ];
    
    for (html, expected, description) in test_cases {
        println!("\n🔍 Test: {}", description);
        println!("   Input: {}", html);
        
        // We need to test the private method through the public metadata extraction
        let metadata = extractor.extract_video_metadata(html);
        let result = metadata.duration_seconds;
        
        println!("   Result: {:?}", result);
        
        if result == expected {
            println!("   ✅ PASS");
        } else {
            println!("   ❌ FAIL - Expected: {:?}", expected);
        }
    }
}

/// Test edge cases and error handling
fn test_edge_cases(extractor: &MetadataExtractor) {
    println!("\n📝 Testing Edge Cases");
    println!("=====================");
    
    let edge_cases = vec![
        // Malformed JSON (regex still matches the pattern)
        (r#"{"duration_s":123"#, "2:03 (123 seconds)", "malformed JSON (regex still matches)"),
        (r#"duration_s":123}"#, "2:03 (123 seconds)", "incomplete JSON (regex still matches)"),

        // Very large numbers
        (r#"{"duration_s":999999}"#, "16666:39 (999999 seconds)", "very large duration"),
        (r#"{"duration_s":0}"#, "0:00 (0 seconds)", "zero duration"),

        // Multiple occurrences
        (r#"{"duration_s":60} and {"duration_s":120}"#, "1:00 (60 seconds)", "multiple occurrences"),

        // Nested JSON (regex can still match)
        (r#"{"video":{"duration_s":90}}"#, "1:30 (90 seconds)", "nested duration (regex matches)"),

        // Mixed with other content
        (r#"<html><script>{"duration_s":45}</script></html>"#, "0:45 (45 seconds)", "duration in HTML script"),

        // Unicode and special characters
        (r#"{"title":"测试","duration_s":30}"#, "0:30 (30 seconds)", "duration with unicode content"),

        // Truly invalid cases
        (r#"{"duration_s":"not_a_number"}"#, "Unknown duration", "non-numeric string"),
        (r#"{"duration_s":null}"#, "Unknown duration", "null value"),
        (r#"no json here at all"#, "Unknown duration", "no JSON structure"),
    ];
    
    for (html, expected, description) in edge_cases {
        println!("\n🔍 Test: {}", description);
        println!("   Input: {}", if html.len() > 50 { &html[..50] } else { html });
        
        let result = extractor.extract_duration_from_html(html);
        println!("   Result: {}", result);
        
        if result == expected {
            println!("   ✅ PASS");
        } else {
            println!("   ❌ FAIL - Expected: {}", expected);
        }
    }
}

/// Test different duration format outputs
fn test_duration_formats(extractor: &MetadataExtractor) {
    println!("\n📝 Testing Duration Format Outputs");
    println!("==================================");
    
    let format_tests = vec![
        // Test various time formats
        (1, "0:01 (1 seconds)", "1 second"),
        (10, "0:10 (10 seconds)", "10 seconds"),
        (60, "1:00 (60 seconds)", "1 minute"),
        (61, "1:01 (61 seconds)", "1 minute 1 second"),
        (119, "1:59 (119 seconds)", "1 minute 59 seconds"),
        (120, "2:00 (120 seconds)", "2 minutes"),
        (3600, "60:00 (3600 seconds)", "1 hour (displayed as 60 minutes)"),
        (3661, "61:01 (3661 seconds)", "1 hour 1 minute 1 second"),
        (7200, "120:00 (7200 seconds)", "2 hours"),
        (7323, "122:03 (7323 seconds)", "2 hours 2 minutes 3 seconds"),
    ];
    
    for (seconds, expected, description) in format_tests {
        println!("\n🔍 Test: {}", description);
        
        let html = format!(r#"{{"duration_s":{}}}"#, seconds);
        let result = extractor.extract_duration_from_html(&html);
        
        println!("   Input: {} seconds", seconds);
        println!("   Result: {}", result);
        
        if result == expected {
            println!("   ✅ PASS");
        } else {
            println!("   ❌ FAIL - Expected: {}", expected);
        }
    }
}

/// Test MP4 duration probing functionality
async fn test_mp4_duration_probing(extractor: &MetadataExtractor) {
    println!("\n📝 Testing MP4 Duration Probing");
    println!("===============================");
    
    // Test with invalid URLs (should return None)
    let invalid_urls = vec![
        "https://invalid-url.com/video.mp4",
        "not-a-url",
        "",
        "https://httpbin.org/status/404", // Valid URL but not MP4
    ];
    
    for url in invalid_urls {
        println!("\n🔍 Testing invalid URL: {}", if url.is_empty() { "(empty)" } else { url });
        
        let result = extractor.probe_duration_from_video_url(url).await;
        println!("   Result: {:?}", result);
        
        if result.is_none() {
            println!("   ✅ PASS - Correctly returned None for invalid URL");
        } else {
            println!("   ❌ FAIL - Expected None for invalid URL");
        }
    }
    
    println!("\n📝 Note: Testing with real MP4 URLs requires network access");
    println!("     and valid video files. This test focuses on error handling.");
    println!("     For integration testing with real videos, use separate");
    println!("     integration tests with known good MP4 URLs.");
}

/// Test realistic Facebook HTML scenarios
fn test_realistic_facebook_scenarios(extractor: &MetadataExtractor) {
    println!("\n📝 Testing Realistic Facebook HTML Scenarios");
    println!("============================================");

    let realistic_cases = vec![
        // Realistic Facebook JSON structure
        (
            r#"{"videoData":{"duration_s":142,"title":"Sample Video"},"otherData":{}}"#,
            "2:22 (142 seconds)",
            "realistic Facebook JSON structure"
        ),

        // Multiple video objects (should match first)
        (
            r#"{"videos":[{"duration_s":60},{"duration_s":120}]}"#,
            "1:00 (60 seconds)",
            "multiple video objects"
        ),

        // Mixed duration fields in complex JSON
        (
            r#"{"video":{"metadata":{"duration":180},"streams":[{"length_seconds":240}]}}"#,
            "3:00 (180 seconds)",
            "complex nested JSON with multiple duration fields"
        ),

        // Real-world Facebook page structure simulation
        (
            r#"<html><head><script>window.__INITIAL_DATA__={"video":{"duration_s":95}}</script></head></html>"#,
            "1:35 (95 seconds)",
            "Facebook page with embedded JSON"
        ),

        // Duration in different units (should only match seconds)
        (
            r#"{"duration_minutes":5,"duration_s":300,"duration_hours":1}"#,
            "5:00 (300 seconds)",
            "mixed duration units (seconds preferred)"
        ),

        // Very short videos (common on social media)
        (
            r#"{"duration_s":3}"#,
            "0:03 (3 seconds)",
            "very short video (3 seconds)"
        ),

        // Long-form content
        (
            r#"{"duration_s":3900}"#,
            "65:00 (3900 seconds)",
            "long-form content (65 minutes)"
        ),

        // Live stream or unknown duration indicators
        (
            r#"{"is_live":true,"duration_s":0}"#,
            "0:00 (0 seconds)",
            "live stream with zero duration"
        ),
    ];

    for (html, expected, description) in realistic_cases {
        println!("\n🔍 Test: {}", description);
        println!("   Input: {}", if html.len() > 80 { &html[..80] } else { html });

        let result = extractor.extract_duration_from_html(html);
        println!("   Result: {}", result);

        if result == expected {
            println!("   ✅ PASS");
        } else {
            println!("   ❌ FAIL - Expected: {}", expected);
        }
    }

    // Test comprehensive metadata extraction with duration
    println!("\n📝 Testing Comprehensive Metadata with Duration");
    println!("===============================================");

    let metadata_html = r#"{"video":{"title":"Test Video","duration_s":125,"author":"Test Author","views":1000,"likes":50}}"#;

    println!("\n🔍 Testing comprehensive metadata extraction");
    let metadata = extractor.extract_video_metadata(metadata_html);

    println!("   Duration (seconds): {:?}", metadata.duration_seconds);
    println!("   Author: {}", metadata.author);
    println!("   Views: {}", metadata.views);

    if metadata.duration_seconds == Some(125) {
        println!("   ✅ PASS - Duration correctly extracted in metadata");
    } else {
        println!("   ❌ FAIL - Duration not correctly extracted in metadata");
    }
}

/// Test duration parsing with real video examples
fn test_real_video_duration_parsing(extractor: &MetadataExtractor) {
    println!("\n📝 Testing Real Video Duration Examples");
    println!("=======================================");

    // Test cases based on real Facebook video durations
    let real_video_cases = vec![
        // Real video examples with expected durations in seconds
        (184, "3:04 (184 seconds)", "3:04 video"),
        (204, "3:24 (204 seconds)", "3:24 video"),
        (128, "2:08 (128 seconds)", "2:08 video"),

        // Additional realistic durations
        (30, "0:30 (30 seconds)", "30-second video"),
        (90, "1:30 (90 seconds)", "1.5-minute video"),
        (300, "5:00 (300 seconds)", "5-minute video"),
        (600, "10:00 (600 seconds)", "10-minute video"),
        (900, "15:00 (900 seconds)", "15-minute video"),
    ];

    for (seconds, expected_format, description) in real_video_cases {
        println!("\n🔍 Test: {}", description);

        let html = format!(r#"{{"duration_s":{}}}"#, seconds);
        let result = extractor.extract_duration_from_html(&html);

        println!("   Input: {} seconds", seconds);
        println!("   Result: {}", result);

        if result == expected_format {
            println!("   ✅ PASS");
        } else {
            println!("   ❌ FAIL - Expected: {}", expected_format);
        }

        // Also test the metadata extraction
        let metadata = extractor.extract_video_metadata(&html);
        if metadata.duration_seconds == Some(seconds) {
            println!("   ✅ Metadata extraction: PASS");
        } else {
            println!("   ❌ Metadata extraction: FAIL - Expected: Some({}), Got: {:?}",
                     seconds, metadata.duration_seconds);
        }
    }

    // Test duration format validation
    println!("\n📝 Testing Duration Format Validation");
    println!("=====================================");

    let format_tests = vec![
        (184, "3:04"), // 3 minutes 4 seconds
        (204, "3:24"), // 3 minutes 24 seconds
        (128, "2:08"), // 2 minutes 8 seconds
    ];

    for (seconds, expected_mm_ss) in format_tests {
        let html = format!(r#"{{"duration_s":{}}}"#, seconds);
        let result = extractor.extract_duration_from_html(&html);

        // Check if the result contains the expected MM:SS format
        if result.contains(expected_mm_ss) {
            println!("   ✅ {} seconds -> contains '{}' ✓", seconds, expected_mm_ss);
        } else {
            println!("   ❌ {} seconds -> missing '{}' in '{}'", seconds, expected_mm_ss, result);
        }
    }
}
