//! # No Compression Test
//!
//! ## Purpose
//! Tests video extraction without HTTP compression
//!
//! ## Category
//! Network Testing
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_no_compression
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

use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Facebook Request Without Compression");
    println!("===============================================");
    
    // Test URL
    let test_url = "https://www.facebook.com/watch/?v=2209933269449948";
    println!("🎯 Testing URL: {}", test_url);
    
    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    println!("\n📡 Making HTTP request WITHOUT compression headers...");
    
    let response = client
        .get(test_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        // NOTE: No Accept-Encoding header to avoid compression
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-User", "?1")
        .header("Cache-Control", "max-age=0")
        .header("sec-ch-ua", "\"Google Chrome\";v=\"120\", \"Chromium\";v=\"120\", \"Not_A Brand\";v=\"99\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
        .send()
        .await?;
    
    println!("📊 Response Status: {}", response.status());
    println!("📊 Response Headers:");
    for (name, value) in response.headers() {
        println!("   {}: {:?}", name, value);
    }
    
    if !response.status().is_success() {
        println!("❌ HTTP request failed with status: {}", response.status());
        return Ok(());
    }
    
    // Try to get text
    match response.text().await {
        Ok(html) => {
            println!("\n✅ Successfully got text response!");
            println!("📄 HTML Response Analysis:");
            println!("   Length: {} characters", html.len());
            
            // Check if it looks like valid HTML
            let is_html = html.contains("<html") || html.contains("<!DOCTYPE") || html.contains("<head");
            println!("   Looks like HTML: {}", is_html);
            
            if is_html {
                println!("   Contains 'video': {}", html.contains("video"));
                println!("   Contains '.mp4': {}", html.contains(".mp4"));
                println!("   Contains 'fbcdn.net': {}", html.contains("fbcdn.net"));
                println!("   Contains 'playable_url': {}", html.contains("playable_url"));
                
                // Save sample for inspection
                std::fs::write("facebook_uncompressed_sample.html", &html[..5000.min(html.len())])?;
                println!("   Saved first 5KB to facebook_uncompressed_sample.html");
            } else {
                println!("   First 200 chars: {}", &html[..200.min(html.len())]);
            }
        }
        Err(e) => {
            println!("\n❌ Failed to get text: {}", e);
            
            // Try to get bytes and analyze
            println!("🔍 Trying to get raw bytes...");
            let response2 = client
                .get(test_url)
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .send()
                .await?;
            
            let bytes = response2.bytes().await?;
            println!("   Raw bytes length: {}", bytes.len());
            if bytes.len() > 10 {
                println!("   First 10 bytes: {:02x?}", &bytes[..10]);
            }
        }
    }
    
    println!("\n📋 Test completed");
    Ok(())
}
