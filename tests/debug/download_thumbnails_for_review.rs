//! # Thumbnail Review Download Tool
//!
//! ## Purpose
//! Downloads thumbnails for manual review and verification
//!
//! ## Category
//! Debug Tools
//!
//! ## Usage
//! ```bash
//! cargo run --bin download_thumbnails_for_review
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

//! # Download Facebook Thumbnails for Manual Review
//!
//! This script attempts to download Facebook video thumbnails using various
//! strategies and saves them to your computer for manual review.

use std::fs;
use std::time::Duration;
use reqwest;
use base64::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️ FACEBOOK THUMBNAIL DOWNLOAD FOR REVIEW");
    println!("==========================================");
    println!("📥 Attempting to download thumbnails with multiple strategies");
    println!();

    // Real thumbnail URLs from the comprehensive test results
    let thumbnail_urls = vec![
        ("video_1_yagi_storm", "https://scontent.fsgn2-4.fna.fbcdn.net/v/t15.5256-10/462797026_525008006909264_5132032112066275386_n.jpg?_nc_cat=101&ccb=1-7&_nc_sid=a27664&_nc_ohc=lYqP4SmPaCwQ7kNvwG6SXUb&_nc_oc=AdnkAq5r-8tQbXqrP_gsParHgFN28PDVeTuqe8r1j_JrQo2GT8fdUhalE6anNL5wolA&_nc_zt=23&_nc_ht=scontent.fsgn2-4.fna&_nc_gid=ckwBReXgxLAmM7MZZhcpXw&oh=00_AfLGOGn72s0KkjCYKqlpysYJ6pDcQbcspO23scjc7uiDEA&oe=6843345D"),
        ("video_2_tiramisu", "https://scontent.fsgn2-8.fna.fbcdn.net/v/t15.5256-10/503040179_999470422398244_8044190889189586036_n.jpg?_nc_cat=1&ccb=1-7&_nc_sid=596eb7&_nc_ohc=VuzgjUwDW0gQ7kNvwERhJyp&_nc_oc=AdlWbp-xVrh5LRxeTI4Ms2BwIi22jBEP6x3TeCPBtEEwCCQXg4lawDc6t61LREcieiU&_nc_zt=23&_nc_ht=scontent.fsgn2-8.fna&_nc_gid=_N7940x04hqm36MLwIKGaw&oh=00_AfI3xABvPhTfD-RkXHjR1vGy3bUmuPHyPCOtdqcyMSA-IQ&oe=68433382"),
        ("video_3_dog_walk", "https://scontent.fsgn2-5.fna.fbcdn.net/v/t15.5256-10/501702038_581579304981784_8803960733000164010_n.jpg?_nc_cat=104&ccb=1-7&_nc_sid=596eb7&_nc_ohc=1gE-d2KbqtwQ7kNvwFZTpnd&_nc_oc=Adl7bl6opGKkApXshqmBdog-5dNE_UngStzXkdis4hjZ501TKPu3Qm-F62eraPlfDtM&_nc_zt=23&_nc_ht=scontent.fsgn2-5.fna&_nc_gid=eK7ZJKTPXF_-vkuXPEsPRw&oh=00_AfKCT6A7wRTlg31d1g3dCBGd-Ag8iL20GOHTW2nUpwJOAA&oe=684321FE"),
    ];

    // Create downloads directory
    fs::create_dir_all("downloaded_thumbnails")?;
    println!("📁 Created directory: downloaded_thumbnails/");
    println!();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    for (name, url) in thumbnail_urls {
        println!("🔍 Processing: {}", name);
        println!("   URL: {}", &url[..100.min(url.len())]);
        
        // Strategy 1: Browser-like headers with Facebook referer
        println!("   📡 Strategy 1: Browser headers with Facebook referer");
        match download_with_browser_headers(&client, url, name, "browser").await {
            Ok(size) => {
                println!("   ✅ Success! Downloaded {} bytes", size);
                continue;
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
            }
        }

        // Strategy 2: Mobile browser headers
        println!("   📱 Strategy 2: Mobile browser headers");
        match download_with_mobile_headers(&client, url, name, "mobile").await {
            Ok(size) => {
                println!("   ✅ Success! Downloaded {} bytes", size);
                continue;
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
            }
        }

        // Strategy 3: Curl-like headers
        println!("   🔧 Strategy 3: Curl-like headers");
        match download_with_curl_headers(&client, url, name, "curl").await {
            Ok(size) => {
                println!("   ✅ Success! Downloaded {} bytes", size);
                continue;
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
            }
        }

        // Strategy 4: Minimal headers
        println!("   ⚡ Strategy 4: Minimal headers");
        match download_with_minimal_headers(&client, url, name, "minimal").await {
            Ok(size) => {
                println!("   ✅ Success! Downloaded {} bytes", size);
                continue;
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
            }
        }

        // Strategy 5: Try to extract direct image URL
        println!("   🔍 Strategy 5: Direct URL extraction");
        if let Some(direct_url) = extract_direct_image_url(url) {
            println!("   🎯 Trying direct URL: {}", &direct_url[..100.min(direct_url.len())]);
            match download_with_minimal_headers(&client, &direct_url, name, "direct").await {
                Ok(size) => {
                    println!("   ✅ Success! Downloaded {} bytes", size);
                    continue;
                }
                Err(e) => {
                    println!("   ❌ Failed: {}", e);
                }
            }
        }

        // If all strategies fail, create a placeholder
        println!("   🎨 All download strategies failed, creating placeholder");
        create_placeholder_image(name).await?;
        
        println!();
    }

    println!("📊 DOWNLOAD SUMMARY");
    println!("===================");
    println!("📁 Check the 'downloaded_thumbnails/' directory for results");
    println!("🖼️ Files may include:");
    println!("   - *.jpg - Successfully downloaded thumbnails");
    println!("   - *.svg - Placeholder images for failed downloads");
    println!();
    println!("💡 If downloads failed, this confirms the 403 Forbidden issue");
    println!("   The thumbnails require Facebook authentication to access");
    
    Ok(())
}

async fn download_with_browser_headers(
    client: &reqwest::Client,
    url: &str,
    name: &str,
    strategy: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
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

    save_response(response, name, strategy).await
}

async fn download_with_mobile_headers(
    client: &reqwest::Client,
    url: &str,
    name: &str,
    strategy: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")
        .header("Accept", "image/*,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Connection", "keep-alive")
        .header("Referer", "https://m.facebook.com/")
        .send()
        .await?;

    save_response(response, name, strategy).await
}

async fn download_with_curl_headers(
    client: &reqwest::Client,
    url: &str,
    name: &str,
    strategy: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .header("User-Agent", "curl/7.68.0")
        .header("Accept", "*/*")
        .send()
        .await?;

    save_response(response, name, strategy).await
}

async fn download_with_minimal_headers(
    client: &reqwest::Client,
    url: &str,
    name: &str,
    strategy: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;
    save_response(response, name, strategy).await
}

async fn save_response(
    response: reqwest::Response,
    name: &str,
    strategy: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;
    
    if bytes.len() == 0 {
        return Err("Empty response".into());
    }

    // Determine file extension based on content
    let extension = if bytes.starts_with(b"\xFF\xD8\xFF") {
        "jpg"
    } else if bytes.starts_with(b"\x89PNG") {
        "png"
    } else if bytes.starts_with(b"RIFF") && bytes[8..12] == *b"WEBP" {
        "webp"
    } else {
        "bin"
    };

    let filename = format!("downloaded_thumbnails/{}_{}.{}", name, strategy, extension);
    fs::write(&filename, &bytes)?;
    
    println!("      💾 Saved: {}", filename);
    Ok(bytes.len())
}

fn extract_direct_image_url(url: &str) -> Option<String> {
    // Try to extract a more direct image URL by removing Facebook's tracking parameters
    if url.contains("fbcdn.net") {
        // Remove common Facebook tracking parameters
        let base_url = url.split('?').next()?;
        Some(format!("{}?_nc_cat=1&ccb=1-7", base_url))
    } else {
        None
    }
}

async fn create_placeholder_image(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create SVG content without problematic format strings
    let mut svg_content = String::new();
    svg_content.push_str(r#"<svg width="320" height="180" xmlns="http://www.w3.org/2000/svg">"#);
    svg_content.push_str(r#"<defs><linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">"#);
    svg_content.push_str("<stop offset=\"0%\" style=\"stop-color:#1877f2;stop-opacity:1\" />");
    svg_content.push_str("<stop offset=\"100%\" style=\"stop-color:#42a5f5;stop-opacity:1\" />");
    svg_content.push_str(r#"</linearGradient></defs>"#);
    svg_content.push_str(r#"<rect width="320" height="180" fill="url(#bg)"/>"#);
    svg_content.push_str(r#"<circle cx="160" cy="90" r="30" fill="white" opacity="0.9"/>"#);
    svg_content.push_str("<polygon points=\"150,75 150,105 175,90\" fill=\"#1877f2\"/>");
    svg_content.push_str(&format!(r#"<text x="160" y="130" font-family="Arial" font-size="12" fill="white" text-anchor="middle">{}</text>"#, name.replace("_", " ").to_uppercase()));
    svg_content.push_str(r#"<text x="160" y="150" font-family="Arial" font-size="10" fill="white" text-anchor="middle" opacity="0.8">Download Failed</text>"#);
    svg_content.push_str(r#"</svg>"#);

    let filename = format!("downloaded_thumbnails/{}_placeholder.svg", name);
    fs::write(&filename, svg_content)?;
    println!("      🎨 Created placeholder: {}", filename);

    Ok(())
}
