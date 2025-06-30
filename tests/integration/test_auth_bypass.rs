//! # Authentication Bypass Test
//!
//! ## Purpose
//! Tests methods for accessing public videos without authentication
//!
//! ## Category
//! Authentication
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_auth_bypass
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

#!/usr/bin/env rust-script

//! Test script to verify authentication bypass functionality
//! 
//! This script tests both URL validation and authentication bypass
//! for the Facebook video extractor.

use facebook_extractor_core::{FacebookExtractor, FacebookExtractorError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🧪 Testing Facebook Video Extractor Authentication Bypass");
    println!("{}", "=".repeat(60));

    let extractor = FacebookExtractor::new()?;
    
    // Test URLs
    let test_urls = vec![
        ("Traditional watch URL", "https://www.facebook.com/watch?v=578837441825938"),
        ("New sharing format", "https://www.facebook.com/share/v/16VH5WhMbd/"),
        ("Another sharing format", "https://www.facebook.com/share/r/1234567890/"),
    ];
    
    for (description, url) in test_urls {
        println!("\n🔍 Testing: {}", description);
        println!("URL: {}", url);
        println!("{}", "-".repeat(50));

        // Test URL validation first
        let validation = extractor.validate_url(url);
        println!("✅ URL Validation: valid={}, type={:?}, video_id={:?}",
                 validation.is_valid, validation.content_type, validation.video_id);

        if validation.is_valid {
                    // Test video extraction with authentication bypass
                    println!("🎬 Attempting video extraction with authentication bypass...");
                    
                    match extractor.extract_video_info(url).await {
                        Ok(video_info) => {
                            println!("🎉 SUCCESS! Video extraction completed:");
                            println!("   Title: {}", video_info.title);
                            println!("   Video ID: {}", video_info.video_id);
                            println!("   Content Type: {:?}", video_info.content_type);
                            println!("   Privacy Level: {:?}", video_info.privacy_level);
                            println!("   Access Method: {:?}", video_info.access_method);
                            println!("   Qualities Found: {}", video_info.qualities.len());
                            
                            for (i, quality) in video_info.qualities.iter().enumerate() {
                                println!("     {}. {} - {}x{} - {}", 
                                         i + 1, quality.quality, quality.width, quality.height, quality.format);
                            }
                        }
                        Err(FacebookExtractorError::AuthenticationRequired) => {
                            println!("❌ FAILED: Still requires authentication (bypass didn't work)");
                        }
                        Err(e) => {
                            println!("⚠️  EXTRACTION FAILED: {}", e);
                            println!("   This may be expected for private videos or if Facebook is blocking access");
                        }
                    }
        } else {
            println!("❌ URL validation failed - this shouldn't happen with our fix");
            if let Some(error) = validation.error_message {
                println!("   Error: {}", error);
            }
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!("🏁 Test completed!");
    println!("\nExpected Results:");
    println!("✅ All URLs should pass validation");
    println!("✅ Extraction should attempt bypass methods instead of failing immediately on auth");
    println!("⚠️  Some extractions may still fail due to privacy/blocking, but should provide better error messages");
    
    Ok(())
}
