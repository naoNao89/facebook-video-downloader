//! # Facebook Video Extraction Improvement Test
//!
//! ## Purpose
//! Tests the improved Facebook video extraction functionality with enhanced
//! error handling and detailed feedback. This test validates the core extraction
//! capabilities and provides specific guidance for different error scenarios.
//!
//! ## Scope
//! - Tests core video extraction functionality
//! - Validates video metadata extraction
//! - Tests error handling for different failure modes
//! - Provides specific troubleshooting guidance
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_improved_extraction
//! ```
//!
//! ## Test Cases
//! 1. **Video Information Extraction**: Tests complete video info retrieval
//! 2. **Stream Quality Analysis**: Validates video quality detection
//! 3. **Error Classification**: Tests different error scenarios
//! 4. **Troubleshooting Guidance**: Provides specific help for failures
//!
//! ## Expected Behavior
//! - Should successfully extract video information for public videos
//! - Should provide detailed stream quality information
//! - Should classify errors appropriately and provide helpful guidance
//! - Should display comprehensive video metadata
//!
//! ## Dependencies
//! - facebook-extractor-core: Core extraction functionality
//!
//! ## Setup Requirements
//! - Internet connection for Facebook access
//! - Valid Facebook video URL (hardcoded test URL)

use facebook_video_downloader_core::{FacebookExtractor, FacebookExtractorError};

/// Main test function for improved extraction functionality
///
/// Tests the enhanced video extraction with comprehensive error handling
/// and detailed feedback for different failure scenarios.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Improved Facebook Video Extraction");
    println!("=============================================");

    // Test URL
    let test_url = "https://www.facebook.com/watch?v=2891256754381289";
    println!("🎯 Testing URL: {}", test_url);

    // Create extractor
    let extractor = FacebookExtractor::new()?;
    
    println!("\n📡 Starting extraction...");
    
    match extractor.extract_video_info(test_url).await {
        Ok(video_info) => {
            println!("\n✅ Extraction successful!");
            println!("📋 Video Info:");
            println!("   Title: {}", video_info.title);
            println!("   Video ID: {}", video_info.video_id);
            println!("   Duration: {:?}", video_info.duration);
            println!("   Qualities: {} streams", video_info.qualities.len());
            
            for (i, quality) in video_info.qualities.iter().enumerate() {
                println!("   Stream {}: {}x{} ({}p) - {}",
                    i + 1,
                    quality.width,
                    quality.height,
                    quality.height,
                    &quality.download_url[..100.min(quality.download_url.len())]
                );
            }
            
            println!("   Content Type: {:?}", video_info.content_type);
            println!("   Privacy Level: {:?}", video_info.privacy_level);
            println!("   Access Method: {:?}", video_info.access_method);
            println!("   Source URL: {}", video_info.source_url);
            println!("   Extraction Time: {}", video_info.extraction_timestamp);
            
            if !video_info.thumbnail.is_empty() {
                println!("   Thumbnail: {}", &video_info.thumbnail[..100.min(video_info.thumbnail.len())]);
            }
        }
        Err(e) => {
            println!("\n❌ Extraction failed:");
            println!("   Error: {}", e);
            println!("   Error type: {:?}", e);
            
            // Provide specific guidance based on error type
            match e {
                FacebookExtractorError::AuthenticationRequired => {
                    println!("\n💡 Suggestion: This video may require login to Facebook");
                    println!("   Try using a different video URL or implement authentication");
                }
                FacebookExtractorError::HtmlParsing { message } => {
                    if message.contains("CAPTCHA") {
                        println!("\n💡 Suggestion: Facebook detected automation");
                        println!("   Try again later or use different request patterns");
                    } else if message.contains("No video URLs found") {
                        println!("\n💡 Suggestion: Facebook may have changed their HTML structure");
                        println!("   Check if the URL is valid and the video is publicly accessible");
                    }
                }
                FacebookExtractorError::StreamAnalysis { message } => {
                    println!("\n💡 Suggestion: Video URLs were found but couldn't be processed");
                    println!("   This might be a temporary issue or the URLs may be incomplete");
                }
                _ => {
                    println!("\n💡 Suggestion: Check network connectivity and try again");
                }
            }
        }
    }
    
    println!("\n📋 Test completed");
    Ok(())
}
