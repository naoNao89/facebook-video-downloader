//! # Extract Button Test
//!
//! ## Purpose
//! Tests the extract button functionality in the user interface
//!
//! ## Category
//! UI Integration
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_extract_button
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
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Facebook Extract Button Test");
    println!("===============================");
    
    // Test the problematic URL from your logs
    let test_url = "https://www.facebook.com/watch/?v=900186485344685";
    println!("🔍 Test URL: {}", test_url);
    
    // Test URL validation first
    println!("\n📋 Step 1: Test URL validation");
    println!("------------------------------");
    
    let is_valid = is_valid_facebook_url(test_url);
    println!("✅ URL validation result: {}", if is_valid { "VALID" } else { "INVALID" });
    
    if !is_valid {
        println!("❌ URL validation failed - this would prevent extraction");
        return Ok(());
    }
    
    // Test video extraction
    println!("\n📋 Step 2: Test video extraction");
    println!("---------------------------------");
    
    let extractor = FacebookExtractor::new()?;

    println!("🚀 Starting video extraction...");
    match extractor.extract_video_info(test_url).await {
        Ok(video_info) => {
            println!("✅ Video extraction successful!");
            println!("📊 Video Info:");
            println!("   Title: {}", video_info.title);
            println!("   Duration: {}", video_info.duration);
            println!("   Video ID: {}", video_info.video_id);
            println!("   Qualities: {} available", video_info.qualities.len());
            println!("   Thumbnail: {}", if video_info.thumbnail.is_empty() { "None" } else { "Available" });
            
            if !video_info.thumbnail.is_empty() {
                println!("   Thumbnail URL: {}", &video_info.thumbnail[..100.min(video_info.thumbnail.len())]);
            }
            
            // Test thumbnail fetching if available
            if !video_info.thumbnail.is_empty() {
                println!("\n📋 Step 3: Test thumbnail fetching");
                println!("----------------------------------");
                
                match test_thumbnail_fetch(&video_info.thumbnail).await {
                    Ok(size) => {
                        println!("✅ Thumbnail fetch successful ({} bytes)", size);
                    }
                    Err(e) => {
                        println!("❌ Thumbnail fetch failed: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Video extraction failed: {}", e);
            println!("   This would cause the Extract button to show an error");
        }
    }
    
    println!("\n📊 Test Summary");
    println!("===============");
    println!("🎯 URL: {}", test_url);
    println!("✅ URL validation: {}", if is_valid { "PASSED" } else { "FAILED" });
    println!("   If URL validation fails, the Extract button won't proceed");
    println!("   If extraction fails, the Extract button will show an error");
    println!("   If extraction succeeds but thumbnails don't load, it's a frontend issue");
    
    Ok(())
}

async fn test_thumbnail_fetch(thumbnail_url: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;
    
    println!("🌐 Fetching thumbnail: {}", &thumbnail_url[..80.min(thumbnail_url.len())]);
    
    let response = client
        .get(thumbnail_url)
        .header("Accept", "image/*,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Referer", "https://www.facebook.com/")
        .header("Origin", "https://www.facebook.com")
        .send()
        .await?;
    
    println!("📊 Response status: {}", response.status());
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let bytes = response.bytes().await?;
    println!("📊 Downloaded {} bytes", bytes.len());
    
    Ok(bytes.len())
}

fn is_valid_facebook_url(url: &str) -> bool {
    use regex::Regex;
    
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
            regex.is_match(url)
        } else {
            false
        }
    })
}
