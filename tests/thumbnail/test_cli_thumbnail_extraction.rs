//! # CLI Thumbnail Extraction Test
//!
//! ## Purpose
//! Command-line interface test for thumbnail extraction functionality
//!
//! ## Category
//! Thumbnail Testing
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_cli_thumbnail_extraction
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
use std::fs;
use std::time::Duration;
use reqwest;
use base64::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 CLI Facebook Thumbnail Extraction Test");
    println!("==========================================");
    
    // Test with the problematic URL
    let test_url = "https://www.facebook.com/watch/?v=900186485344685";
    println!("🔍 Test URL: {}", test_url);
    
    // Step 1: Extract video info using the CLI extractor
    println!("\n📋 Step 1: Extract video info via CLI");
    println!("-------------------------------------");
    
    let extractor = FacebookExtractor::new()?;
    
    match extractor.extract_video_info(test_url).await {
        Ok(video_info) => {
            println!("✅ Video extraction successful!");
            println!("📊 Video Info:");
            println!("   Title: {}", video_info.title);
            println!("   Duration: {}", video_info.duration);
            println!("   Video ID: {}", video_info.video_id);
            println!("   Qualities: {} available", video_info.qualities.len());
            
            // Focus on thumbnail extraction
            println!("\n📋 Step 2: Analyze extracted thumbnail");
            println!("--------------------------------------");
            
            if video_info.thumbnail.is_empty() {
                println!("❌ No thumbnail URL found in extraction");
                return Ok(());
            }
            
            println!("✅ Thumbnail URL found: {}", &video_info.thumbnail[..100.min(video_info.thumbnail.len())]);
            println!("📏 Full thumbnail URL length: {} characters", video_info.thumbnail.len());
            
            // Check if it's already a data URL
            if video_info.thumbnail.starts_with("data:") {
                println!("🎯 Thumbnail is already a data URL!");
                
                // Save the data URL content
                if let Some(base64_part) = video_info.thumbnail.strip_prefix("data:image/jpeg;base64,") {
                    match base64::engine::general_purpose::STANDARD.decode(base64_part) {
                        Ok(image_bytes) => {
                            println!("✅ Successfully decoded base64 data ({} bytes)", image_bytes.len());
                            
                            // Save to file
                            fs::write("cli_extracted_thumbnail.jpg", &image_bytes)?;
                            println!("💾 Saved thumbnail to: cli_extracted_thumbnail.jpg");
                            
                            // Create test HTML
                            create_test_html_with_data_url(&video_info.thumbnail).await?;
                            println!("💾 Created test HTML: cli_thumbnail_test.html");
                        }
                        Err(e) => {
                            println!("❌ Failed to decode base64 data: {}", e);
                        }
                    }
                } else if let Some(base64_part) = video_info.thumbnail.strip_prefix("data:image/svg+xml;base64,") {
                    println!("🎨 Thumbnail is an SVG placeholder");
                    match base64::engine::general_purpose::STANDARD.decode(base64_part) {
                        Ok(svg_bytes) => {
                            let svg_content = String::from_utf8_lossy(&svg_bytes);
                            println!("📄 SVG content: {}", &svg_content[..200.min(svg_content.len())]);
                            
                            // Save SVG
                            fs::write("cli_extracted_thumbnail.svg", &svg_bytes)?;
                            println!("💾 Saved SVG placeholder to: cli_extracted_thumbnail.svg");
                            
                            // Create test HTML
                            create_test_html_with_data_url(&video_info.thumbnail).await?;
                            println!("💾 Created test HTML: cli_thumbnail_test.html");
                        }
                        Err(e) => {
                            println!("❌ Failed to decode SVG data: {}", e);
                        }
                    }
                } else {
                    println!("⚠️ Unknown data URL format");
                }
            } else {
                // It's a regular URL, try to download it
                println!("🌐 Thumbnail is a regular URL, attempting download...");
                
                match download_thumbnail_with_strategies(&video_info.thumbnail).await {
                    Ok((size, data_url)) => {
                        println!("✅ Successfully downloaded thumbnail ({} bytes)", size);
                        
                        // Save the downloaded image
                        if let Some(base64_part) = data_url.strip_prefix("data:image/jpeg;base64,") {
                            if let Ok(image_bytes) = base64::engine::general_purpose::STANDARD.decode(base64_part) {
                                fs::write("cli_downloaded_thumbnail.jpg", &image_bytes)?;
                                println!("💾 Saved downloaded thumbnail to: cli_downloaded_thumbnail.jpg");
                            }
                        }
                        
                        // Create test HTML
                        create_test_html_with_data_url(&data_url).await?;
                        println!("💾 Created test HTML: cli_thumbnail_test.html");
                    }
                    Err(e) => {
                        println!("❌ Failed to download thumbnail: {}", e);
                        println!("🔄 This explains why thumbnails don't show in the Tauri app");
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Video extraction failed: {}", e);
            println!("   This would prevent the Extract button from working");
        }
    }
    
    println!("\n📊 CLI Test Summary");
    println!("===================");
    println!("🎯 URL: {}", test_url);
    println!("📁 Check the following files:");
    println!("   - cli_extracted_thumbnail.jpg (if extraction provided data URL)");
    println!("   - cli_downloaded_thumbnail.jpg (if download succeeded)");
    println!("   - cli_extracted_thumbnail.svg (if SVG placeholder)");
    println!("   - cli_thumbnail_test.html (test page)");
    
    Ok(())
}

async fn download_thumbnail_with_strategies(thumbnail_url: &str) -> Result<(usize, String), Box<dyn std::error::Error>> {
    println!("🔄 Trying multiple download strategies...");
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;
    
    // Strategy 1: Enhanced Facebook headers (same as Tauri backend)
    println!("   📡 Strategy 1: Enhanced Facebook headers");
    match try_facebook_headers(&client, thumbnail_url).await {
        Ok(result) => {
            println!("   ✅ Strategy 1 succeeded");
            return Ok(result);
        }
        Err(e) => {
            println!("   ❌ Strategy 1 failed: {}", e);
        }
    }
    
    // Strategy 2: Mobile user agent
    println!("   📱 Strategy 2: Mobile user agent");
    match try_mobile_headers(&client, thumbnail_url).await {
        Ok(result) => {
            println!("   ✅ Strategy 2 succeeded");
            return Ok(result);
        }
        Err(e) => {
            println!("   ❌ Strategy 2 failed: {}", e);
        }
    }
    
    // Strategy 3: Minimal headers
    println!("   🔧 Strategy 3: Minimal headers");
    match try_minimal_headers(&client, thumbnail_url).await {
        Ok(result) => {
            println!("   ✅ Strategy 3 succeeded");
            return Ok(result);
        }
        Err(e) => {
            println!("   ❌ Strategy 3 failed: {}", e);
        }
    }
    
    Err("All download strategies failed".into())
}

async fn try_facebook_headers(client: &reqwest::Client, url: &str) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Referer", "https://www.facebook.com/")
        .header("Origin", "https://www.facebook.com")
        .header("Connection", "keep-alive")
        .header("Sec-Fetch-Dest", "image")
        .header("Sec-Fetch-Mode", "no-cors")
        .header("Sec-Fetch-Site", "cross-site")
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .send()
        .await?;
    
    process_download_response(response).await
}

async fn try_mobile_headers(client: &reqwest::Client, url: &str) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")
        .header("Accept", "image/*,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Connection", "keep-alive")
        .send()
        .await?;
    
    process_download_response(response).await
}

async fn try_minimal_headers(client: &reqwest::Client, url: &str) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;
    process_download_response(response).await
}

async fn process_download_response(response: reqwest::Response) -> Result<(usize, String), Box<dyn std::error::Error>> {
    println!("      📊 Response status: {}", response.status());
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let bytes = response.bytes().await?;
    println!("      📊 Downloaded {} bytes", bytes.len());
    
    if bytes.len() == 0 {
        return Err("Empty response".into());
    }
    
    // Convert to base64 data URL
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:image/jpeg;base64,{}", base64_data);
    
    Ok((bytes.len(), data_url))
}

async fn create_test_html_with_data_url(data_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let html_content = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>CLI Thumbnail Extraction Test</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .thumbnail-container {{ margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 8px; }}
        .thumbnail-container img {{ max-width: 500px; border: 1px solid #ccc; margin: 10px 0; }}
        .data-url {{ word-break: break-all; font-family: monospace; background: #f5f5f5; padding: 10px; max-height: 200px; overflow-y: auto; }}
    </style>
</head>
<body>
    <h1>🎯 CLI Facebook Thumbnail Extraction Test</h1>
    <p>This page shows the thumbnail extracted via CLI from the Facebook video.</p>
    
    <div class="thumbnail-container">
        <h3>Extracted Thumbnail</h3>
        <img src="{}" alt="Extracted thumbnail" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
        <p style="display:none; color: red;">❌ Failed to load thumbnail</p>
        
        <h4>Data URL (first 500 characters):</h4>
        <div class="data-url">{}</div>
        
        <h4>Data URL Info:</h4>
        <ul>
            <li>Length: {} characters</li>
            <li>Type: {}</li>
            <li>Format: {}</li>
        </ul>
    </div>
</body>
</html>"#, 
        data_url,
        &data_url[..500.min(data_url.len())],
        data_url.len(),
        if data_url.starts_with("data:image/jpeg") { "JPEG Image" } 
        else if data_url.starts_with("data:image/svg") { "SVG Placeholder" }
        else { "Unknown" },
        if data_url.contains("base64") { "Base64 Encoded" } else { "Raw Data" }
    );

    fs::write("cli_thumbnail_test.html", html_content)?;
    Ok(())
}
