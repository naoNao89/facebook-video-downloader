//! # Facebook Parsing Debug
//!
//! ## Purpose
//! Debug tool for Facebook HTML parsing issues
//!
//! ## Category
//! Debug Tools
//!
//! ## Usage
//! ```bash
//! cargo run --bin debug_facebook_parsing
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
use regex::Regex;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Facebook HTML Parsing Debug Tool");
    println!("===================================");
    
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
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Cache-Control", "no-cache")
        .send()
        .await?;
    
    println!("📊 Response Status: {}", response.status());
    println!("📊 Response Headers:");
    for (name, value) in response.headers() {
        if name.as_str().contains("content") || name.as_str().contains("location") {
            println!("   {}: {:?}", name, value);
        }
    }
    
    if !response.status().is_success() {
        println!("❌ HTTP request failed with status: {}", response.status());
        return Ok(());
    }
    
    let html = response.text().await?;
    println!("\n📄 HTML Response Analysis:");
    println!("   Length: {} characters", html.len());
    
    // Check for common Facebook patterns
    println!("\n🔍 Content Analysis:");
    println!("   Contains 'fbcdn.net': {}", html.contains("fbcdn.net"));
    println!("   Contains '.mp4': {}", html.contains(".mp4"));
    println!("   Contains 'video': {}", html.contains("video"));
    println!("   Contains 'playable_url': {}", html.contains("playable_url"));
    println!("   Contains 'browser_native': {}", html.contains("browser_native"));
    println!("   Contains 'dash_manifest': {}", html.contains("dash_manifest"));
    println!("   Contains 'hls_playlist': {}", html.contains("hls_playlist"));
    
    // Show first 500 characters to understand the structure
    println!("\n📝 HTML Sample (first 500 chars):");
    println!("{}", &html[..500.min(html.len())]);
    
    // Check if it's a redirect page or login page
    if html.contains("login") || html.contains("Log In") {
        println!("\n⚠️  Detected login page - Facebook is requiring authentication");
    }
    
    if html.contains("redirect") || html.contains("window.location") {
        println!("\n⚠️  Detected redirect - Facebook may be redirecting the request");
    }
    
    // Test video URL patterns
    println!("\n🎬 Testing Video URL Patterns:");
    let video_url_patterns = vec![
        r#"https://video[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
        r#"https://scontent[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
        r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
        r#""playable_url":"([^"]*\.mp4[^"]*)"#,
        r#""playable_url_quality_hd":"([^"]*\.mp4[^"]*)"#,
        r#""browser_native_hd_url":"([^"]*\.mp4[^"]*)"#,
        r#""browser_native_sd_url":"([^"]*\.mp4[^"]*)"#,
        r#""src":"([^"]*\.mp4[^"]*)"#,
        r#""url":"([^"]*\.mp4[^"]*)"#,
    ];
    
    let mut found_any = false;
    for (i, pattern) in video_url_patterns.iter().enumerate() {
        println!("   Pattern {}: {}", i + 1, pattern);
        
        if let Ok(regex) = Regex::new(pattern) {
            let matches: Vec<_> = regex.captures_iter(&html).collect();
            println!("     Matches: {}", matches.len());
            
            for (j, capture) in matches.iter().take(3).enumerate() {
                if let Some(url_match) = capture.get(1).or_else(|| capture.get(0)) {
                    let url = url_match.as_str();
                    println!("     Match {}: {}", j + 1, &url[..100.min(url.len())]);
                    found_any = true;
                }
            }
        } else {
            println!("     ❌ Invalid regex pattern");
        }
    }
    
    if !found_any {
        println!("\n❌ No video URLs found with current patterns");
        
        // Look for any URLs that might be video-related
        println!("\n🔍 Searching for any .mp4 or video-related content:");
        if let Ok(mp4_regex) = Regex::new(r"\S*\.mp4\S*") {
            let mp4_matches: Vec<_> = mp4_regex.find_iter(&html).collect();
            println!("   Found {} .mp4 references", mp4_matches.len());
            for (i, m) in mp4_matches.iter().take(5).enumerate() {
                println!("     {}: {}", i + 1, m.as_str());
            }
        }
        
        // Look for JSON data that might contain video info
        if let Ok(json_regex) = Regex::new(r#"\{"[^"]*video[^"]*":[^}]*\}"#) {
            let json_matches: Vec<_> = json_regex.find_iter(&html).collect();
            println!("   Found {} JSON objects with 'video'", json_matches.len());
            for (i, m) in json_matches.iter().take(3).enumerate() {
                println!("     {}: {}", i + 1, &m.as_str()[..200.min(m.as_str().len())]);
            }
        }
    } else {
        println!("\n✅ Found video URLs!");
    }
    
    // Check for Facebook's anti-bot measures
    println!("\n🛡️  Anti-bot Detection:");
    if html.contains("captcha") || html.contains("CAPTCHA") {
        println!("   ⚠️  CAPTCHA detected");
    }
    if html.contains("bot") || html.contains("automated") {
        println!("   ⚠️  Bot detection messages found");
    }
    if html.contains("rate limit") || html.contains("too many requests") {
        println!("   ⚠️  Rate limiting detected");
    }
    if html.len() < 1000 {
        println!("   ⚠️  Response too short - possible blocking");
    }
    
    println!("\n📋 Summary:");
    println!("   - Response received: ✅");
    println!("   - HTML length: {} chars", html.len());
    println!("   - Video URLs found: {}", if found_any { "✅" } else { "❌" });
    
    Ok(())
}
