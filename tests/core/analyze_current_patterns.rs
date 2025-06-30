//! # Current Patterns Analysis
//!
//! ## Purpose
//! Analyzes current Facebook video URL patterns for extraction
//!
//! ## Category
//! Pattern Analysis
//!
//! ## Usage
//! ```bash
//! cargo run --bin analyze_current_patterns
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
    println!("🔍 Analyzing Current Facebook Video URL Patterns");
    println!("===============================================");
    
    // Test URL from the logs
    let test_url = "https://www.facebook.com/watch?v=1193939392365151";
    println!("🎯 Testing URL: {}", test_url);
    
    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    println!("\n📡 Making HTTP request without compression...");
    
    let response = client
        .get(test_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        // No Accept-Encoding header
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
    
    // Check for authentication/blocking
    if html.contains("login") || html.contains("Log In") {
        println!("⚠️  Authentication required detected");
    }
    
    // Look for video-related content
    println!("\n🔍 Content Analysis:");
    println!("   Contains 'video': {}", html.contains("video"));
    println!("   Contains '.mp4': {}", html.contains(".mp4"));
    println!("   Contains 'fbcdn.net': {}", html.contains("fbcdn.net"));
    println!("   Contains 'playable_url': {}", html.contains("playable_url"));
    println!("   Contains 'browser_native': {}", html.contains("browser_native"));
    
    // Test current patterns from the extractor
    println!("\n🎯 Testing Current Extractor Patterns:");
    let current_patterns = vec![
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
    
    // Look for new patterns
    println!("\n🔍 Searching for New Video URL Patterns:");
    
    // Look for any .mp4 URLs
    if let Ok(mp4_regex) = Regex::new(r"https://[^'\s]*\.mp4[^'\s]*") {
        let mp4_matches: Vec<_> = mp4_regex.find_iter(&html).collect();
        println!("   Found {} .mp4 URLs:", mp4_matches.len());
        for (i, m) in mp4_matches.iter().take(5).enumerate() {
            println!("     {}: {}", i + 1, m.as_str());
        }
    }
    
    // Look for video-related JSON structures
    println!("\n📋 Searching for Video JSON Structures:");
    let json_patterns = vec![
        r#"\{"[^"]*video[^"]*":[^}]*\}"#,
        r#"\{"[^"]*playable[^"]*":[^}]*\}"#,
        r#"\{"[^"]*stream[^"]*":[^}]*\}"#,
        r#"videoData[^}]*\}"#,
        r#"video_url[^}]*\}"#,
    ];
    
    for (i, pattern) in json_patterns.iter().enumerate() {
        if let Ok(regex) = Regex::new(pattern) {
            let matches: Vec<_> = regex.find_iter(&html).collect();
            println!("   JSON Pattern {}: {} matches", i + 1, matches.len());
            for (j, m) in matches.iter().take(2).enumerate() {
                println!("     Match {}: {}", j + 1, &m.as_str()[..200.min(m.as_str().len())]);
            }
        }
    }
    
    // Look for script tags with video data
    println!("\n📜 Analyzing Script Tags:");
    if let Ok(script_regex) = Regex::new(r"<script[^>]*>(.*?)</script>") {
        let script_matches: Vec<_> = script_regex.captures_iter(&html).collect();
        println!("   Found {} script tags", script_matches.len());
        
        let mut video_scripts = 0;
        for script_match in script_matches.iter() {
            if let Some(script_content) = script_match.get(1) {
                let content = script_content.as_str();
                if content.contains("video") || content.contains(".mp4") || content.contains("fbcdn") {
                    video_scripts += 1;
                    if video_scripts <= 3 {
                        println!("   Video-related script {} (first 500 chars):", video_scripts);
                        println!("     {}", &content[..500.min(content.len())]);
                        
                        // Look for specific patterns in this script
                        if content.contains(".mp4") {
                            if let Ok(mp4_in_script) = Regex::new(r#""[^"]*\.mp4[^"]*""#) {
                                let script_mp4s: Vec<_> = mp4_in_script.find_iter(content).collect();
                                println!("     Found {} .mp4 URLs in this script:", script_mp4s.len());
                                for (k, mp4_match) in script_mp4s.iter().take(3).enumerate() {
                                    println!("       {}: {}", k + 1, mp4_match.as_str());
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("   Total video-related scripts: {}", video_scripts);
    }
    
    // Save sample for manual inspection
    println!("\n💾 Saving HTML sample...");
    std::fs::write("facebook_current_sample.html", &html[..20000.min(html.len())])?;
    println!("   Saved first 20KB to facebook_current_sample.html");
    
    println!("\n📋 Analysis Complete");
    Ok(())
}
