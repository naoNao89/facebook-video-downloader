//! # Comprehensive Facebook Thumbnail Extraction Strategy Testing
//!
//! ## Purpose
//! This CLI test suite systematically tests all possible methods for extracting
//! Facebook video thumbnails to identify the most reliable approach. It evaluates
//! multiple extraction strategies across different Facebook URL formats and provides
//! detailed analysis reports.
//!
//! ## Scope
//! - Tests core library extraction methods
//! - Evaluates CDN download strategies with different headers
//! - Tests fallback placeholder generation
//! - Analyzes effectiveness across multiple URL formats
//! - Generates comprehensive HTML and JSON reports
//!
//! ## Usage
//! ```bash
//! # Run the comprehensive test suite
//! cargo run --bin test_comprehensive_thumbnail_strategies
//!
//! # The test will automatically:
//! # 1. Test multiple Facebook video URLs
//! # 2. Try all extraction strategies for each URL
//! # 3. Generate detailed reports and save thumbnail files
//! # 4. Provide recommendations for implementation
//! ```
//!
//! ## Generated Files
//! - `comprehensive_thumbnail_test_results.json` - Raw test data
//! - `comprehensive_thumbnail_test_report.html` - Detailed visual report
//! - `strategy_effectiveness_analysis.html` - Strategy comparison
//! - `thumbnail_*_*.jpg/png/svg` - Extracted thumbnail files
//!
//! ## Dependencies
//! - facebook-extractor-core: Core extraction functionality
//! - reqwest: HTTP client for CDN downloads
//! - base64: Data URL encoding/decoding
//! - serde_json: JSON report generation
//! - chrono: Timestamp handling
//!
//! ## Test Strategy
//! 1. **Core Library Extraction**: Tests the main extraction method
//! 2. **CDN Download Strategies**: Tests different HTTP headers for CDN access
//! 3. **Fallback Generation**: Tests SVG placeholder creation
//! 4. **Cross-URL Analysis**: Evaluates strategies across different URL formats
//!
//! ## Expected Behavior
//! - Should identify the most reliable thumbnail extraction method
//! - Should provide fallback options when primary methods fail
//! - Should generate actionable recommendations for implementation
//! - Should save actual thumbnail files for visual verification

use facebook_extractor_core::FacebookExtractor;
use std::fs;
use std::time::Duration;
use reqwest;
use base64::Engine;
use serde_json;
use chrono::{DateTime, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 COMPREHENSIVE FACEBOOK THUMBNAIL EXTRACTION STRATEGY TEST");
    println!("============================================================");
    println!("🎯 Goal: Identify the most reliable thumbnail extraction method");
    println!("📊 Testing all strategies across multiple URL formats");
    println!();

    // Initialize test environment
    let test_session = TestSession::new();
    
    // Define comprehensive test URLs covering different Facebook formats
    // Using the real URLs that were working in your original test
    let test_urls = vec![
        TestUrl::new("https://www.facebook.com/watch?v=4171119853175259", "watch_direct_param_real"),
        TestUrl::new("https://www.facebook.com/watch/?v=900186485344685", "watch_query_param"),
        TestUrl::new("https://www.facebook.com/watch?v=999173769094576", "watch_direct_param"),
        TestUrl::new("https://www.facebook.com/watch?v=718400947385071", "watch_direct_param_2"),
        // Note: Commenting out fake URLs that will always fail
        // TestUrl::new("https://www.facebook.com/videos/1234567890123456", "videos_format"),
        // TestUrl::new("https://fb.watch/abc123def456", "fb_watch_short"),
        // TestUrl::new("https://www.facebook.com/reel/1234567890123456", "reel_format"),
    ];

    println!("📋 Testing {} URL formats", test_urls.len());
    println!();

    let mut all_results = Vec::new();

    // Test each URL with all extraction strategies
    for (url_index, test_url) in test_urls.iter().enumerate() {
        println!("🔍 URL Test {}/{}: {}", url_index + 1, test_urls.len(), test_url.description);
        println!("   URL: {}", test_url.url);
        println!("   {}", "=".repeat(80));

        let url_results = test_all_strategies_for_url(test_url, url_index + 1).await?;
        all_results.extend(url_results);
        
        println!();
    }

    // Generate comprehensive analysis
    generate_comprehensive_report(&all_results, &test_session).await?;
    
    // Print summary
    print_test_summary(&all_results);
    
    Ok(())
}

#[derive(Debug, Clone)]
struct TestUrl {
    url: String,
    description: String,
}

impl TestUrl {
    fn new(url: &str, description: &str) -> Self {
        Self {
            url: url.to_string(),
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct TestSession {
    start_time: DateTime<Utc>,
    session_id: String,
}

impl TestSession {
    fn new() -> Self {
        Self {
            start_time: Utc::now(),
            session_id: format!("test_{}", Utc::now().timestamp()),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct StrategyTestResult {
    url: String,
    url_type: String,
    strategy_name: String,
    strategy_type: String, // "extraction", "cdn_download", "fallback_generation"
    success: bool,
    thumbnail_url: String,
    thumbnail_type: String, // "data_url_jpeg", "data_url_svg", "cdn_url", "none"
    file_size: usize,
    response_code: Option<u16>,
    error_message: Option<String>,
    execution_time_ms: u64,
    saved_filename: Option<String>,
}

async fn test_all_strategies_for_url(test_url: &TestUrl, url_index: usize) -> Result<Vec<StrategyTestResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    // Strategy 1: Core Library Extraction
    println!("📋 Strategy 1: Core Library Extraction");
    println!("--------------------------------------");
    
    let start_time = std::time::Instant::now();
    let extractor = FacebookExtractor::new()?;
    
    match extractor.extract_video_info(&test_url.url).await {
        Ok(video_info) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("✅ Core extraction successful ({} ms)", execution_time);
            println!("   Title: {}", video_info.title);
            println!("   Video ID: {}", video_info.video_id);
            
            if !video_info.thumbnail.is_empty() {
                println!("   Thumbnail: {}", &video_info.thumbnail[..100.min(video_info.thumbnail.len())]);
                
                // Analyze what type of thumbnail we got
                let (thumbnail_type, file_size, saved_filename) = if video_info.thumbnail.starts_with("data:") {
                    analyze_and_save_data_url(&video_info.thumbnail, url_index, "core_extraction").await?
                } else {
                    ("cdn_url".to_string(), 0, None)
                };
                
                results.push(StrategyTestResult {
                    url: test_url.url.clone(),
                    url_type: test_url.description.clone(),
                    strategy_name: "Core Library Extraction".to_string(),
                    strategy_type: "extraction".to_string(),
                    success: true,
                    thumbnail_url: video_info.thumbnail.clone(),
                    thumbnail_type,
                    file_size,
                    response_code: None,
                    error_message: None,
                    execution_time_ms: execution_time,
                    saved_filename,
                });
                
                // If we got a CDN URL, test downloading it with different strategies
                if !video_info.thumbnail.starts_with("data:") {
                    let cdn_results = test_cdn_download_strategies(&video_info.thumbnail, test_url, url_index).await;
                    results.extend(cdn_results);
                }
            } else {
                results.push(StrategyTestResult {
                    url: test_url.url.clone(),
                    url_type: test_url.description.clone(),
                    strategy_name: "Core Library Extraction".to_string(),
                    strategy_type: "extraction".to_string(),
                    success: false,
                    thumbnail_url: String::new(),
                    thumbnail_type: "none".to_string(),
                    file_size: 0,
                    response_code: None,
                    error_message: Some("No thumbnail URL found".to_string()),
                    execution_time_ms: execution_time,
                    saved_filename: None,
                });
            }
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("❌ Core extraction failed: {}", e);
            
            results.push(StrategyTestResult {
                url: test_url.url.clone(),
                url_type: test_url.description.clone(),
                strategy_name: "Core Library Extraction".to_string(),
                strategy_type: "extraction".to_string(),
                success: false,
                thumbnail_url: String::new(),
                thumbnail_type: "none".to_string(),
                file_size: 0,
                response_code: None,
                error_message: Some(e.to_string()),
                execution_time_ms: execution_time,
                saved_filename: None,
            });
        }
    }
    
    // Strategy 2: Fallback Placeholder Generation
    println!("\n📋 Strategy 2: Fallback Placeholder Generation");
    println!("-----------------------------------------------");
    
    let placeholder_result = test_placeholder_generation(test_url, url_index).await?;
    results.push(placeholder_result);
    
    Ok(results)
}

async fn analyze_and_save_data_url(data_url: &str, url_index: usize, strategy: &str) -> Result<(String, usize, Option<String>), Box<dyn std::error::Error>> {
    if let Some(base64_part) = data_url.strip_prefix("data:image/jpeg;base64,") {
        let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(base64_part)?;
        let filename = format!("thumbnail_{}_{}.jpg", url_index, strategy);
        fs::write(&filename, &decoded_bytes)?;
        println!("   💾 Saved JPEG: {} ({} bytes)", filename, decoded_bytes.len());
        Ok(("data_url_jpeg".to_string(), decoded_bytes.len(), Some(filename)))
    } else if let Some(base64_part) = data_url.strip_prefix("data:image/svg+xml;base64,") {
        let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(base64_part)?;
        let filename = format!("thumbnail_{}_{}.svg", url_index, strategy);
        fs::write(&filename, &decoded_bytes)?;
        println!("   💾 Saved SVG: {} ({} bytes)", filename, decoded_bytes.len());
        Ok(("data_url_svg".to_string(), decoded_bytes.len(), Some(filename)))
    } else if let Some(base64_part) = data_url.strip_prefix("data:image/png;base64,") {
        let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(base64_part)?;
        let filename = format!("thumbnail_{}_{}.png", url_index, strategy);
        fs::write(&filename, &decoded_bytes)?;
        println!("   💾 Saved PNG: {} ({} bytes)", filename, decoded_bytes.len());
        Ok(("data_url_png".to_string(), decoded_bytes.len(), Some(filename)))
    } else {
        Ok(("data_url_unknown".to_string(), 0, None))
    }
}

async fn test_cdn_download_strategies(cdn_url: &str, test_url: &TestUrl, url_index: usize) -> Vec<StrategyTestResult> {
    let mut results = Vec::new();

    println!("\n📋 CDN Download Strategies for: {}", &cdn_url[..100.min(cdn_url.len())]);
    println!("   {}", "-".repeat(60));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap();

    // Strategy 2.1: Enhanced Facebook Headers
    println!("   🔄 CDN Strategy 1: Enhanced Facebook Headers");
    let start_time = std::time::Instant::now();
    match try_enhanced_facebook_headers(&client, cdn_url, url_index, "cdn_enhanced").await {
        Ok((size, filename, response_code)) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("   ✅ Enhanced headers succeeded ({} bytes, {} ms)", size, execution_time);
            results.push(StrategyTestResult {
                url: test_url.url.clone(),
                url_type: test_url.description.clone(),
                strategy_name: "CDN Download - Enhanced Facebook Headers".to_string(),
                strategy_type: "cdn_download".to_string(),
                success: true,
                thumbnail_url: cdn_url.to_string(),
                thumbnail_type: "cdn_url".to_string(),
                file_size: size,
                response_code: Some(response_code),
                error_message: None,
                execution_time_ms: execution_time,
                saved_filename: filename,
            });
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("   ❌ Enhanced headers failed: {}", e);
            results.push(StrategyTestResult {
                url: test_url.url.clone(),
                url_type: test_url.description.clone(),
                strategy_name: "CDN Download - Enhanced Facebook Headers".to_string(),
                strategy_type: "cdn_download".to_string(),
                success: false,
                thumbnail_url: cdn_url.to_string(),
                thumbnail_type: "cdn_url".to_string(),
                file_size: 0,
                response_code: None,
                error_message: Some(e.to_string()),
                execution_time_ms: execution_time,
                saved_filename: None,
            });
        }
    }

    // Strategy 2.2: Mobile User Agent
    println!("   📱 CDN Strategy 2: Mobile User Agent");
    let start_time = std::time::Instant::now();
    match try_mobile_user_agent(&client, cdn_url, url_index, "cdn_mobile").await {
        Ok((size, filename, response_code)) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("   ✅ Mobile headers succeeded ({} bytes, {} ms)", size, execution_time);
            results.push(StrategyTestResult {
                url: test_url.url.clone(),
                url_type: test_url.description.clone(),
                strategy_name: "CDN Download - Mobile User Agent".to_string(),
                strategy_type: "cdn_download".to_string(),
                success: true,
                thumbnail_url: cdn_url.to_string(),
                thumbnail_type: "cdn_url".to_string(),
                file_size: size,
                response_code: Some(response_code),
                error_message: None,
                execution_time_ms: execution_time,
                saved_filename: filename,
            });
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("   ❌ Mobile headers failed: {}", e);
            results.push(StrategyTestResult {
                url: test_url.url.clone(),
                url_type: test_url.description.clone(),
                strategy_name: "CDN Download - Mobile User Agent".to_string(),
                strategy_type: "cdn_download".to_string(),
                success: false,
                thumbnail_url: cdn_url.to_string(),
                thumbnail_type: "cdn_url".to_string(),
                file_size: 0,
                response_code: None,
                error_message: Some(e.to_string()),
                execution_time_ms: execution_time,
                saved_filename: None,
            });
        }
    }

    // Strategy 2.3: Minimal Headers
    println!("   🔧 CDN Strategy 3: Minimal Headers");
    let start_time = std::time::Instant::now();
    match try_minimal_headers(&client, cdn_url, url_index, "cdn_minimal").await {
        Ok((size, filename, response_code)) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("   ✅ Minimal headers succeeded ({} bytes, {} ms)", size, execution_time);
            results.push(StrategyTestResult {
                url: test_url.url.clone(),
                url_type: test_url.description.clone(),
                strategy_name: "CDN Download - Minimal Headers".to_string(),
                strategy_type: "cdn_download".to_string(),
                success: true,
                thumbnail_url: cdn_url.to_string(),
                thumbnail_type: "cdn_url".to_string(),
                file_size: size,
                response_code: Some(response_code),
                error_message: None,
                execution_time_ms: execution_time,
                saved_filename: filename,
            });
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("   ❌ Minimal headers failed: {}", e);
            results.push(StrategyTestResult {
                url: test_url.url.clone(),
                url_type: test_url.description.clone(),
                strategy_name: "CDN Download - Minimal Headers".to_string(),
                strategy_type: "cdn_download".to_string(),
                success: false,
                thumbnail_url: cdn_url.to_string(),
                thumbnail_type: "cdn_url".to_string(),
                file_size: 0,
                response_code: None,
                error_message: Some(e.to_string()),
                execution_time_ms: execution_time,
                saved_filename: None,
            });
        }
    }

    results
}

async fn try_enhanced_facebook_headers(client: &reqwest::Client, url: &str, url_index: usize, strategy: &str) -> Result<(usize, Option<String>, u16), Box<dyn std::error::Error>> {
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

    let status_code = response.status().as_u16();

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    if bytes.len() == 0 {
        return Err("Empty response".into());
    }

    let filename = format!("thumbnail_{}_{}.jpg", url_index, strategy);
    fs::write(&filename, &bytes)?;

    Ok((bytes.len(), Some(filename), status_code))
}

async fn try_mobile_user_agent(client: &reqwest::Client, url: &str, url_index: usize, strategy: &str) -> Result<(usize, Option<String>, u16), Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")
        .header("Accept", "image/*,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Connection", "keep-alive")
        .send()
        .await?;

    let status_code = response.status().as_u16();

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    if bytes.len() == 0 {
        return Err("Empty response".into());
    }

    let filename = format!("thumbnail_{}_{}.jpg", url_index, strategy);
    fs::write(&filename, &bytes)?;

    Ok((bytes.len(), Some(filename), status_code))
}

async fn try_minimal_headers(client: &reqwest::Client, url: &str, url_index: usize, strategy: &str) -> Result<(usize, Option<String>, u16), Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;

    let status_code = response.status().as_u16();

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    if bytes.len() == 0 {
        return Err("Empty response".into());
    }

    let filename = format!("thumbnail_{}_{}.jpg", url_index, strategy);
    fs::write(&filename, &bytes)?;

    Ok((bytes.len(), Some(filename), status_code))
}

async fn test_placeholder_generation(test_url: &TestUrl, url_index: usize) -> Result<StrategyTestResult, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    // Extract video ID from URL for placeholder generation
    let video_id = extract_video_id_from_url(&test_url.url);
    let title = format!("Facebook Video {}", &video_id[..8.min(video_id.len())]);

    let placeholder_svg = create_placeholder_thumbnail(&title, &video_id);
    let base64_svg = base64::engine::general_purpose::STANDARD.encode(placeholder_svg.as_bytes());
    let data_url = format!("data:image/svg+xml;base64,{}", base64_svg);

    let filename = format!("thumbnail_{}_placeholder.svg", url_index);
    fs::write(&filename, placeholder_svg.as_bytes())?;

    let execution_time = start_time.elapsed().as_millis() as u64;

    println!("✅ Generated SVG placeholder ({} bytes, {} ms)", placeholder_svg.len(), execution_time);
    println!("   💾 Saved to: {}", filename);

    Ok(StrategyTestResult {
        url: test_url.url.clone(),
        url_type: test_url.description.clone(),
        strategy_name: "Fallback Placeholder Generation".to_string(),
        strategy_type: "fallback_generation".to_string(),
        success: true,
        thumbnail_url: data_url,
        thumbnail_type: "data_url_svg".to_string(),
        file_size: placeholder_svg.len(),
        response_code: None,
        error_message: None,
        execution_time_ms: execution_time,
        saved_filename: Some(filename),
    })
}

fn extract_video_id_from_url(url: &str) -> String {
    // Extract video ID from various Facebook URL formats
    if let Some(captures) = regex::Regex::new(r"[?&]v=([^&]+)").unwrap().captures(url) {
        captures.get(1).unwrap().as_str().to_string()
    } else if let Some(captures) = regex::Regex::new(r"/videos/(\d+)").unwrap().captures(url) {
        captures.get(1).unwrap().as_str().to_string()
    } else if let Some(captures) = regex::Regex::new(r"/reel/(\d+)").unwrap().captures(url) {
        captures.get(1).unwrap().as_str().to_string()
    } else if let Some(captures) = regex::Regex::new(r"fb\.watch/([^/?]+)").unwrap().captures(url) {
        captures.get(1).unwrap().as_str().to_string()
    } else {
        "unknown".to_string()
    }
}

fn create_placeholder_thumbnail(title: &str, video_id: &str) -> String {
    let truncated_title = if title.len() > 50 {
        format!("{}...", &title[..47])
    } else {
        title.to_string()
    };

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
    svg_content.push_str(&format!(r#"<text x="160" y="130" font-family="Arial" font-size="12" fill="white" text-anchor="middle">{}</text>"#, truncated_title));
    svg_content.push_str(&format!(r#"<text x="160" y="150" font-family="Arial" font-size="10" fill="white" text-anchor="middle" opacity="0.8">ID: {}</text>"#, &video_id[..12.min(video_id.len())]));
    svg_content.push_str(r#"</svg>"#);
    svg_content
}

async fn generate_comprehensive_report(results: &[StrategyTestResult], session: &TestSession) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Generating comprehensive analysis report...");

    // Generate JSON report for programmatic analysis
    let json_report = serde_json::to_string_pretty(results)?;
    fs::write("comprehensive_thumbnail_test_results.json", json_report)?;

    // Generate detailed HTML report
    generate_html_report(results, session).await?;

    // Generate strategy effectiveness analysis
    generate_strategy_analysis(results).await?;

    println!("✅ Reports generated:");
    println!("   📄 comprehensive_thumbnail_test_results.json - Raw test data");
    println!("   🌐 comprehensive_thumbnail_test_report.html - Detailed visual report");
    println!("   📊 strategy_effectiveness_analysis.html - Strategy comparison");

    Ok(())
}

async fn generate_html_report(results: &[StrategyTestResult], session: &TestSession) -> Result<(), Box<dyn std::error::Error>> {
    let mut html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Comprehensive Facebook Thumbnail Extraction Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; line-height: 1.6; }}
        .header {{ background: #1877f2; color: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin: 20px 0; }}
        .metric {{ background: #f8f9fa; padding: 15px; border-radius: 8px; text-align: center; }}
        .metric h3 {{ margin: 0 0 10px 0; color: #1877f2; }}
        .metric .value {{ font-size: 2em; font-weight: bold; color: #333; }}
        .test-result {{ margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 8px; }}
        .success {{ border-left: 4px solid #4CAF50; }}
        .failed {{ border-left: 4px solid #f44336; }}
        .strategy-group {{ margin: 20px 0; }}
        .strategy-name {{ font-weight: bold; color: #1877f2; }}
        .url-info {{ background: #f5f5f5; padding: 10px; border-radius: 4px; font-family: monospace; }}
        .thumbnail-preview {{ max-width: 200px; margin: 10px 0; border: 1px solid #ccc; }}
        .error {{ color: #f44336; font-style: italic; }}
        .success-text {{ color: #4CAF50; font-weight: bold; }}
        .failed-text {{ color: #f44336; font-weight: bold; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🧪 Comprehensive Facebook Thumbnail Extraction Test Report</h1>
        <p>Session ID: {} | Started: {} | Total Tests: {}</p>
    </div>
"#, session.session_id, session.start_time.format("%Y-%m-%d %H:%M:%S UTC"), results.len());

    // Calculate summary metrics
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let extraction_tests = results.iter().filter(|r| r.strategy_type == "extraction").count();
    let cdn_download_tests = results.iter().filter(|r| r.strategy_type == "cdn_download").count();
    let fallback_tests = results.iter().filter(|r| r.strategy_type == "fallback_generation").count();

    let successful_extractions = results.iter().filter(|r| r.strategy_type == "extraction" && r.success).count();
    let successful_downloads = results.iter().filter(|r| r.strategy_type == "cdn_download" && r.success).count();
    let successful_fallbacks = results.iter().filter(|r| r.strategy_type == "fallback_generation" && r.success).count();

    html.push_str(&format!(r#"
    <div class="summary">
        <div class="metric">
            <h3>Total Tests</h3>
            <div class="value">{}</div>
        </div>
        <div class="metric">
            <h3>Success Rate</h3>
            <div class="value">{:.1}%</div>
        </div>
        <div class="metric">
            <h3>Extraction Success</h3>
            <div class="value">{}/{}</div>
        </div>
        <div class="metric">
            <h3>Download Success</h3>
            <div class="value">{}/{}</div>
        </div>
        <div class="metric">
            <h3>Fallback Success</h3>
            <div class="value">{}/{}</div>
        </div>
    </div>

    <h2>📋 Detailed Test Results</h2>
"#,
        total_tests,
        (successful_tests as f64 / total_tests as f64) * 100.0,
        successful_extractions, extraction_tests,
        successful_downloads, cdn_download_tests,
        successful_fallbacks, fallback_tests
    ));

    // Group results by URL
    let mut url_groups: std::collections::HashMap<String, Vec<&StrategyTestResult>> = std::collections::HashMap::new();
    for result in results {
        url_groups.entry(result.url.clone()).or_insert_with(Vec::new).push(result);
    }

    for (url, url_results) in url_groups {
        html.push_str(&format!(r#"
    <div class="test-result">
        <h3>🎯 URL: {}</h3>
        <div class="url-info">{}</div>
        <p><strong>URL Type:</strong> {}</p>
        <p><strong>Strategies Tested:</strong> {}</p>

        <div class="strategy-group">
"#,
            url_results[0].url_type,
            url,
            url_results[0].url_type,
            url_results.len()
        ));

        for result in url_results {
            let _class = if result.success { "success" } else { "failed" };
            let status_text = if result.success { "SUCCESS" } else { "FAILED" };
            let status_class = if result.success { "success-text" } else { "failed-text" };

            html.push_str(&format!(r#"
            <div class="strategy-result" style="margin: 10px 0; padding: 10px; background: #f9f9f9; border-radius: 4px;">
                <div class="strategy-name">{}</div>
                <p><strong>Status:</strong> <span class="{}">{}</span></p>
                <p><strong>Type:</strong> {}</p>
                <p><strong>Execution Time:</strong> {} ms</p>
                <p><strong>File Size:</strong> {} bytes</p>
"#,
                result.strategy_name,
                status_class, status_text,
                result.thumbnail_type,
                result.execution_time_ms,
                result.file_size
            ));

            if let Some(response_code) = result.response_code {
                html.push_str(&format!("<p><strong>HTTP Status:</strong> {}</p>", response_code));
            }

            if let Some(filename) = &result.saved_filename {
                html.push_str(&format!("<p><strong>Saved File:</strong> {}</p>", filename));
            }

            if let Some(error) = &result.error_message {
                html.push_str(&format!("<p class=\"error\"><strong>Error:</strong> {}</p>", error));
            }

            html.push_str("            </div>");
        }

        html.push_str("        </div>\n    </div>");
    }

    html.push_str(r#"
</body>
</html>"#);

    fs::write("comprehensive_thumbnail_test_report.html", html)?;
    Ok(())
}

async fn generate_strategy_analysis(results: &[StrategyTestResult]) -> Result<(), Box<dyn std::error::Error>> {
    let mut analysis = String::from(r#"<!DOCTYPE html>
<html>
<head>
    <title>Strategy Effectiveness Analysis</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .strategy-analysis { margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 8px; }
        .best-strategy { border-left: 4px solid #4CAF50; background: #f8fff8; }
        .worst-strategy { border-left: 4px solid #f44336; background: #fff8f8; }
        .recommendation { background: #e3f2fd; padding: 15px; border-radius: 8px; margin: 20px 0; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .success-rate { font-weight: bold; }
        .high-success { color: #4CAF50; }
        .medium-success { color: #FF9800; }
        .low-success { color: #f44336; }
    </style>
</head>
<body>
    <h1>📊 Strategy Effectiveness Analysis</h1>
    <p>Analysis of thumbnail extraction strategy performance across different Facebook URL formats.</p>
"#);

    // Calculate strategy effectiveness
    let mut strategy_stats: std::collections::HashMap<String, (usize, usize, u64, usize)> = std::collections::HashMap::new();

    for result in results {
        let entry = strategy_stats.entry(result.strategy_name.clone()).or_insert((0, 0, 0, 0));
        entry.0 += 1; // total attempts
        if result.success {
            entry.1 += 1; // successful attempts
            entry.2 += result.execution_time_ms; // total execution time
            entry.3 += result.file_size; // total file size
        }
    }

    analysis.push_str("<h2>📈 Strategy Performance Summary</h2>\n<table>\n");
    analysis.push_str("<tr><th>Strategy</th><th>Success Rate</th><th>Attempts</th><th>Avg Time (ms)</th><th>Avg Size (bytes)</th></tr>\n");

    let mut sorted_strategies: Vec<_> = strategy_stats.iter().collect();
    sorted_strategies.sort_by(|a, b| {
        let success_rate_a = a.1.1 as f64 / a.1.0 as f64;
        let success_rate_b = b.1.1 as f64 / b.1.0 as f64;
        success_rate_b.partial_cmp(&success_rate_a).unwrap()
    });

    for (strategy_name, (total, successful, total_time, total_size)) in &sorted_strategies {
        let success_rate = (*successful as f64 / *total as f64) * 100.0;
        let avg_time = if *successful > 0 { *total_time / *successful as u64 } else { 0 };
        let avg_size = if *successful > 0 { *total_size / *successful } else { 0 };

        let success_class = if success_rate >= 80.0 {
            "high-success"
        } else if success_rate >= 50.0 {
            "medium-success"
        } else {
            "low-success"
        };

        analysis.push_str(&format!(
            "<tr><td>{}</td><td class=\"success-rate {}\">{}% ({}/{})</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            strategy_name, success_class, success_rate as u32, successful, total, total, avg_time, avg_size
        ));
    }

    analysis.push_str("</table>\n");

    // Find best and worst strategies
    if let Some((best_strategy, best_stats)) = sorted_strategies.first() {
        let best_success_rate = (best_stats.1 as f64 / best_stats.0 as f64) * 100.0;
        analysis.push_str(&format!(r#"
<div class="strategy-analysis best-strategy">
    <h3>🏆 Most Effective Strategy</h3>
    <p><strong>{}</strong></p>
    <p>Success Rate: {:.1}% ({}/{} attempts)</p>
    <p>Average Execution Time: {} ms</p>
    <p>Average File Size: {} bytes</p>
</div>
"#, best_strategy, best_success_rate, best_stats.1, best_stats.0,
    if best_stats.1 > 0 { best_stats.2 / best_stats.1 as u64 } else { 0 },
    if best_stats.1 > 0 { best_stats.3 / best_stats.1 } else { 0 }));
    }

    if let Some((worst_strategy, worst_stats)) = sorted_strategies.last() {
        let worst_success_rate = (worst_stats.1 as f64 / worst_stats.0 as f64) * 100.0;
        analysis.push_str(&format!(r#"
<div class="strategy-analysis worst-strategy">
    <h3>⚠️ Least Effective Strategy</h3>
    <p><strong>{}</strong></p>
    <p>Success Rate: {:.1}% ({}/{} attempts)</p>
    <p>This strategy may need improvement or should be avoided.</p>
</div>
"#, worst_strategy, worst_success_rate, worst_stats.1, worst_stats.0));
    }

    // Generate recommendations
    analysis.push_str(r#"
<div class="recommendation">
    <h3>💡 Implementation Recommendations</h3>
    <ol>
        <li><strong>Primary Strategy:</strong> Use the most effective strategy identified above as the primary method.</li>
        <li><strong>Fallback Chain:</strong> Implement a fallback chain starting with the highest success rate strategies.</li>
        <li><strong>Performance Optimization:</strong> Consider execution time vs success rate trade-offs for user experience.</li>
        <li><strong>Error Handling:</strong> Always implement placeholder generation as the final fallback.</li>
        <li><strong>Caching:</strong> Cache successful thumbnail URLs to avoid repeated downloads.</li>
    </ol>
</div>
"#);

    // URL format analysis
    let mut url_format_stats: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();

    for result in results.iter().filter(|r| r.strategy_type == "extraction") {
        let entry = url_format_stats.entry(result.url_type.clone()).or_insert((0, 0));
        entry.0 += 1;
        if result.success {
            entry.1 += 1;
        }
    }

    analysis.push_str("<h2>🔗 URL Format Analysis</h2>\n<table>\n");
    analysis.push_str("<tr><th>URL Format</th><th>Extraction Success Rate</th><th>Attempts</th></tr>\n");

    for (url_type, (total, successful)) in url_format_stats {
        let success_rate = (successful as f64 / total as f64) * 100.0;
        let success_class = if success_rate >= 80.0 {
            "high-success"
        } else if success_rate >= 50.0 {
            "medium-success"
        } else {
            "low-success"
        };

        analysis.push_str(&format!(
            "<tr><td>{}</td><td class=\"success-rate {}\">{:.1}% ({}/{})</td><td>{}</td></tr>\n",
            url_type, success_class, success_rate, successful, total, total
        ));
    }

    analysis.push_str("</table>\n");

    analysis.push_str("</body>\n</html>");

    fs::write("strategy_effectiveness_analysis.html", analysis)?;
    Ok(())
}

fn print_test_summary(results: &[StrategyTestResult]) {
    println!("\n🎯 COMPREHENSIVE TEST SUMMARY");
    println!("=============================");

    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let extraction_tests = results.iter().filter(|r| r.strategy_type == "extraction").count();
    let cdn_download_tests = results.iter().filter(|r| r.strategy_type == "cdn_download").count();
    let fallback_tests = results.iter().filter(|r| r.strategy_type == "fallback_generation").count();

    let successful_extractions = results.iter().filter(|r| r.strategy_type == "extraction" && r.success).count();
    let successful_downloads = results.iter().filter(|r| r.strategy_type == "cdn_download" && r.success).count();
    let successful_fallbacks = results.iter().filter(|r| r.strategy_type == "fallback_generation" && r.success).count();

    println!("📊 Overall Statistics:");
    println!("   Total Tests: {}", total_tests);
    println!("   Overall Success Rate: {:.1}%", (successful_tests as f64 / total_tests as f64) * 100.0);
    println!();

    println!("📋 Strategy Breakdown:");
    println!("   Extraction Tests: {}/{} ({:.1}%)", successful_extractions, extraction_tests,
             if extraction_tests > 0 { (successful_extractions as f64 / extraction_tests as f64) * 100.0 } else { 0.0 });
    println!("   CDN Download Tests: {}/{} ({:.1}%)", successful_downloads, cdn_download_tests,
             if cdn_download_tests > 0 { (successful_downloads as f64 / cdn_download_tests as f64) * 100.0 } else { 0.0 });
    println!("   Fallback Tests: {}/{} ({:.1}%)", successful_fallbacks, fallback_tests,
             if fallback_tests > 0 { (successful_fallbacks as f64 / fallback_tests as f64) * 100.0 } else { 0.0 });
    println!();

    println!("📁 Generated Files:");
    println!("   📄 comprehensive_thumbnail_test_results.json");
    println!("   🌐 comprehensive_thumbnail_test_report.html");
    println!("   📊 strategy_effectiveness_analysis.html");
    println!("   🖼️ thumbnail_*_*.jpg/png/svg (extracted thumbnails)");
    println!();

    println!("💡 Next Steps:");
    println!("   1. Review the HTML reports for detailed analysis");
    println!("   2. Check the saved thumbnail files to verify quality");
    println!("   3. Implement the most effective strategy in the core extractor");
    println!("   4. Add fallback mechanisms based on the test results");
    println!("   5. Test the updated implementation in the Tauri app");
}
