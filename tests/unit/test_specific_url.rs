//! # Specific URL Test
//!
//! ## Purpose
//! Tests extraction with specific Facebook video URLs
//!
//! ## Category
//! URL Validation
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_specific_url
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Specific Facebook URL for Thumbnail");
    println!("==============================================");
    
    let test_url = "https://www.facebook.com/watch?v=2891256754381289";
    println!("🎯 Testing URL: {}", test_url);
    
    let extractor = FacebookExtractor::new();
    
    match extractor.extract_video_info(test_url).await {
        Ok(video_info) => {
            println!("✅ Extraction successful!");
            println!("📋 Video Info:");
            println!("   Title: {}", video_info.title);
            println!("   Video ID: {}", video_info.video_id);
            println!("   Duration: {}", video_info.duration);
            println!("   Qualities: {} streams", video_info.qualities.len());
            
            // Focus on thumbnail
            println!("\n🖼️  THUMBNAIL ANALYSIS:");
            println!("   Thumbnail URL: {}", video_info.thumbnail);
            println!("   Is Empty: {}", video_info.thumbnail.is_empty());
            println!("   Contains 'placeholder': {}", video_info.thumbnail.contains("placeholder"));
            println!("   URL Length: {} characters", video_info.thumbnail.len());
            
            if !video_info.thumbnail.is_empty() {
                println!("   ✅ Thumbnail URL extracted successfully");
                
                // Test if thumbnail URL is accessible
                println!("\n🌐 Testing thumbnail accessibility...");
                let client = reqwest::Client::new();
                match client.head(&video_info.thumbnail).send().await {
                    Ok(response) => {
                        println!("   Status: {}", response.status());
                        if response.status().is_success() {
                            println!("   ✅ Thumbnail is accessible");
                        } else {
                            println!("   ❌ Thumbnail returned error status");
                        }
                        
                        if let Some(content_type) = response.headers().get("content-type") {
                            println!("   Content-Type: {:?}", content_type);
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Failed to access thumbnail: {}", e);
                    }
                }
            } else {
                println!("   ❌ No thumbnail URL extracted");
            }
            
            println!("\n📊 Stream Details:");
            for (i, quality) in video_info.qualities.iter().enumerate() {
                println!("   Stream {}: {}x{} ({}) - {}", 
                    i + 1, 
                    quality.width, 
                    quality.height, 
                    quality.quality_label,
                    quality.url.chars().take(80).collect::<String>()
                );
            }
        }
        Err(e) => {
            println!("❌ Extraction failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!("\n📋 Test completed");
    Ok(())
}
