//! # Duration Integration Test
//!
//! ## Purpose
//! Integration test to verify that duration extraction works correctly
//! with the Tauri application and the provided test URLs.
//!
//! ## Category
//! Integration Testing - Duration Extraction in Tauri App
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_duration_integration --features debug-tools
//! ```
//!
//! ## Dependencies
//! - facebook-extractor-core: Core extraction functionality
//! - tokio: Async runtime
//!
//! ## Setup Requirements
//! - Internet connection for Facebook access
//! - Valid Facebook video URLs for testing

use facebook_extractor_core::FacebookExtractor;

#[tokio::main]
async fn main() {
    println!("🧪 Testing Duration Integration with Tauri Application");
    println!("======================================================");
    
    // Initialize the extractor (same as used in Tauri backend)
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
            184,
            "First test video (3:04)"
        ),
        (
            "https://www.facebook.com/watch?v=23904368679254369",
            "3:24", 
            204,
            "Second test video (3:24)"
        ),
        (
            "https://www.facebook.com/watch?v=1740580203551202",
            "2:08",
            128,
            "Third test video (2:08)"
        ),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (url, expected_time, expected_seconds, description) in test_cases {
        println!("\n🔍 Testing: {}", description);
        println!("   URL: {}", url);
        println!("   Expected Duration: {} ({} seconds)", expected_time, expected_seconds);
        
        match test_video_duration_integration(&extractor, url, expected_time, expected_seconds).await {
            Ok((actual_time, actual_seconds)) => {
                println!("   ✅ PASS");
                println!("   📊 Extracted Duration: {} ({} seconds)", actual_time, actual_seconds);
                passed += 1;
            }
            Err(error) => {
                println!("   ❌ FAIL - {}", error);
                failed += 1;
            }
        }
    }
    
    println!("\n📊 Integration Test Results Summary");
    println!("===================================");
    println!("✅ Passed: {}", passed);
    println!("❌ Failed: {}", failed);
    println!("📈 Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);
    
    if failed == 0 {
        println!("\n🎉 All duration integration tests passed!");
        println!("✅ Duration extraction is working correctly in the Tauri application");
    } else {
        println!("\n⚠️  Some tests failed. This could be due to:");
        println!("   - Network connectivity issues");
        println!("   - Facebook blocking requests");
        println!("   - Changes in Facebook's HTML structure");
        println!("   - Video privacy settings or availability");
    }
    
    // Test the formatting functions used in the frontend
    println!("\n📝 Testing Frontend Duration Formatting");
    println!("=======================================");
    test_frontend_formatting();
}

/// Test duration extraction for a single video (integration test)
async fn test_video_duration_integration(
    extractor: &FacebookExtractor,
    url: &str,
    expected_time: &str,
    expected_seconds: u32,
) -> Result<(String, u32), String> {
    // Extract video information (same as Tauri backend)
    let video_info = match extractor.extract_video_info(url).await {
        Ok(info) => info,
        Err(e) => return Err(format!("Failed to extract video info: {}", e)),
    };
    
    // Check the duration string format (e.g., "3:04 (184 seconds)")
    let duration_string = &video_info.duration;
    
    // Check the metadata duration_seconds
    let metadata_seconds = video_info.metadata.duration_seconds;
    
    println!("   📋 Raw duration string: {}", duration_string);
    println!("   📋 Metadata duration_seconds: {:?}", metadata_seconds);
    
    // Verify that we have duration information
    if duration_string == "Unknown duration" && metadata_seconds.is_none() {
        return Err("No duration information extracted".to_string());
    }
    
    // Extract time format from duration string
    let time_format = if let Some(seconds) = metadata_seconds {
        format_duration_time(seconds)
    } else {
        extract_time_format_from_string(duration_string)
    };
    
    // Get actual seconds
    let actual_seconds = metadata_seconds.unwrap_or_else(|| {
        parse_seconds_from_duration_string(duration_string).unwrap_or(0)
    });
    
    // Check if the duration matches (allowing for slight variations)
    let time_matches = time_format == expected_time || 
                      duration_matches_flexible(&time_format, expected_time);
    
    let seconds_matches = (actual_seconds as i32 - expected_seconds as i32).abs() <= 2;
    
    if time_matches && seconds_matches {
        Ok((time_format, actual_seconds))
    } else {
        Err(format!(
            "Duration mismatch - Expected: {} ({}s), Got: {} ({}s)",
            expected_time, expected_seconds, time_format, actual_seconds
        ))
    }
}

/// Format duration in seconds to MM:SS or H:MM:SS format
fn format_duration_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

/// Extract time format from duration string
fn extract_time_format_from_string(duration_str: &str) -> String {
    // Handle format like "3:04 (184 seconds)"
    if let Some(space_pos) = duration_str.find(' ') {
        let time_part = &duration_str[..space_pos];
        if time_part.contains(':') {
            return time_part.to_string();
        }
    }
    
    // If it's already in MM:SS format
    if duration_str.contains(':') && !duration_str.contains('(') {
        return duration_str.to_string();
    }
    
    "Unknown".to_string()
}

/// Parse seconds from duration string
fn parse_seconds_from_duration_string(duration_str: &str) -> Option<u32> {
    // Handle format like "3:04 (184 seconds)"
    if let Some(start) = duration_str.find('(') {
        if let Some(end) = duration_str.find(" seconds)") {
            let seconds_str = &duration_str[start + 1..end];
            return seconds_str.parse().ok();
        }
    }
    None
}

/// Check if two duration strings match with some flexibility
fn duration_matches_flexible(actual: &str, expected: &str) -> bool {
    // Direct match
    if actual == expected {
        return true;
    }
    
    // Parse both to seconds and compare
    if let (Some(actual_secs), Some(expected_secs)) = (
        parse_time_to_seconds(actual),
        parse_time_to_seconds(expected)
    ) {
        (actual_secs as i32 - expected_secs as i32).abs() <= 2
    } else {
        false
    }
}

/// Parse MM:SS or H:MM:SS format to seconds
fn parse_time_to_seconds(time_str: &str) -> Option<u32> {
    let parts: Vec<&str> = time_str.split(':').collect();
    match parts.len() {
        2 => {
            // MM:SS format
            if let (Ok(minutes), Ok(seconds)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                Some(minutes * 60 + seconds)
            } else {
                None
            }
        }
        3 => {
            // H:MM:SS format
            if let (Ok(hours), Ok(minutes), Ok(seconds)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                Some(hours * 3600 + minutes * 60 + seconds)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Test the frontend formatting functions
fn test_frontend_formatting() {
    let test_cases = vec![
        (184, "3:04"),
        (204, "3:24"),
        (128, "2:08"),
        (30, "0:30"),
        (3661, "1:01:01"),
    ];
    
    for (seconds, expected) in test_cases {
        let formatted = format_duration_time(seconds);
        if formatted == expected {
            println!("   ✅ {} seconds -> {} ✓", seconds, formatted);
        } else {
            println!("   ❌ {} seconds -> {} (expected {})", seconds, formatted, expected);
        }
    }
}
