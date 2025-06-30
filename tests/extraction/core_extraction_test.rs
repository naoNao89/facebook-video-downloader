//! # Core Facebook Video Extraction Test
//!
//! ## Purpose
//! Simplified, focused test for core Facebook video extraction functionality.
//! This replaces the basic extraction tests with a clean, maintainable approach.
//!
//! ## Features
//! - Core extraction functionality testing
//! - URL validation and video ID extraction
//! - Basic HTML parsing and stream detection
//! - Simple error handling and reporting
//!
//! ## Usage
//! ```bash
//! cargo run --bin core_extraction_test [URL]
//! ```

mod common;

use common::*;
use std::env;

/// Core extraction test runner
pub struct CoreExtractionTest {
    client: reqwest::Client,
    config: TestConfig,
}

impl CoreExtractionTest {
    /// Create a new core extraction test
    pub fn new() -> TestResult<Self> {
        let client = create_test_client(None)?;
        let config = TestConfig {
            save_debug_files: true,
            verbose_logging: true,
            ..Default::default()
        };

        Ok(Self { client, config })
    }

    /// Run core extraction test for a URL
    pub async fn test_extraction(&self, url: &str) -> TestResult<VideoInfo> {
        println!("🧪 CORE FACEBOOK VIDEO EXTRACTION TEST");
        println!("======================================");
        println!("🎯 Testing URL: {}", url);
        println!();

        // Step 1: URL Validation
        println!("📋 Step 1: URL Validation");
        println!("-------------------------");
        
        if !is_valid_facebook_url(url) {
            println!("❌ Invalid Facebook URL format");
            return Err(ExtractionTestError::InvalidUrl(url.to_string()));
        }
        println!("✅ URL format is valid");

        // Step 2: Video ID Extraction
        println!("\n🆔 Step 2: Video ID Extraction");
        println!("------------------------------");
        
        let video_id = extract_video_id(url)?;
        println!("✅ Video ID extracted: {}", video_id);

        // Step 3: HTML Fetch and Analysis
        println!("\n📡 Step 3: HTML Fetch and Analysis");
        println!("----------------------------------");
        
        let response = self.client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await?;

        println!("📊 Response status: {}", response.status());
        
        if !response.status().is_success() {
            return Err(ExtractionTestError::NetworkError(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }

        let html = response.text().await?;
        println!("📄 HTML length: {} characters", html.len());

        // Save debug HTML
        save_debug_html(&html, "debug_core_extraction.html", &self.config)?;

        // Step 4: Authentication and Blocking Detection
        println!("\n🔒 Step 4: Authentication and Blocking Detection");
        println!("------------------------------------------------");
        
        let (auth_detected, blocking_detected, privacy_detected) = check_authentication_status(&html);
        println!("   🔐 Authentication required: {}", auth_detected);
        println!("   🚫 Blocking detected: {}", blocking_detected);
        println!("   🔒 Privacy restrictions: {}", privacy_detected);

        if blocking_detected {
            return Err(ExtractionTestError::RateLimited(
                "Facebook is blocking automated access".to_string()
            ));
        }

        // Step 5: Content Analysis
        println!("\n🔍 Step 5: Content Analysis");
        println!("---------------------------");
        
        let content_indicators = vec![
            ("fbcdn.net", html.contains("fbcdn.net")),
            (".mp4", html.contains(".mp4")),
            ("video", html.contains("video")),
            ("playable_url", html.contains("playable_url")),
            ("browser_native", html.contains("browser_native")),
            ("dash_manifest", html.contains("dash_manifest")),
        ];

        for (indicator, found) in &content_indicators {
            println!("   {} {}: {}", 
                if *found { "✅" } else { "❌" }, 
                indicator, 
                found
            );
        }

        // Step 6: Video URL Extraction
        println!("\n🎬 Step 6: Video URL Extraction");
        println!("-------------------------------");
        
        let video_patterns = get_video_url_patterns();
        let found_urls = extract_urls_from_html(&html, &video_patterns);
        
        println!("   📊 Found {} video URLs", found_urls.len());
        
        if found_urls.is_empty() {
            if auth_detected || privacy_detected {
                return Err(ExtractionTestError::AuthenticationRequired(
                    "Video appears to be private and requires authentication".to_string()
                ));
            } else {
                return Err(ExtractionTestError::StreamAnalysisError(
                    "No video URLs found in HTML response".to_string()
                ));
            }
        }

        // Display found URLs (truncated for readability)
        for (i, url) in found_urls.iter().take(5).enumerate() {
            let display_url = if url.len() > 80 {
                format!("{}...", &url[..80])
            } else {
                url.clone()
            };
            println!("   {}. {}", i + 1, display_url);
        }

        if found_urls.len() > 5 {
            println!("   ... and {} more URLs", found_urls.len() - 5);
        }

        // Step 7: Video Information Assembly
        println!("\n📋 Step 7: Video Information Assembly");
        println!("-------------------------------------");
        
        let title = extract_title_from_html(&html, &video_id);
        println!("   📝 Title: {}", title);

        // Create video qualities from found URLs
        let qualities: Vec<VideoQuality> = found_urls
            .into_iter()
            .enumerate()
            .map(|(i, url)| {
                let mut quality = analyze_video_stream(&url);
                quality.quality = format!("Quality {}", i + 1);
                quality
            })
            .collect();

        println!("   🎬 Created {} video quality entries", qualities.len());

        // Create final video info
        let video_info = VideoInfo {
            title,
            duration: "Unknown duration".to_string(),
            thumbnail: String::new(),
            qualities,
            video_id,
            metadata: VideoMetadata::default(),
            extraction_timestamp: chrono::Utc::now(),
            source_url: url.to_string(),
            privacy_status: if auth_detected || privacy_detected { 
                "Private" 
            } else { 
                "Public" 
            }.to_string(),
            access_method: "Core Direct".to_string(),
        };

        // Step 8: Results Summary
        println!("\n🎉 Step 8: Extraction Results");
        println!("=============================");
        println!("✅ Extraction completed successfully!");
        println!("   📝 Title: {}", video_info.title);
        println!("   🆔 Video ID: {}", video_info.video_id);
        println!("   🔒 Privacy Status: {}", video_info.privacy_status);
        println!("   🎬 Video Qualities: {}", video_info.qualities.len());
        
        // Show quality details
        for (i, quality) in video_info.qualities.iter().take(3).enumerate() {
            println!("      {}. {} ({}x{}) - {:?}", 
                i + 1, 
                quality.quality, 
                quality.width, 
                quality.height,
                quality.stream_type
            );
        }

        if video_info.qualities.len() > 3 {
            println!("      ... and {} more qualities", video_info.qualities.len() - 3);
        }

        println!("\n💡 Next Steps:");
        println!("   1. Use these URLs for video downloading");
        println!("   2. Check debug_core_extraction.html for detailed HTML analysis");
        println!("   3. Verify video accessibility in a browser if extraction failed");

        Ok(video_info)
    }

    /// Test with multiple URLs for comparison
    pub async fn test_multiple_urls(&self) -> TestResult<()> {
        println!("🧪 CORE EXTRACTION TEST - MULTIPLE URLS");
        println!("========================================");
        
        let test_urls = TestUrls::get_working_public_urls();
        let mut results = Vec::new();

        for (i, url) in test_urls.iter().enumerate() {
            println!("\n🎯 Test {}/{}: {}", i + 1, test_urls.len(), url);
            println!("{}", "=".repeat(80));

            match self.test_extraction(url).await {
                Ok(video_info) => {
                    println!("✅ Test {} succeeded", i + 1);
                    results.push((url, true, video_info.qualities.len()));
                }
                Err(e) => {
                    println!("❌ Test {} failed: {}", i + 1, e);
                    results.push((url, false, 0));
                }
            }
        }

        // Summary
        println!("\n📊 MULTIPLE URL TEST SUMMARY");
        println!("============================");
        
        let successful = results.iter().filter(|(_, success, _)| *success).count();
        let total = results.len();
        
        for (i, (url, success, qualities)) in results.iter().enumerate() {
            let status = if *success { "✅" } else { "❌" };
            println!("{}. {} {} (qualities: {})", i + 1, status, url, qualities);
        }
        
        println!("\n📈 Success Rate: {}/{} ({:.1}%)", 
            successful, 
            total, 
            (successful as f64 / total as f64) * 100.0
        );

        Ok(())
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    
    // Create test instance
    let test = CoreExtractionTest::new()?;

    if let Some(url) = args.get(1) {
        // Test single URL
        match test.test_extraction(url).await {
            Ok(_) => println!("\n🎉 Core extraction test completed successfully!"),
            Err(e) => {
                println!("\n❌ Core extraction test failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Test multiple URLs
        println!("ℹ️  No URL provided, testing with known working URLs...");
        match test.test_multiple_urls().await {
            Ok(_) => println!("\n🎉 Multiple URL tests completed!"),
            Err(e) => {
                println!("\n❌ Multiple URL tests failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
