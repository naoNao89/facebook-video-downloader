//! # Facebook HTML Analysis
//!
//! ## Purpose
//! Analyzes Facebook HTML structure for video extraction patterns
//!
//! ## Category
//! HTML Analysis
//!
//! ## Usage
//! ```bash
//! cargo run --bin analyze_facebook_html
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
    println!("🔍 Facebook HTML Analysis Tool");
    println!("==============================");
    
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
    
    if !response.status().is_success() {
        println!("❌ HTTP request failed with status: {}", response.status());
        return Ok(());
    }
    
    let html = response.text().await?;
    println!("📄 HTML Response Analysis:");
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
    
    // Look for any .mp4 URLs
    println!("\n🎬 Searching for .mp4 URLs:");
    if let Ok(mp4_regex) = Regex::new(r"https://[^'\s]*\.mp4[^'\s]*") {
        let mp4_matches: Vec<_> = mp4_regex.find_iter(&html).collect();
        println!("   Found {} .mp4 URLs", mp4_matches.len());
        for (i, m) in mp4_matches.iter().take(10).enumerate() {
            println!("     {}: {}", i + 1, m.as_str());
        }
    }
    
    // Look for fbcdn.net URLs
    println!("\n🌐 Searching for fbcdn.net URLs:");
    if let Ok(fbcdn_regex) = Regex::new(r"https://[^'\s]*\.fbcdn\.net[^'\s]*") {
        let fbcdn_matches: Vec<_> = fbcdn_regex.find_iter(&html).collect();
        println!("   Found {} fbcdn.net URLs", fbcdn_matches.len());
        for (i, m) in fbcdn_matches.iter().take(10).enumerate() {
            println!("     {}: {}", i + 1, &m.as_str()[..100.min(m.as_str().len())]);
        }
    }
    
    // Look for JSON objects that might contain video data
    println!("\n📋 Searching for JSON with video data:");
    let json_patterns = vec![
        r#"\{"[^"]*video[^"]*":[^}]*\}"#,
        r#"\{"[^"]*playable[^"]*":[^}]*\}"#,
        r#"\{"[^"]*stream[^"]*":[^}]*\}"#,
        r#"\{"[^"]*url[^"]*":[^}]*\.mp4[^}]*\}"#,
    ];
    
    for (i, pattern) in json_patterns.iter().enumerate() {
        if let Ok(regex) = Regex::new(pattern) {
            let matches: Vec<_> = regex.find_iter(&html).collect();
            println!("   Pattern {}: {} matches", i + 1, matches.len());
            for (j, m) in matches.iter().take(3).enumerate() {
                println!("     Match {}: {}", j + 1, &m.as_str()[..200.min(m.as_str().len())]);
            }
        }
    }
    
    // Look for script tags that might contain video data
    println!("\n📜 Searching for script tags with video data:");
    if let Ok(script_regex) = Regex::new(r"<script[^>]*>(.*?)</script>") {
        let script_matches: Vec<_> = script_regex.find_iter(&html).collect();
        println!("   Found {} script tags", script_matches.len());
        
        let mut video_scripts = 0;
        for script_match in script_matches.iter() {
            let script_content = script_match.as_str();
            if script_content.contains("video") || script_content.contains(".mp4") || script_content.contains("fbcdn") {
                video_scripts += 1;
                if video_scripts <= 3 {
                    println!("   Video-related script {} (first 300 chars):", video_scripts);
                    println!("     {}", &script_content[..300.min(script_content.len())]);
                }
            }
        }
        println!("   Found {} video-related scripts", video_scripts);
    }
    
    // Look for specific Facebook video patterns
    println!("\n🎯 Testing Current Facebook Patterns:");
    let current_patterns = vec![
        r#""playable_url":"([^"]*\.mp4[^"]*)"#,
        r#""browser_native_hd_url":"([^"]*\.mp4[^"]*)"#,
        r#""browser_native_sd_url":"([^"]*\.mp4[^"]*)"#,
        r#""dash_manifest":"([^"]*)"#,
        r#""hls_playlist":"([^"]*)"#,
        r#"https://video[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
        r#"https://scontent[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#,
    ];
    
    for (i, pattern) in current_patterns.iter().enumerate() {
        if let Ok(regex) = Regex::new(pattern) {
            let matches: Vec<_> = regex.captures_iter(&html).collect();
            println!("   Pattern {}: {} matches - {}", i + 1, matches.len(), pattern);
            for (j, capture) in matches.iter().take(2).enumerate() {
                if let Some(url_match) = capture.get(1).or_else(|| capture.get(0)) {
                    let url = url_match.as_str();
                    println!("     Match {}: {}", j + 1, &url[..100.min(url.len())]);
                }
            }
        }
    }
    
    // Save a sample of the HTML for manual inspection
    println!("\n💾 Saving HTML sample for manual inspection...");
    std::fs::write("facebook_html_sample.html", &html[..10000.min(html.len())])?;
    println!("   Saved first 10KB to facebook_html_sample.html");
    
    println!("\n📋 Summary:");
    println!("   - Response received: ✅");
    println!("   - HTML length: {} chars", html.len());
    println!("   - Contains video-related content: {}", 
        html.contains("video") || html.contains(".mp4") || html.contains("fbcdn"));
    
    Ok(())
}
