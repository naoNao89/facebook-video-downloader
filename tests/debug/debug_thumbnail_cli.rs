//! # Thumbnail CLI Debug Tool
//!
//! ## Purpose
//! Command-line debug tool for thumbnail extraction issues
//!
//! ## Category
//! Debug Tools
//!
//! ## Usage
//! ```bash
//! cargo run --bin debug_thumbnail_cli
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

//! # Facebook Video Thumbnail Debugging CLI
//!
//! A specialized debugging tool for diagnosing thumbnail loading issues in the Facebook Video Downloader.
//! This tool provides comprehensive debugging for thumbnail URL extraction, accessibility testing,
//! and network diagnostics.
//!
//! ## Features
//! - Thumbnail URL extraction debugging with detailed pattern matching
//! - Network accessibility testing for thumbnail URLs
//! - CORS and security header analysis
//! - Image format and size validation
//! - Detailed logging of all thumbnail-related operations
//!
//! ## Usage
//! ```bash
//! cargo run --bin debug_thumbnail_cli [URL]
//! ```

use facebook_extractor_core::FacebookExtractor;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Thumbnail accessibility test result
#[derive(Debug, Serialize, Deserialize)]
pub struct ThumbnailAccessibilityResult {
    pub status_code: u16,
    pub content_length: u64,
    pub content_type: String,
    pub final_url: String,
    pub is_cors_enabled: bool,
    pub response_headers: Vec<(String, String)>,
    pub is_image_valid: bool,
    pub image_dimensions: Option<(u32, u32)>,
    pub error_message: Option<String>,
}

/// HTML analysis result for thumbnail debugging
#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlAnalysisResult {
    pub html_length: usize,
    pub pattern_matches: Vec<(String, Vec<String>)>,
    pub image_urls: Vec<String>,
    pub meta_tags: Vec<(String, String)>,
    pub has_facebook_content: bool,
    pub has_video_content: bool,
}

/// Test thumbnail URL accessibility and gather diagnostic information
async fn test_thumbnail_accessibility(thumbnail_url: &str) -> Result<ThumbnailAccessibilityResult, Box<dyn std::error::Error>> {
    println!("🔍 Testing thumbnail accessibility for: {}", thumbnail_url);
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    // First, try a HEAD request to get headers without downloading the full image
    println!("📡 Sending HEAD request to check headers...");
    let head_response = client.head(thumbnail_url).send().await?;
    
    let status_code = head_response.status().as_u16();
    let final_url = head_response.url().to_string();
    
    println!("📊 HEAD Response Analysis:");
    println!("   Status: {}", status_code);
    println!("   Final URL: {}", final_url);
    
    // Collect all response headers
    let mut response_headers = Vec::new();
    let mut content_length = 0u64;
    let mut content_type = String::new();
    let mut is_cors_enabled = false;
    
    for (name, value) in head_response.headers() {
        let header_name = name.to_string();
        let header_value = value.to_str().unwrap_or("").to_string();
        
        println!("   {}: {}", header_name, header_value);
        
        match header_name.to_lowercase().as_str() {
            "content-length" => {
                content_length = header_value.parse().unwrap_or(0);
            }
            "content-type" => {
                content_type = header_value.clone();
            }
            "access-control-allow-origin" => {
                is_cors_enabled = true;
            }
            _ => {}
        }
        
        response_headers.push((header_name, header_value));
    }
    
    // If HEAD request was successful, try to download a small portion to validate image
    let mut is_image_valid = false;
    let mut image_dimensions = None;
    let mut error_message = None;
    
    if status_code == 200 {
        println!("🖼️ Attempting to download and validate image...");
        match client.get(thumbnail_url)
            .header("Range", "bytes=0-2048") // Download first 2KB to check image header
            .send()
            .await
        {
            Ok(response) => {
                match response.bytes().await {
                    Ok(bytes) => {
                        println!("📥 Downloaded {} bytes for validation", bytes.len());
                        
                        // Basic image format detection
                        if bytes.len() >= 4 {
                            let header = &bytes[0..4];
                            match header {
                                [0xFF, 0xD8, 0xFF, _] => {
                                    println!("✅ Valid JPEG image detected");
                                    is_image_valid = true;
                                }
                                [0x89, 0x50, 0x4E, 0x47] => {
                                    println!("✅ Valid PNG image detected");
                                    is_image_valid = true;
                                }
                                [0x47, 0x49, 0x46, _] => {
                                    println!("✅ Valid GIF image detected");
                                    is_image_valid = true;
                                }
                                [0x52, 0x49, 0x46, 0x46] => {
                                    println!("✅ Valid WebP image detected");
                                    is_image_valid = true;
                                }
                                _ => {
                                    println!("⚠️ Unknown image format or corrupted header");
                                    println!("   Header bytes: {:02X} {:02X} {:02X} {:02X}", 
                                        header[0], header[1], header[2], header[3]);
                                }
                            }
                        }
                        
                        // Try to extract image dimensions for JPEG
                        if is_image_valid && bytes.len() >= 10 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
                            image_dimensions = extract_jpeg_dimensions(&bytes);
                        }
                    }
                    Err(e) => {
                        error_message = Some(format!("Failed to download image data: {}", e));
                        println!("❌ Failed to download image data: {}", e);
                    }
                }
            }
            Err(e) => {
                error_message = Some(format!("Failed to make image request: {}", e));
                println!("❌ Failed to make image request: {}", e);
            }
        }
    } else {
        error_message = Some(format!("HTTP error: {}", status_code));
        println!("❌ HTTP error: {}", status_code);
    }
    
    Ok(ThumbnailAccessibilityResult {
        status_code,
        content_length,
        content_type,
        final_url,
        is_cors_enabled,
        response_headers,
        is_image_valid,
        image_dimensions,
        error_message,
    })
}

/// Extract JPEG image dimensions from header bytes
fn extract_jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    // This is a simplified JPEG dimension extractor
    // In a real implementation, you'd want a proper image library
    if bytes.len() < 20 {
        return None;
    }

    // Look for SOF (Start of Frame) markers
    for i in 0..bytes.len().saturating_sub(10) {
        if bytes[i] == 0xFF && (bytes[i + 1] == 0xC0 || bytes[i + 1] == 0xC2) {
            if i + 9 < bytes.len() {
                let height = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
                let width = u16::from_be_bytes([bytes[i + 7], bytes[i + 8]]) as u32;
                return Some((width, height));
            }
        }
    }

    None
}

/// Analyze HTML content for thumbnail patterns and debugging
async fn analyze_html_for_thumbnails(url: &str) -> Result<HtmlAnalysisResult, Box<dyn std::error::Error>> {
    println!("🔍 Fetching HTML content for analysis...");

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url).send().await?;
    let html = response.text().await?;

    println!("📄 HTML content fetched: {} characters", html.len());

    // Define thumbnail patterns to test
    let thumbnail_patterns = vec![
        (r#""thumbnail":"([^"]+)""#, "JSON thumbnail"),
        (r#""image":"([^"]+\.jpg[^"]*)""#, "JSON image JPG"),
        (r#""image":"([^"]+\.png[^"]*)""#, "JSON image PNG"),
        (r#"<meta property="og:image" content="([^"]+)""#, "Open Graph image"),
        (r#""preview_image":"([^"]+)""#, "Preview image"),
        (r#""thumbnailImage":"([^"]+)""#, "Thumbnail image"),
        (r#""poster":"([^"]+)""#, "Poster image"),
        (r#""video_thumbnail":"([^"]+)""#, "Video thumbnail"),
        (r#""cover_photo":\{"source":"([^"]+)""#, "Cover photo"),
    ];

    let mut pattern_matches = Vec::new();

    // Test each pattern
    for (pattern, description) in thumbnail_patterns {
        if let Ok(regex) = regex::Regex::new(pattern) {
            let matches: Vec<String> = regex.captures_iter(&html)
                .filter_map(|cap| cap.get(1))
                .map(|m| {
                    let mut url = m.as_str().to_string();
                    url = url.replace("\\", "");
                    url = url.replace("\\/", "/");
                    url = url.replace("\\u0026", "&");
                    url
                })
                .filter(|url| url.starts_with("http"))
                .collect();

            if !matches.is_empty() {
                pattern_matches.push((format!("{} ({})", description, pattern), matches));
            }
        }
    }

    // Find all image URLs
    let mut image_urls = Vec::new();
    if let Ok(regex) = regex::Regex::new(r#"https://[^"]*\.(jpg|jpeg|png|webp|gif)[^"]*"#) {
        image_urls = regex.find_iter(&html)
            .map(|m| m.as_str().to_string())
            .collect();
    }

    // Find meta tags
    let mut meta_tags = Vec::new();
    if let Ok(regex) = regex::Regex::new(r#"<meta[^>]+property="([^"]+)"[^>]+content="([^"]+)""#) {
        meta_tags = regex.captures_iter(&html)
            .filter_map(|cap| {
                let property = cap.get(1)?.as_str();
                let content = cap.get(2)?.as_str();
                Some((property.to_string(), content.to_string()))
            })
            .collect();
    }

    // Check for Facebook and video content indicators
    let has_facebook_content = html.contains("facebook.com") || html.contains("fbcdn.net");
    let has_video_content = html.contains("video") || html.contains("playable_url");

    Ok(HtmlAnalysisResult {
        html_length: html.len(),
        pattern_matches,
        image_urls,
        meta_tags,
        has_facebook_content,
        has_video_content,
    })
}

/// Enhanced thumbnail debugging with pattern analysis
async fn debug_thumbnail_extraction(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 ENHANCED THUMBNAIL DEBUGGING");
    println!("================================");
    println!("🎯 Target URL: {}", url);
    println!();
    
    // Step 1: Extract video info using the core library
    println!("📋 Step 1: Extracting video information...");
    let extractor = FacebookExtractor::new()?;
    
    match extractor.extract_video_info(url).await {
        Ok(video_info) => {
            println!("✅ Video extraction successful!");
            println!("   📝 Title: {}", video_info.title);
            println!("   🆔 Video ID: {}", video_info.video_id);
            println!();
            
            // Step 2: Analyze thumbnail extraction
            println!("🖼️ Step 2: Thumbnail Analysis");
            println!("==============================");
            
            if !video_info.thumbnail.is_empty() {
                println!("✅ Thumbnail URL found: {}", video_info.thumbnail);
                println!();
                
                // Step 3: Test thumbnail accessibility
                println!("🔍 Step 3: Thumbnail Accessibility Testing");
                println!("==========================================");
                
                match test_thumbnail_accessibility(&video_info.thumbnail).await {
                    Ok(result) => {
                        println!("📊 Accessibility Test Results:");
                        println!("   🌐 Status Code: {}", result.status_code);
                        println!("   📏 Content Length: {} bytes", result.content_length);
                        println!("   🏷️ Content Type: {}", result.content_type);
                        println!("   🔗 Final URL: {}", result.final_url);
                        println!("   🔓 CORS Enabled: {}", result.is_cors_enabled);
                        println!("   🖼️ Valid Image: {}", result.is_image_valid);
                        
                        if let Some((width, height)) = result.image_dimensions {
                            println!("   📐 Dimensions: {}x{}", width, height);
                        }
                        
                        if let Some(error) = &result.error_message {
                            println!("   ❌ Error: {}", error);
                        }
                        
                        println!();
                        println!("🔍 Response Headers:");
                        for (name, value) in &result.response_headers {
                            println!("   {}: {}", name, value);
                        }
                        
                        // Step 4: Provide diagnostic recommendations
                        println!();
                        println!("💡 DIAGNOSTIC RECOMMENDATIONS");
                        println!("==============================");
                        
                        if result.status_code != 200 {
                            println!("❌ HTTP Error: The thumbnail URL returned status {}", result.status_code);
                            println!("   💡 This indicates the thumbnail URL is invalid or inaccessible");
                            println!("   🔧 Check if the URL extraction patterns need updating");
                        }
                        
                        if !result.is_cors_enabled {
                            println!("⚠️ CORS Not Enabled: The thumbnail server doesn't allow cross-origin requests");
                            println!("   💡 This may prevent thumbnails from loading in web browsers");
                            println!("   🔧 Consider using a proxy or server-side thumbnail fetching");
                        }
                        
                        if !result.is_image_valid {
                            println!("❌ Invalid Image: The downloaded content is not a valid image");
                            println!("   💡 The URL might point to HTML or other non-image content");
                            println!("   🔧 Verify the thumbnail extraction patterns");
                        }
                        
                        if result.content_length == 0 {
                            println!("⚠️ Empty Content: The thumbnail URL returned no content");
                            println!("   💡 The image might be dynamically generated or protected");
                        }
                        
                        if result.status_code == 200 && result.is_image_valid && result.is_cors_enabled {
                            println!("✅ Thumbnail appears to be fully accessible!");
                            println!("   💡 The issue might be in the UI rendering logic");
                            println!("   🔧 Check the frontend thumbnail display implementation");
                        }
                        
                    }
                    Err(e) => {
                        println!("❌ Accessibility test failed: {}", e);
                        println!("   💡 This indicates a network or URL issue");
                    }
                }
                
            } else {
                println!("❌ No thumbnail URL found in video info");
                println!("   💡 This indicates the thumbnail extraction failed");
                println!("   🔧 The HTML parsing patterns may need updating");
                
                // Step 3: Analyze HTML for thumbnail patterns
                println!();
                println!("🔍 Step 3: HTML Pattern Analysis");
                println!("=================================");

                match analyze_html_for_thumbnails(url).await {
                    Ok(analysis) => {
                        println!("📊 HTML Analysis Results:");
                        println!("   📄 HTML Length: {} characters", analysis.html_length);
                        println!("   🔍 Thumbnail Patterns Found: {}", analysis.pattern_matches.len());
                        println!("   🖼️ Image URLs Found: {}", analysis.image_urls.len());
                        println!("   📋 Meta Tags Found: {}", analysis.meta_tags.len());

                        if !analysis.pattern_matches.is_empty() {
                            println!();
                            println!("🎯 Thumbnail Pattern Matches:");
                            for (i, (pattern, matches)) in analysis.pattern_matches.iter().enumerate() {
                                println!("   {}. Pattern: {}", i + 1, pattern);
                                println!("      Matches: {}", matches.len());
                                for (j, m) in matches.iter().take(3).enumerate() {
                                    println!("      {}. {}", j + 1, &m[..80.min(m.len())]);
                                }
                                if matches.len() > 3 {
                                    println!("      ... and {} more", matches.len() - 3);
                                }
                            }
                        }

                        if !analysis.image_urls.is_empty() {
                            println!();
                            println!("🖼️ All Image URLs Found:");
                            for (i, url) in analysis.image_urls.iter().take(10).enumerate() {
                                println!("   {}. {}", i + 1, &url[..100.min(url.len())]);
                            }
                            if analysis.image_urls.len() > 10 {
                                println!("   ... and {} more", analysis.image_urls.len() - 10);
                            }
                        }

                        println!();
                        println!("💡 HTML Analysis Recommendations:");
                        if analysis.pattern_matches.is_empty() {
                            println!("   ❌ No thumbnail patterns matched");
                            println!("   🔧 The HTML structure may have changed");
                            println!("   💡 Consider updating thumbnail extraction patterns");
                        } else {
                            println!("   ✅ Thumbnail patterns found in HTML");
                            println!("   🔧 Check why the extraction is not working");
                        }

                        if analysis.image_urls.is_empty() {
                            println!("   ❌ No image URLs found in HTML");
                            println!("   💡 The page may be heavily JavaScript-rendered");
                        } else {
                            println!("   ✅ Image URLs found in HTML");
                            println!("   💡 Consider using alternative image URLs as fallback");
                        }
                    }
                    Err(e) => {
                        println!("❌ HTML analysis failed: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Video extraction failed: {}", e);
            println!("   💡 Cannot proceed with thumbnail debugging");
            return Err(e.into());
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    
    let test_url = if args.len() > 1 {
        &args[1]
    } else {
        println!("❌ Please provide a Facebook video URL as an argument");
        println!("Usage: cargo run --bin debug_thumbnail_cli <URL>");
        println!();
        println!("Example:");
        println!("cargo run --bin debug_thumbnail_cli \"https://www.facebook.com/watch/?v=123456789\"");
        std::process::exit(1);
    };
    
    println!("🚀 Facebook Video Thumbnail Debugging Tool");
    println!("==========================================");
    println!();
    
    debug_thumbnail_extraction(test_url).await?;
    
    println!();
    println!("🎉 Thumbnail debugging completed!");
    
    Ok(())
}
