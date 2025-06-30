//! # Response Format Debug
//!
//! ## Purpose
//! Debug tool for analyzing Facebook response formats
//!
//! ## Category
//! Debug Tools
//!
//! ## Usage
//! ```bash
//! cargo run --bin debug_response_format
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
    println!("🔍 Facebook Response Format Debug Tool");
    println!("======================================");
    
    // Test URL
    let test_url = "https://www.facebook.com/watch/?v=2209933269449948";
    println!("🎯 Testing URL: {}", test_url);
    
    // Create HTTP client with realistic browser headers
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    println!("\n📡 Making HTTP request...");
    
    let response = client
        .get(test_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Encoding", "gzip, deflate, br")
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
    
    // Get raw bytes first
    let bytes = response.bytes().await?;
    println!("\n📄 Raw Response Analysis:");
    println!("   Length: {} bytes", bytes.len());
    
    if bytes.len() > 10 {
        println!("   First 10 bytes: {:02x?}", &bytes[..10]);
        println!("   First 10 bytes as chars: {:?}", 
            bytes[..10].iter().map(|&b| b as char).collect::<String>());
    }
    
    // Check for compression signatures
    println!("\n🔍 Compression Analysis:");
    if bytes.len() > 2 {
        if bytes[0] == 0x1f && bytes[1] == 0x8b {
            println!("   ✅ Gzip signature detected (1f 8b)");
        } else {
            println!("   ❌ No gzip signature (expected 1f 8b, got {:02x} {:02x})", bytes[0], bytes[1]);
        }
    }
    
    // Try different decoding approaches
    println!("\n🧪 Decoding Tests:");
    
    // Test 1: Direct UTF-8
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => {
            println!("   ✅ Direct UTF-8 decoding successful");
            println!("   Sample (first 200 chars): {}", &text[..200.min(text.len())]);
        }
        Err(e) => {
            println!("   ❌ Direct UTF-8 decoding failed: {}", e);
        }
    }
    
    // Test 2: Gzip decompression
    if bytes.len() > 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut decompressed = String::new();
        match decoder.read_to_string(&mut decompressed) {
            Ok(_) => {
                println!("   ✅ Gzip decompression successful");
                println!("   Sample (first 200 chars): {}", &decompressed[..200.min(decompressed.len())]);
            }
            Err(e) => {
                println!("   ❌ Gzip decompression failed: {}", e);
            }
        }
    }
    
    // Test 3: Deflate decompression
    {
        use flate2::read::DeflateDecoder;
        use std::io::Read;
        
        let mut decoder = DeflateDecoder::new(&bytes[..]);
        let mut decompressed = String::new();
        match decoder.read_to_string(&mut decompressed) {
            Ok(_) => {
                println!("   ✅ Deflate decompression successful");
                println!("   Sample (first 200 chars): {}", &decompressed[..200.min(decompressed.len())]);
            }
            Err(e) => {
                println!("   ❌ Deflate decompression failed: {}", e);
            }
        }
    }
    
    // Test 4: Latin-1 decoding
    {
        let latin1_text: String = bytes.iter().map(|&b| b as char).collect();
        println!("   ✅ Latin-1 decoding (always works)");
        println!("   Sample (first 200 chars): {}", &latin1_text[..200.min(latin1_text.len())]);
    }
    
    println!("\n📋 Summary:");
    println!("   - Response received: ✅");
    println!("   - Response size: {} bytes", bytes.len());
    
    Ok(())
}
