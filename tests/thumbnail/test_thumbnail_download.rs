//! # Thumbnail Download Test
//!
//! ## Purpose
//! Tests thumbnail downloading capabilities and file saving functionality
//!
//! ## Category
//! Thumbnail Testing
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_thumbnail_download
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
    println!("🖼️ Facebook Thumbnail Download Test");
    println!("===================================");
    println!();

    // Test URLs from your logs
    let test_thumbnails = vec![
        "https://scontent.fsgn2-9.fna.fbcdn.net/v/t15.5256-10/503040179_999470422398244_8044190889189586036_n",
        "https://scontent.fsgn2-6.fna.fbcdn.net/v/t15.5256-10/500210780_1213412090031928_5809932302650839313_",
    ];

    println!("🎯 Testing {} thumbnail URLs from your logs", test_thumbnails.len());

    // Test downloading each thumbnail directly
    println!("\n📋 Step 1: Test direct thumbnail downloads");
    println!("------------------------------------------");
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut successful_downloads = 0;
    let mut test_results: Vec<(String, usize, String)> = Vec::new();

    for (i, thumbnail_url) in test_thumbnails.iter().enumerate() {
        println!("\n🔍 Testing thumbnail {}: {}", i + 1, thumbnail_url);

        match download_and_test_thumbnail(&client, thumbnail_url, i + 1).await {
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
    println!("\n📋 Step 2: Create test HTML file");
    println!("--------------------------------");

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
        println!("\n🎉 SUCCESS: Thumbnail downloads working!");
        println!("   The issue is likely in the Tauri frontend rendering, not the backend fetching.");
        println!("   Open thumbnail_test.html in a browser to verify the images display correctly.");
        println!("   Check the individual .jpg files to see the raw images.");
    } else {
        println!("\n❌ ISSUE: All thumbnail downloads failed");
        println!("   This suggests a network connectivity issue or Facebook blocking access.");
    }

    Ok(())
}

async fn download_and_test_thumbnail(client: &reqwest::Client, thumbnail_url: &str, index: usize) -> Result<(usize, String), Box<dyn std::error::Error>> {
    println!("   🌐 Downloading from: {}", &thumbnail_url[..80.min(thumbnail_url.len())]);

    let response = client
        .get(thumbnail_url)
        .header("Accept", "image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .header("Sec-Fetch-Dest", "image")
        .header("Sec-Fetch-Mode", "no-cors")
        .header("Sec-Fetch-Site", "cross-site")
        .header("Referer", "https://www.facebook.com/")
        .send()
        .await?;

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
        println!("   ⚠️  Very small file ({} bytes) - might be an error page", bytes.len());
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
    let filename = format!("test_thumbnail_{}.jpg", index);
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
    <title>Facebook Thumbnail Test</title>
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
    <h1>🖼️ Facebook Thumbnail Download Test Results</h1>
    <p>This page shows the results of downloading Facebook video thumbnails directly.</p>
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
        <p><strong>Direct URL Image:</strong></p>
        <img src="{}" alt="Direct thumbnail" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
        <p style="display:none; color: red;">❌ Failed to load direct URL</p>

        <p><strong>Base64 Data URL Image:</strong></p>
        <img src="{}" alt="Base64 thumbnail" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
        <p style="display:none; color: red;">❌ Failed to load base64 data</p>
        "#, url, data_url));
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
