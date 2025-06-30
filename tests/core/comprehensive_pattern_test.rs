//! # Comprehensive Pattern Test
//!
//! ## Purpose
//! Comprehensive test of all video extraction patterns
//!
//! ## Category
//! Pattern Testing
//!
//! ## Usage
//! ```bash
//! cargo run --bin comprehensive_pattern_test
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
    println!("🔍 Comprehensive Facebook Video Pattern Analysis");
    println!("===============================================");
    
    // Test URL that was working in the logs
    let test_url = "https://www.facebook.com/watch?v=419280024562892";
    println!("🎯 Testing URL: {}", test_url);
    
    // Create HTTP client with realistic browser headers
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    let response = client
        .get(test_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-User", "?1")
        .header("Cache-Control", "max-age=0")
        .send()
        .await?;
    
    println!("📊 Response Status: {}", response.status());
    
    if !response.status().is_success() {
        println!("❌ HTTP request failed");
        return Ok(());
    }
    
    let html = response.text().await?;
    println!("📄 HTML Response Length: {} characters", html.len());
    
    // Comprehensive pattern testing
    println!("\n🎬 Testing ALL possible video URL patterns:");
    
    let all_patterns = vec![
        // Direct video URLs
        (r#"https://video[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#, "Direct video URLs"),
        (r#"https://scontent[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#, "Scontent URLs"),
        (r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*"#, "Any fbcdn URLs"),
        
        // JSON embedded URLs
        (r#""playable_url":"([^"]*\.mp4[^"]*)"#, "Playable URL"),
        (r#""playable_url_quality_hd":"([^"]*\.mp4[^"]*)"#, "Playable HD"),
        (r#""playable_url_quality_sd":"([^"]*\.mp4[^"]*)"#, "Playable SD"),
        (r#""browser_native_hd_url":"([^"]*\.mp4[^"]*)"#, "Browser Native HD"),
        (r#""browser_native_sd_url":"([^"]*\.mp4[^"]*)"#, "Browser Native SD"),
        (r#""src":"([^"]*\.mp4[^"]*)"#, "Src attribute"),
        (r#""url":"([^"]*\.mp4[^"]*)"#, "URL attribute"),
        
        // Facebook's new patterns
        (r#""representation_url":"([^"]*\.mp4[^"]*)"#, "Representation URL"),
        (r#""base_url":"([^"]*\.mp4[^"]*)"#, "Base URL"),
        (r#""video_url":"([^"]*\.mp4[^"]*)"#, "Video URL"),
        
        // Path-based quality patterns
        (r#"https://[^"]*\.fbcdn\.net/o1/v/t2/f[0-9]+/m[0-9]+/[^"\s]*"#, "Path-based quality"),
        (r#"https://video[^"]*\.fbcdn\.net/o1/v/t2/f[0-9]+/m[0-9]+/[^"\s]*"#, "Video path-based"),
        
        // Escaped URLs
        (r#"https:\\\/\\\/[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*"#, "Escaped URLs"),
        (r#"https:\\\/\\\/video[^"]*\.fbcdn\.net[^"]*\.mp4[^"\s]*"#, "Escaped video URLs"),
        
        // DASH and HLS
        (r#""dash_manifest":"([^"]*)"#, "DASH manifest"),
        (r#""hls_playlist":"([^"]*)"#, "HLS playlist"),
        (r#""manifest_url":"([^"]*)"#, "Manifest URL"),
        
        // EFG metadata patterns
        (r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*efg=[^"]*dash_vp9[^"\s]*"#, "VP9 streams"),
        (r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*efg=[^"]*dash_h264[^"\s]*"#, "H264 streams"),
        (r#"https://[^"]*\.fbcdn\.net/[^"]*\.mp4[^"\s]*efg=[^"]*xpv_progressive[^"\s]*"#, "Progressive streams"),
        
        // Quality-specific m-number patterns
        (r#"https://[^"]*\.fbcdn\.net/[^"]*m366[^"\s]*"#, "m366 (1080p)"),
        (r#"https://[^"]*\.fbcdn\.net/[^"]*m365[^"\s]*"#, "m365 (1080p)"),
        (r#"https://[^"]*\.fbcdn\.net/[^"]*m69[^"\s]*"#, "m69 (720p)"),
        (r#"https://[^"]*\.fbcdn\.net/[^"]*m68[^"\s]*"#, "m68 (480p)"),
        (r#"https://[^"]*\.fbcdn\.net/[^"]*m67[^"\s]*"#, "m67 (360p)"),
    ];
    
    let mut total_found = 0;
    let mut unique_urls = std::collections::HashSet::new();
    
    for (i, (pattern, description)) in all_patterns.iter().enumerate() {
        if let Ok(regex) = Regex::new(pattern) {
            let matches: Vec<_> = regex.find_iter(&html).collect();
            if matches.len() > 0 {
                println!("   ✅ Pattern {}: {} - {} matches", i + 1, description, matches.len());
                total_found += matches.len();
                
                for (j, m) in matches.iter().take(3).enumerate() {
                    let match_str = m.as_str();
                    // Clean up escaped URLs
                    let clean_url = match_str.replace("\\", "").replace("\\/", "/");
                    unique_urls.insert(clean_url.clone());
                    
                    let display_str = if clean_url.len() > 100 {
                        format!("{}...", &clean_url[..100])
                    } else {
                        clean_url
                    };
                    println!("     Match {}: {}", j + 1, display_str);
                }
            } else {
                println!("   ❌ Pattern {}: {} - 0 matches", i + 1, description);
            }
        }
    }
    
    println!("\n📊 Summary:");
    println!("   Total pattern matches: {}", total_found);
    println!("   Unique URLs found: {}", unique_urls.len());
    
    // Analyze unique URLs for quality indicators
    println!("\n🔍 Quality Analysis of Found URLs:");
    for (i, url) in unique_urls.iter().enumerate() {
        let quality = analyze_url_quality(url);
        println!("   URL {}: {} - {}", i + 1, quality, &url[..80.min(url.len())]);
    }
    
    if unique_urls.len() <= 2 {
        println!("\n⚠️  Issue Identified:");
        println!("   Facebook is only providing {} video URL(s) for this video.", unique_urls.len());
        println!("   This could mean:");
        println!("   1. The video only has limited quality options available");
        println!("   2. Facebook is restricting quality options due to anti-bot detection");
        println!("   3. The video source only has these quality levels");
        println!("   4. Additional qualities are loaded dynamically via JavaScript");
    } else {
        println!("\n✅ Multiple qualities found! The extraction logic should be updated to capture all of them.");
    }
    
    Ok(())
}

fn analyze_url_quality(url: &str) -> String {
    // Analyze URL for quality indicators
    if url.contains("m366") { return "1080p Full HD".to_string(); }
    if url.contains("m365") { return "1080p Full HD".to_string(); }
    if url.contains("m364") { return "900p".to_string(); }
    if url.contains("m363") { return "840p".to_string(); }
    if url.contains("m362") { return "840p".to_string(); }
    if url.contains("m361") { return "720p HD".to_string(); }
    if url.contains("m360") { return "720p HD".to_string(); }
    if url.contains("m69") { return "720p HD".to_string(); }
    if url.contains("m68") { return "480p SD".to_string(); }
    if url.contains("m67") { return "360p".to_string(); }
    
    if url.contains("browser_native_hd") { return "HD (Native)".to_string(); }
    if url.contains("browser_native_sd") { return "SD (Native)".to_string(); }
    
    if url.contains("1080") { return "1080p".to_string(); }
    if url.contains("720") { return "720p".to_string(); }
    if url.contains("480") { return "480p".to_string(); }
    if url.contains("360") { return "360p".to_string(); }
    
    "Unknown Quality".to_string()
}
