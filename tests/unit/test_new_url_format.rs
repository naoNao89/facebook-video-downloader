//! # New URL Format Test
//!
//! ## Purpose
//! Tests support for new Facebook URL formats and patterns
//!
//! ## Category
//! URL Validation
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_new_url_format
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

use regex::Regex;

fn is_valid_facebook_url(url: &str) -> bool {
    let patterns = [
        r"^https?://(?:www\.)?facebook\.com/watch\?.*v=\d+",  // Fixed: removed /? and made v= more flexible
        r"^https?://(?:www\.)?facebook\.com/[^/]+/videos/\d+",
        r"^https?://fb\.watch/[a-zA-Z0-9_-]+",
        r"^https?://(?:www\.)?facebook\.com/video\.php\?v=\d+",
        r"^https?://(?:www\.)?facebook\.com/[^/]+/posts/\d+",
        r"^https?://(?:www\.)?facebook\.com/reel/\d+",  // Added reel support
        r"^https?://(?:www\.)?facebook\.com/share/v/[a-zA-Z0-9_-]+/?",  // New sharing format
        r"^https?://(?:www\.)?facebook\.com/share/r/[a-zA-Z0-9_-]+/?",  // New sharing format for reels
    ];

    patterns.iter().any(|pattern| {
        if let Ok(regex) = Regex::new(pattern) {
            regex.is_match(url)
        } else {
            false
        }
    })
}

fn extract_video_id(url: &str) -> Option<String> {
    // Try to extract video ID from various Facebook URL formats
    let patterns = [
        (r"v=(\d+)", 1),
        (r"videos/(\d+)", 1),
        (r"fb\.watch/([a-zA-Z0-9_-]+)", 1),
        (r"posts/(\d+)", 1),
        (r"share/v/([a-zA-Z0-9_-]+)", 1),  // New sharing format
        (r"share/r/([a-zA-Z0-9_-]+)", 1),  // New sharing format for reels
        (r"reel/(\d+)", 1),  // Reel format
    ];

    for (pattern, group) in patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(url) {
                if let Some(id) = captures.get(group) {
                    return Some(id.as_str().to_string());
                }
            }
        }
    }

    None
}

fn main() {
    println!("🧪 Testing New Facebook URL Format Support");
    println!("==========================================");
    
    let test_urls = vec![
        "https://www.facebook.com/share/v/16VH5WhMbd/",  // The problematic URL
        "https://www.facebook.com/watch/?v=1234567890",  // Traditional format
        "https://www.facebook.com/reel/1234567890",      // Reel format
        "https://fb.watch/abc123",                       // Short format
        "https://www.facebook.com/share/r/abc123def/",   // New reel sharing format
    ];
    
    for url in test_urls {
        println!("\n🔗 Testing URL: {}", url);
        
        let is_valid = is_valid_facebook_url(url);
        println!("   ✅ Valid: {}", is_valid);
        
        if is_valid {
            if let Some(video_id) = extract_video_id(url) {
                println!("   🆔 Video ID: {}", video_id);
            } else {
                println!("   ❌ Could not extract video ID");
            }
        }
    }
    
    println!("\n🎯 Focus Test: Your Problematic URL");
    println!("===================================");
    let problem_url = "https://www.facebook.com/share/v/16VH5WhMbd/";
    
    println!("URL: {}", problem_url);
    let is_valid = is_valid_facebook_url(problem_url);
    println!("Valid: {}", is_valid);
    
    if is_valid {
        if let Some(video_id) = extract_video_id(problem_url) {
            println!("Video ID: {}", video_id);
            println!("✅ SUCCESS: URL is now recognized and video ID extracted!");
        } else {
            println!("❌ PARTIAL: URL is valid but video ID extraction failed");
        }
    } else {
        println!("❌ FAILED: URL is still not recognized as valid");
    }
}
