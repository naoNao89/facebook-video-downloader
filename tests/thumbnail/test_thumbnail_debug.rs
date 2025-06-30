//! # Facebook Video Thumbnail Debugging Test
//!
//! ## Purpose
//! A simple test script to run thumbnail debugging on a known Facebook video URL.
//! This helps verify that the debugging functionality works correctly and provides
//! basic validation of thumbnail extraction capabilities.
//!
//! ## Scope
//! - Tests basic video extraction functionality
//! - Validates thumbnail URL format and accessibility
//! - Performs basic network connectivity tests
//! - Provides debugging guidance when issues are found
//!
//! ## Usage
//! ```bash
//! # Test with default URL
//! cargo run --bin test_thumbnail_debug
//!
//! # Test with custom URL
//! cargo run --bin test_thumbnail_debug "https://www.facebook.com/watch/?v=YOUR_VIDEO_ID"
//! ```
//!
//! ## Test Cases
//! 1. **Basic Video Extraction**: Verifies core extraction works
//! 2. **Thumbnail Analysis**: Validates thumbnail URL format
//! 3. **Network Test**: Checks if thumbnail URL is accessible
//! 4. **Debug Guidance**: Provides troubleshooting steps
//!
//! ## Expected Behavior
//! - Should successfully extract video information
//! - Should find a valid thumbnail URL
//! - Should verify thumbnail accessibility via HTTP request
//! - Should provide clear feedback on any failures
//!
//! ## Dependencies
//! - facebook-extractor-core: Core extraction functionality
//! - reqwest: HTTP client for thumbnail URL testing
//! - env_logger: Logging configuration
//!
//! ## Setup Requirements
//! - Internet connection for Facebook access
//! - Valid Facebook video URL (public video recommended)

use facebook_extractor_core::FacebookExtractor;
use std::env;

/// Main test function that runs all thumbnail debugging tests
///
/// This function performs a comprehensive test of thumbnail extraction
/// functionality, including basic extraction, URL validation, and network
/// accessibility testing.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see debug output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("🧪 Facebook Video Thumbnail Debugging Test");
    println!("==========================================");
    
    // Use a test URL or get from command line
    let test_url = env::args().nth(1).unwrap_or_else(|| {
        "https://www.facebook.com/watch/?v=2209933269449948".to_string()
    });
    
    println!("🎯 Testing URL: {}", test_url);
    println!();
    
    // Test 1: Basic video extraction
    println!("📋 Test 1: Basic Video Extraction");
    println!("==================================");
    
    let extractor = FacebookExtractor::new()?;
    
    match extractor.extract_video_info(&test_url).await {
        Ok(video_info) => {
            println!("✅ Video extraction successful!");
            println!("   📝 Title: {}", video_info.title);
            println!("   🆔 Video ID: {}", video_info.video_id);
            println!("   ⏱️ Duration: {:?}", video_info.duration);
            println!("   🎬 Qualities: {} streams", video_info.qualities.len());
            
            // Test 2: Thumbnail analysis
            println!();
            println!("🖼️ Test 2: Thumbnail Analysis");
            println!("==============================");
            
            if !video_info.thumbnail.is_empty() {
                println!("✅ Thumbnail URL found: {}", video_info.thumbnail);
                
                // Basic URL validation
                if video_info.thumbnail.starts_with("https://") {
                    println!("✅ Thumbnail URL uses HTTPS");
                } else {
                    println!("⚠️ Thumbnail URL does not use HTTPS");
                }
                
                if video_info.thumbnail.contains("fbcdn.net") {
                    println!("✅ Thumbnail URL is from Facebook CDN");
                } else {
                    println!("⚠️ Thumbnail URL is not from Facebook CDN");
                }
                
                if video_info.thumbnail.contains(".jpg") || 
                   video_info.thumbnail.contains(".png") || 
                   video_info.thumbnail.contains(".webp") {
                    println!("✅ Thumbnail URL has image file extension");
                } else {
                    println!("⚠️ Thumbnail URL has no recognizable image extension");
                }
                
                // Test 3: Network accessibility (basic)
                println!();
                println!("🌐 Test 3: Basic Network Test");
                println!("==============================");
                
                match test_thumbnail_url(&video_info.thumbnail).await {
                    Ok(status) => {
                        println!("✅ Thumbnail URL is accessible (HTTP {})", status);
                        if status == 200 {
                            println!("✅ Thumbnail returns OK status");
                        } else {
                            println!("⚠️ Thumbnail returns non-OK status: {}", status);
                        }
                    }
                    Err(e) => {
                        println!("❌ Thumbnail URL is not accessible: {}", e);
                    }
                }
                
            } else {
                println!("❌ No thumbnail URL found");
                println!("   💡 This indicates thumbnail extraction failed");
                
                // Test 4: Debug why thumbnail extraction failed
                println!();
                println!("🔍 Test 4: Thumbnail Extraction Debug");
                println!("======================================");
                println!("   💡 The thumbnail extraction patterns may need updating");
                println!("   🔧 Check the HTML content for new thumbnail patterns");
                println!("   📋 Run the full debug_thumbnail_cli tool for detailed analysis");
            }
            
        }
        Err(e) => {
            println!("❌ Video extraction failed: {}", e);
            println!("   💡 Cannot test thumbnail functionality without video info");
            return Err(e.into());
        }
    }
    
    // Test 5: Summary and recommendations
    println!();
    println!("📊 Test Summary and Recommendations");
    println!("===================================");
    println!("✅ Basic thumbnail debugging test completed");
    println!();
    println!("💡 Next Steps:");
    println!("   1. Run the full debugging tool:");
    println!("      cargo run --bin debug_thumbnail_cli \"{}\"", test_url);
    println!("   2. Check the enhanced logging output in the core library");
    println!("   3. Verify thumbnail URLs in a web browser");
    println!("   4. Test with different Facebook video URLs");
    println!();
    println!("🔧 If thumbnails still don't work:");
    println!("   - Check CORS settings in the UI");
    println!("   - Verify image loading in the frontend");
    println!("   - Consider using a proxy for thumbnail loading");
    println!("   - Update thumbnail extraction patterns if needed");
    
    Ok(())
}

/// Test thumbnail URL accessibility via HTTP HEAD request
///
/// This function performs a basic network test to verify that a thumbnail
/// URL is accessible and returns a valid HTTP status code.
///
/// # Arguments
/// * `url` - The thumbnail URL to test
///
/// # Returns
/// * `Ok(status_code)` - HTTP status code if request succeeds
/// * `Err(error)` - Network or request error
///
/// # Example
/// ```rust
/// let status = test_thumbnail_url("https://example.com/thumbnail.jpg").await?;
/// assert_eq!(status, 200);
/// ```
async fn test_thumbnail_url(url: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    let response = client.head(url).send().await?;
    Ok(response.status().as_u16())
}