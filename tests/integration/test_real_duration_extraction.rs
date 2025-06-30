//! # Real Duration Extraction Integration Test
//!
//! ## Purpose
//! Integration test for duration extraction using real Facebook video URLs
//! This test validates that the duration extraction functionality works correctly
//! with actual Facebook videos and returns the expected duration values.
//!
//! ## Category
//! Integration Testing - Real Video Duration Extraction
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_real_duration_extraction --features debug-tools
//! ```
//!
//! ## Dependencies
//! - facebook-extractor-core: Core extraction functionality
//! - reqwest: HTTP client for network requests
//! - tokio: Async runtime
//!
//! ## Setup Requirements
//! - Internet connection for Facebook access
//! - Valid Facebook video URLs for testing

use facebook_extractor_core::FacebookExtractor;

#[tokio::main]
async fn main() {
    println!("🧪 Testing Real Facebook Video Duration Extraction");
    println!("==================================================");
    
    // Initialize the extractor
    let extractor = match FacebookExtractor::new() {
        Ok(extractor) => extractor,
        Err(e) => {
            println!("❌ Failed to initialize FacebookExtractor: {}", e);
            return;
        }
    };
    
    // Test cases with real Facebook URLs and expected durations
    let test_cases = vec![
        (
            "https://www.facebook.com/watch?v=2119954181860033",
            "3:04",
            "First test video (3:04)"
        ),
        (
            "https://www.facebook.com/watch?v=23904368679254369",
            "3:24", 
            "Second test video (3:24)"
        ),
        (
            "https://www.facebook.com/watch?v=1740580203551202",
            "2:08",
            "Third test video (2:08)"
        ),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (url, expected_duration, description) in test_cases {
        println!("\n🔍 Testing: {}", description);
        println!("   URL: {}", url);
        println!("   Expected Duration: {}", expected_duration);
        
        match test_video_duration(&extractor, url, expected_duration).await {
            Ok(actual_duration) => {
                println!("   ✅ PASS - Extracted Duration: {}", actual_duration);
                passed += 1;
            }
            Err(error) => {
                println!("   ❌ FAIL - {}", error);
                failed += 1;
            }
        }
    }
    
    println!("\n📊 Test Results Summary");
    println!("=======================");
    println!("✅ Passed: {}", passed);
    println!("❌ Failed: {}", failed);
    println!("📈 Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);
    
    if failed == 0 {
        println!("\n🎉 All duration extraction tests passed!");
    } else {
        println!("\n⚠️  Some tests failed. This could be due to:");
        println!("   - Network connectivity issues");
        println!("   - Facebook blocking requests");
        println!("   - Changes in Facebook's HTML structure");
        println!("   - Video privacy settings or availability");
    }
}

/// Test duration extraction for a single video
async fn test_video_duration(
    extractor: &FacebookExtractor,
    url: &str,
    expected_duration: &str,
) -> Result<String, String> {
    // Extract video information
    let video_info = match extractor.extract_video_info(url).await {
        Ok(info) => info,
        Err(e) => return Err(format!("Failed to extract video info: {}", e)),
    };
    
    let actual_duration = video_info.duration;
    
    // Check if the duration matches (allowing for slight variations in format)
    if duration_matches(&actual_duration, expected_duration) {
        Ok(actual_duration)
    } else {
        Err(format!(
            "Duration mismatch - Expected: {}, Got: {}",
            expected_duration, actual_duration
        ))
    }
}

/// Check if two duration strings match, allowing for format variations
fn duration_matches(actual: &str, expected: &str) -> bool {
    // Direct match
    if actual == expected {
        return true;
    }
    
    // Extract duration in seconds from both strings for comparison
    let actual_seconds = extract_seconds_from_duration(actual);
    let expected_seconds = extract_seconds_from_duration(expected);
    
    match (actual_seconds, expected_seconds) {
        (Some(actual), Some(expected)) => {
            // Allow for 1-2 second difference due to rounding or precision
            (actual as i32 - expected as i32).abs() <= 2
        }
        _ => false,
    }
}

/// Extract seconds from a duration string (supports various formats)
fn extract_seconds_from_duration(duration: &str) -> Option<u32> {
    // Handle format like "3:04 (184 seconds)"
    if let Some(start) = duration.find('(') {
        if let Some(end) = duration.find(" seconds)") {
            let seconds_str = &duration[start + 1..end];
            return seconds_str.parse().ok();
        }
    }
    
    // Handle format like "3:04"
    if duration.contains(':') {
        let parts: Vec<&str> = duration.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(minutes), Ok(seconds)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                return Some(minutes * 60 + seconds);
            }
        } else if parts.len() == 3 {
            // Handle "h:mm:ss" format
            if let (Ok(hours), Ok(minutes), Ok(seconds)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                return Some(hours * 3600 + minutes * 60 + seconds);
            }
        }
    }
    
    // Handle pure seconds
    if let Ok(seconds) = duration.parse::<u32>() {
        return Some(seconds);
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_matching() {
        // Test exact matches
        assert!(duration_matches("3:04", "3:04"));
        assert!(duration_matches("2:08 (128 seconds)", "2:08"));
        
        // Test format variations
        assert!(duration_matches("3:04 (184 seconds)", "3:04"));
        assert!(duration_matches("184", "3:04"));
        
        // Test tolerance (within 2 seconds)
        assert!(duration_matches("3:04 (184 seconds)", "3:05"));
        assert!(duration_matches("3:04 (184 seconds)", "3:03"));
        
        // Test failures
        assert!(!duration_matches("3:04", "2:08"));
        assert!(!duration_matches("3:04", "3:10"));
    }

    #[test]
    fn test_seconds_extraction() {
        assert_eq!(extract_seconds_from_duration("3:04"), Some(184));
        assert_eq!(extract_seconds_from_duration("2:08"), Some(128));
        assert_eq!(extract_seconds_from_duration("3:24"), Some(204));
        assert_eq!(extract_seconds_from_duration("3:04 (184 seconds)"), Some(184));
        assert_eq!(extract_seconds_from_duration("1:30:45"), Some(5445)); // 1h 30m 45s
        assert_eq!(extract_seconds_from_duration("184"), Some(184));
        assert_eq!(extract_seconds_from_duration("Unknown duration"), None);
    }
}
