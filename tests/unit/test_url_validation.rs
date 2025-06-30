//! # URL Validation Test
//!
//! ## Purpose
//! Comprehensive URL validation and format checking
//!
//! ## Category
//! URL Validation
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_url_validation
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
        r"^https?://(?:www\.)?facebook\.com/watch/?\?.*v=\d+",  // Fixed: handle both /watch? and /watch/?
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
            let matches = regex.is_match(url);
            println!("Pattern: {} -> {}", pattern, if matches { "✅ MATCH" } else { "❌ NO MATCH" });
            matches
        } else {
            println!("Pattern: {} -> ❌ INVALID REGEX", pattern);
            false
        }
    })
}

fn main() {
    println!("🧪 Testing URL validation for problematic URL");
    println!("==============================================");
    
    let test_url = "https://www.facebook.com/watch/?v=900186485344685";
    println!("Test URL: {}", test_url);
    println!();
    
    let is_valid = is_valid_facebook_url(test_url);
    println!();
    println!("Final result: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
}
