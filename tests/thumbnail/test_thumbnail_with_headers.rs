//! # Thumbnail Headers Test
//!
//! ## Purpose
//! Tests thumbnail extraction with different HTTP headers and user agents
//!
//! ## Category
//! Thumbnail Testing
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_thumbnail_with_headers
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

use reqwest;
use std::fs;
use std::time::Duration;
use base64::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️ Facebook Thumbnail Download Test (With Enhanced Headers)");
    println!("============================================================");
    
    // Test URLs from your logs
    let test_thumbnails = vec![
        "https://scontent.fsgn2-9.fna.fbcdn.net/v/t15.5256-10/503040179_999470422398244_8044190889189586036_n",
        "https://scontent.fsgn2-6.fna.fbcdn.net/v/t15.5256-10/500210780_1213412090031928_5809932302650839313_",
    ];
    
    println!("🎯 Testing {} thumbnail URLs with enhanced Facebook headers", test_thumbnails.len());
    
    let mut successful_downloads = 0;
    let mut test_results: Vec<(String, usize, String)> = Vec::new();

    for (i, thumbnail_url) in test_thumbnails.iter().enumerate() {
        println!("\n🔍 Testing thumbnail {}: {}", i + 1, thumbnail_url);
        
        match download_with_facebook_headers(thumbnail_url, i + 1).await {
            Ok((size, data_url)) => {
                println!("✅ Successfully downloaded thumbnail {} ({} bytes)", i + 1, size);
                successful_downloads += 1;
                test_results.push((thumbnail_url.to_string(), size, data_url));
            }
            Err(e) => {
                println!("❌ Failed to download thumbnail {}: {}", i + 1, e);
                test_results.push((thumbnail_url.to_string(), 0, String::new()));
            }
        }
    }
    
    // Create test HTML file
    println!("\n📋 Creating test HTML file");
    println!("---------------------------");
    
    if successful_downloads > 0 {
        create_test_html(&test_results).await?;
        println!("✅ Created thumbnail_test.html with {} thumbnails", successful_downloads);
    }
    
    // Summary
    println!("\n📊 Test Summary");
    println!("===============");
    println!("🎯 Total thumbnails tested: {}", test_thumbnails.len());
    println!("✅ Successful downloads: {}", successful_downloads);
    println!("❌ Failed downloads: {}", test_thumbnails.len() - successful_downloads);
    
    if successful_downloads > 0 {
        println!("\n🎉 SUCCESS: Thumbnail downloads working with enhanced headers!");
        println!("   The issue is likely that the Tauri frontend isn't using the right headers.");
        println!("   Open thumbnail_test.html in a browser to verify the images display correctly.");
        println!("   Check the individual .jpg files to see the raw images.");
    } else {
        println!("\n❌ ISSUE: All thumbnail downloads failed even with enhanced headers");
        println!("   This suggests Facebook has strict access controls on these CDN URLs.");
        println!("   The Tauri backend should be using placeholder thumbnails as fallback.");
    }

    Ok(())
}

async fn download_with_facebook_headers(thumbnail_url: &str, index: usize) -> Result<(usize, String), Box<dyn std::error::Error>> {
    println!("   🌐 Downloading with Facebook headers: {}", &thumbnail_url[..80.min(thumbnail_url.len())]);
    
    // Use the same sophisticated client setup as the Tauri backend
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;
    
    // Try multiple strategies like the Tauri backend does
    
    // Strategy 1: Enhanced Facebook headers
    println!("   🔄 Strategy 1: Enhanced Facebook headers");
    match try_facebook_headers(&client, thumbnail_url).await {
        Ok((size, data_url)) => {
            println!("   ✅ Strategy 1 succeeded");
            return Ok((size, data_url));
        }
        Err(e) => {
            println!("   ⚠️ Strategy 1 failed: {}", e);
        }
    }
    
    // Strategy 2: Alternative user agents
    println!("   🔄 Strategy 2: Alternative user agents");
    match try_alternative_headers(&client, thumbnail_url).await {
        Ok((size, data_url)) => {
            println!("   ✅ Strategy 2 succeeded");
            return Ok((size, data_url));
        }
        Err(e) => {
            println!("   ⚠️ Strategy 2 failed: {}", e);
        }
    }
    
    // Strategy 3: Minimal headers
    println!("   🔄 Strategy 3: Minimal headers");
    match try_minimal_headers(&client, thumbnail_url).await {
        Ok((size, data_url)) => {
            println!("   ✅ Strategy 3 succeeded");
            return Ok((size, data_url));
        }
        Err(e) => {
            println!("   ⚠️ Strategy 3 failed: {}", e);
        }
    }
    
    // Save to file
    let filename = format!("test_thumbnail_{}.jpg", index);
    fs::write(&filename, &[])?; // Empty file to indicate failure
    println!("   💾 Created empty file: {}", filename);
    
    Err("All strategies failed".into())
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
    
    process_response(response).await
}

async fn try_alternative_headers(client: &reqwest::Client, url: &str) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")
        .header("Accept", "image/*,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Connection", "keep-alive")
        .send()
        .await?;
    
    process_response(response).await
}

async fn try_minimal_headers(client: &reqwest::Client, url: &str) -> Result<(usize, String), Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .send()
        .await?;
    
    process_response(response).await
}

async fn process_response(response: reqwest::Response) -> Result<(usize, String), Box<dyn std::error::Error>> {
    println!("   📊 Response status: {}", response.status());
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let bytes = response.bytes().await?;
    println!("   📊 Downloaded {} bytes ({:.2} KB)", bytes.len(), bytes.len() as f64 / 1024.0);
    
    if bytes.len() == 0 {
        return Err("Downloaded thumbnail is empty (0 bytes)".into());
    }
    
    if bytes.len() < 1000 {
        println!("   ⚠️ Very small file ({} bytes) - might be an error page", bytes.len());
    }
    
    // Check file type
    if bytes.len() >= 4 {
        let magic = &bytes[0..4];
        let file_type = match magic {
            [0xFF, 0xD8, 0xFF, _] => "JPEG",
            [0x89, 0x50, 0x4E, 0x47] => "PNG",
            [0x47, 0x49, 0x46, _] => "GIF",
            [0x52, 0x49, 0x46, 0x46] => "WebP",
            _ => "Unknown",
        };
        println!("   🎨 File type: {}", file_type);
    }
    
    // Save to file
    let filename = format!("test_thumbnail_success.jpg");
    fs::write(&filename, &bytes)?;
    println!("   💾 Saved to: {}", filename);
    
    // Convert to base64 data URL
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:image/jpeg;base64,{}", base64_data);
    println!("   🔄 Base64 data URL created ({} chars)", data_url.len());
    
    Ok((bytes.len(), data_url))
}

async fn create_test_html(test_results: &[(String, usize, String)]) -> Result<(), Box<dyn std::error::Error>> {
    let mut html_content = String::from(r#"<!DOCTYPE html>
<html>
<head>
    <title>Facebook Thumbnail Test Results</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .thumbnail-test { margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 8px; }
        .thumbnail-test h3 { margin-top: 0; }
        .thumbnail-test img { max-width: 400px; border: 1px solid #ccc; margin: 10px 0; }
        .url { word-break: break-all; font-family: monospace; background: #f5f5f5; padding: 5px; }
        .success { border-left: 4px solid #4CAF50; }
        .failed { border-left: 4px solid #f44336; }
    </style>
</head>
<body>
    <h1>🖼️ Facebook Thumbnail Download Test Results (Enhanced Headers)</h1>
    <p>This page shows the results of downloading Facebook video thumbnails with enhanced headers.</p>
"#);

    for (i, (url, size, data_url)) in test_results.iter().enumerate() {
        let class = if *size > 0 { "success" } else { "failed" };
        
        html_content.push_str(&format!(r#"
    <div class="thumbnail-test {}">
        <h3>Thumbnail {} - {} bytes</h3>
        <p><strong>URL:</strong></p>
        <div class="url">{}</div>
        "#, class, i + 1, size, url));
        
        if *size > 0 && !data_url.is_empty() {
            html_content.push_str(&format!(r#"
        <p><strong>Base64 Data URL Image:</strong></p>
        <img src="{}" alt="Base64 thumbnail" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
        <p style="display:none; color: red;">❌ Failed to load base64 data</p>
        "#, data_url));
        } else {
            html_content.push_str("<p style='color: red;'>❌ Download failed - no image to display</p>");
        }
        
        html_content.push_str("    </div>");
    }

    html_content.push_str(r#"
</body>
</html>"#);

    fs::write("thumbnail_test.html", html_content)?;
    Ok(())
}
