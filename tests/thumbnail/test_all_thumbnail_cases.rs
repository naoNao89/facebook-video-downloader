//! # All Thumbnail Cases Test
//!
//! ## Purpose
//! Comprehensive test of all thumbnail extraction scenarios and edge cases
//!
//! ## Category
//! Thumbnail Testing
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_all_thumbnail_cases
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
    println!("🧪 Comprehensive Facebook Thumbnail Extraction Test");
    println!("===================================================");
    
    // Test multiple Facebook URL formats
    let test_urls = vec![
        "https://www.facebook.com/watch/?v=900186485344685",
        "https://www.facebook.com/watch?v=999173769094576", 
        "https://www.facebook.com/watch?v=718400947385071",
        "https://fb.watch/abc123def456", // This might fail but let's test
    ];
    
    let mut test_results = Vec::new();
    
    for (i, test_url) in test_urls.iter().enumerate() {
        println!("\n🔍 Test Case {}: {}", i + 1, test_url);
        println!("{}", "=".repeat(60));
        
        match test_thumbnail_extraction(test_url, i + 1).await {
            Ok(result) => {
                test_results.push(result);
                println!("✅ Test case {} completed", i + 1);
            }
            Err(e) => {
                println!("❌ Test case {} failed: {}", i + 1, e);
                test_results.push(ThumbnailTestResult {
                    url: test_url.to_string(),
                    extraction_success: false,
                    thumbnail_found: false,
                    thumbnail_url: String::new(),
                    thumbnail_type: "None".to_string(),
                    download_success: false,
                    file_size: 0,
                    error_message: Some(e.to_string()),
                });
            }
        }
    }
    
    // Generate comprehensive report
    generate_test_report(&test_results).await?;
    
    println!("\n📊 Final Summary");
    println!("================");
    println!("🎯 Total tests: {}", test_results.len());
    println!("✅ Successful extractions: {}", test_results.iter().filter(|r| r.extraction_success).count());
    println!("🖼️ Thumbnails found: {}", test_results.iter().filter(|r| r.thumbnail_found).count());
    println!("⬇️ Successful downloads: {}", test_results.iter().filter(|r| r.download_success).count());
    println!("📁 Check 'thumbnail_test_report.html' for detailed results");
    
    Ok(())
}

#[derive(Debug, Clone)]
struct ThumbnailTestResult {
    url: String,
    extraction_success: bool,
    thumbnail_found: bool,
    thumbnail_url: String,
    thumbnail_type: String, // "data_url", "cdn_url", "placeholder", "none"
    download_success: bool,
    file_size: usize,
    error_message: Option<String>,
}

async fn test_thumbnail_extraction(test_url: &str, test_index: usize) -> Result<ThumbnailTestResult, Box<dyn std::error::Error>> {
    println!("📋 Step 1: Extract video info");
    
    let extractor = FacebookExtractor::new()?;
    
    let video_info = match extractor.extract_video_info(test_url).await {
        Ok(info) => {
            println!("✅ Video extraction successful");
            println!("   Title: {}", info.title);
            println!("   Video ID: {}", info.video_id);
            info
        }
        Err(e) => {
            return Ok(ThumbnailTestResult {
                url: test_url.to_string(),
                extraction_success: false,
                thumbnail_found: false,
                thumbnail_url: String::new(),
                thumbnail_type: "None".to_string(),
                download_success: false,
                file_size: 0,
                error_message: Some(format!("Extraction failed: {}", e)),
            });
        }
    };
    
    println!("\n📋 Step 2: Analyze thumbnail");
    
    if video_info.thumbnail.is_empty() {
        println!("❌ No thumbnail URL found");
        return Ok(ThumbnailTestResult {
            url: test_url.to_string(),
            extraction_success: true,
            thumbnail_found: false,
            thumbnail_url: String::new(),
            thumbnail_type: "None".to_string(),
            download_success: false,
            file_size: 0,
            error_message: Some("No thumbnail URL in extraction result".to_string()),
        });
    }
    
    println!("✅ Thumbnail URL found: {}", &video_info.thumbnail[..100.min(video_info.thumbnail.len())]);
    
    let (thumbnail_type, download_success, file_size, error_msg) = if video_info.thumbnail.starts_with("data:") {
        println!("🎯 Thumbnail is a data URL");
        
        if video_info.thumbnail.starts_with("data:image/jpeg;base64,") {
            println!("   📷 JPEG data URL detected");
            match extract_and_save_data_url(&video_info.thumbnail, test_index, "jpg").await {
                Ok(size) => ("data_url_jpeg".to_string(), true, size, None),
                Err(e) => ("data_url_jpeg".to_string(), false, 0, Some(e.to_string())),
            }
        } else if video_info.thumbnail.starts_with("data:image/svg+xml;base64,") {
            println!("   🎨 SVG placeholder detected");
            match extract_and_save_data_url(&video_info.thumbnail, test_index, "svg").await {
                Ok(size) => ("data_url_svg".to_string(), true, size, None),
                Err(e) => ("data_url_svg".to_string(), false, 0, Some(e.to_string())),
            }
        } else {
            println!("   ❓ Unknown data URL format");
            ("data_url_unknown".to_string(), false, 0, Some("Unknown data URL format".to_string()))
        }
    } else {
        println!("🌐 Thumbnail is a CDN URL");
        println!("   Attempting download with multiple strategies...");
        
        match download_cdn_thumbnail(&video_info.thumbnail, test_index).await {
            Ok(size) => {
                println!("   ✅ Download successful ({} bytes)", size);
                ("cdn_url".to_string(), true, size, None)
            }
            Err(e) => {
                println!("   ❌ Download failed: {}", e);
                ("cdn_url".to_string(), false, 0, Some(e.to_string()))
            }
        }
    };
    
    Ok(ThumbnailTestResult {
        url: test_url.to_string(),
        extraction_success: true,
        thumbnail_found: true,
        thumbnail_url: video_info.thumbnail,
        thumbnail_type,
        download_success,
        file_size,
        error_message: error_msg,
    })
}

async fn extract_and_save_data_url(data_url: &str, test_index: usize, extension: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let base64_part = if let Some(part) = data_url.split(',').nth(1) {
        part
    } else {
        return Err("Invalid data URL format".into());
    };
    
    let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(base64_part)?;
    let filename = format!("test_{}_extracted.{}", test_index, extension);
    
    fs::write(&filename, &decoded_bytes)?;
    println!("   💾 Saved to: {}", filename);
    
    Ok(decoded_bytes.len())
}

async fn download_cdn_thumbnail(thumbnail_url: &str, test_index: usize) -> Result<usize, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    // Strategy 1: Enhanced headers
    println!("   🔄 Strategy 1: Enhanced Facebook headers");
    if let Ok(bytes) = try_download_with_headers(&client, thumbnail_url, true).await {
        let filename = format!("test_{}_downloaded.jpg", test_index);
        fs::write(&filename, &bytes)?;
        println!("   💾 Saved to: {}", filename);
        return Ok(bytes.len());
    }
    
    // Strategy 2: Mobile headers
    println!("   🔄 Strategy 2: Mobile user agent");
    if let Ok(bytes) = try_download_with_headers(&client, thumbnail_url, false).await {
        let filename = format!("test_{}_downloaded.jpg", test_index);
        fs::write(&filename, &bytes)?;
        println!("   💾 Saved to: {}", filename);
        return Ok(bytes.len());
    }
    
    // Strategy 3: Minimal headers
    println!("   🔄 Strategy 3: Minimal headers");
    let response = client.get(thumbnail_url).send().await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }
    
    let bytes = response.bytes().await?;
    let filename = format!("test_{}_downloaded.jpg", test_index);
    fs::write(&filename, &bytes)?;
    println!("   💾 Saved to: {}", filename);
    
    Ok(bytes.len())
}

async fn try_download_with_headers(client: &reqwest::Client, url: &str, enhanced: bool) -> Result<bytes::Bytes, Box<dyn std::error::Error>> {
    let mut request = client.get(url);
    
    if enhanced {
        request = request
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://www.facebook.com/")
            .header("Origin", "https://www.facebook.com")
            .header("Sec-Fetch-Dest", "image")
            .header("Sec-Fetch-Mode", "no-cors")
            .header("Sec-Fetch-Site", "cross-site");
    } else {
        request = request
            .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")
            .header("Accept", "image/*,*/*;q=0.8");
    }
    
    let response = request.send().await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }
    
    Ok(response.bytes().await?)
}

async fn generate_test_report(results: &[ThumbnailTestResult]) -> Result<(), Box<dyn std::error::Error>> {
    let mut html = String::from(r#"<!DOCTYPE html>
<html>
<head>
    <title>Facebook Thumbnail Extraction Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .test-case { margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 8px; }
        .success { border-left: 4px solid #4CAF50; }
        .partial { border-left: 4px solid #FF9800; }
        .failed { border-left: 4px solid #f44336; }
        .url { word-break: break-all; font-family: monospace; background: #f5f5f5; padding: 5px; }
        .thumbnail { max-width: 300px; margin: 10px 0; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .status-success { color: #4CAF50; font-weight: bold; }
        .status-failed { color: #f44336; font-weight: bold; }
        .status-partial { color: #FF9800; font-weight: bold; }
    </style>
</head>
<body>
    <h1>🧪 Facebook Thumbnail Extraction Test Report</h1>
    <p>Comprehensive test results for thumbnail extraction from Facebook videos.</p>
    
    <h2>📊 Summary</h2>
    <table>
        <tr><th>Metric</th><th>Count</th><th>Percentage</th></tr>
"#);

    let total = results.len();
    let successful_extractions = results.iter().filter(|r| r.extraction_success).count();
    let thumbnails_found = results.iter().filter(|r| r.thumbnail_found).count();
    let successful_downloads = results.iter().filter(|r| r.download_success).count();

    html.push_str(&format!(r#"
        <tr><td>Total Tests</td><td>{}</td><td>100%</td></tr>
        <tr><td>Successful Extractions</td><td>{}</td><td>{:.1}%</td></tr>
        <tr><td>Thumbnails Found</td><td>{}</td><td>{:.1}%</td></tr>
        <tr><td>Successful Downloads</td><td>{}</td><td>{:.1}%</td></tr>
    </table>
    
    <h2>📋 Detailed Results</h2>
"#, 
        total, 
        successful_extractions, 
        (successful_extractions as f64 / total as f64) * 100.0,
        thumbnails_found,
        (thumbnails_found as f64 / total as f64) * 100.0,
        successful_downloads,
        (successful_downloads as f64 / total as f64) * 100.0
    ));

    for (i, result) in results.iter().enumerate() {
        let class = if result.download_success {
            "success"
        } else if result.thumbnail_found {
            "partial"
        } else {
            "failed"
        };

        html.push_str(&format!(r#"
    <div class="test-case {}">
        <h3>Test Case {} - {}</h3>
        <p><strong>URL:</strong></p>
        <div class="url">{}</div>
        <p><strong>Extraction:</strong> <span class="status-{}">{}</span></p>
        <p><strong>Thumbnail Found:</strong> <span class="status-{}">{}</span></p>
        <p><strong>Thumbnail Type:</strong> {}</p>
        <p><strong>Download:</strong> <span class="status-{}">{}</span></p>
        <p><strong>File Size:</strong> {} bytes</p>
"#, 
            class,
            i + 1,
            if result.download_success { "SUCCESS" } else if result.thumbnail_found { "PARTIAL" } else { "FAILED" },
            result.url,
            if result.extraction_success { "success" } else { "failed" },
            if result.extraction_success { "SUCCESS" } else { "FAILED" },
            if result.thumbnail_found { "success" } else { "failed" },
            if result.thumbnail_found { "YES" } else { "NO" },
            result.thumbnail_type,
            if result.download_success { "success" } else { "failed" },
            if result.download_success { "SUCCESS" } else { "FAILED" },
            result.file_size
        ));

        if let Some(error) = &result.error_message {
            html.push_str(&format!("<p><strong>Error:</strong> <span style='color: red;'>{}</span></p>", error));
        }

        html.push_str("    </div>");
    }

    html.push_str(r#"
</body>
</html>"#);

    fs::write("thumbnail_test_report.html", html)?;
    Ok(())
}
