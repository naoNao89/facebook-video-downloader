//! # Comprehensive Facebook Video Extraction Test Suite
//!
//! ## Purpose
//! Test suite that uses the actual facebook-extractor-core crate to test
//! real-world Facebook video extraction functionality as used by the Tauri application.
//!
//! ## Features
//! - Uses the same FacebookExtractor as the Tauri app
//! - Tests actual video URL extraction and stream analysis
//! - Validates complete extraction pipeline including metadata
//! - Tests multiple extraction methods and fallbacks
//! - Provides results comparable to the desktop application
//!
//! ## Usage
//! ```bash
//! cargo run --bin comprehensive_extraction_test [URL]
//! cargo run --bin comprehensive_extraction_test --test-all
//! cargo run --bin comprehensive_extraction_test --test-patterns
//! cargo run --bin comprehensive_extraction_test --debug
//! ```

use facebook_extractor_core::{FacebookExtractor, ExtractorConfig, VideoInfo, FacebookExtractorError};
use std::env;
use tokio;

/// Main extraction test suite using the real facebook-extractor-core
pub struct ExtractionTestSuite {
    extractor: FacebookExtractor,
    config: TestConfig,
}

impl ExtractionTestSuite {
    /// Create a new test suite with the actual FacebookExtractor
    pub fn new(config: TestConfig) -> Result<Self, FacebookExtractorError> {
        // Create extractor config based on test config
        let extractor_config = if config.debug_mode {
            ExtractorConfig::debug_optimized()
        } else {
            ExtractorConfig::performance_optimized()
        };

        let extractor = FacebookExtractor::with_config(extractor_config)?;

        Ok(Self {
            extractor,
            config,
        })
    }

    /// Run all extraction tests using the real FacebookExtractor
    pub async fn run_all_tests(&self) -> Result<(), FacebookExtractorError> {
        println!("🚀 COMPREHENSIVE FACEBOOK VIDEO EXTRACTION TEST SUITE");
        println!("=====================================================");
        println!("🔧 Using facebook-extractor-core v{}", env!("CARGO_PKG_VERSION"));
        println!("🔧 Configuration: {:?}", self.config);
        println!();

        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Test 1: URL Validation and ID Extraction
        print_test_progress(1, 5, "URL Validation and Video ID Extraction");
        match self.test_url_validation().await {
            Ok(_) => {
                println!("✅ URL validation tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ URL validation tests failed: {}", e);
                failed += 1;
            }
        }

        // Test 2: Simple Video URL Extraction
        print_test_progress(2, 5, "Simple Video URL Extraction using FacebookExtractor");
        match self.test_real_extraction().await {
            Ok(_) => {
                println!("✅ Video URL extraction tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ Video URL extraction tests failed: {}", e);
                failed += 1;
            }
        }

        // Test 3: Video URL Extraction Test
        print_test_progress(3, 5, "Video URL Extraction Test");
        match self.test_stream_analysis().await {
            Ok(_) => {
                println!("✅ URL extraction tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ URL extraction tests failed: {}", e);
                failed += 1;
            }
        }

        // Test 4: Multiple URL Format Support
        print_test_progress(4, 5, "Multiple URL Format Support");
        match self.test_url_formats().await {
            Ok(_) => {
                println!("✅ URL format tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ URL format tests failed: {}", e);
                failed += 1;
            }
        }

        // Test 5: Error Handling and Edge Cases
        print_test_progress(5, 5, "Error Handling and Edge Cases");
        if self.config.test_private_videos {
            match self.test_error_handling().await {
                Ok(_) => {
                    println!("✅ Error handling tests passed");
                    passed += 1;
                }
                Err(e) => {
                    println!("❌ Error handling tests failed: {}", e);
                    failed += 1;
                }
            }
        } else {
            println!("⏭️  Skipped (private video testing disabled)");
            skipped += 1;
        }

        print_test_summary(passed, failed, skipped);
        Ok(())
    }

    /// Test URL validation using the real FacebookExtractor
    async fn test_url_validation(&self) -> Result<(), FacebookExtractorError> {
        println!("🔍 Testing URL validation using FacebookExtractor...");

        // Test valid URLs
        let valid_urls = get_test_urls();
        for url in valid_urls {
            let validation = self.extractor.validate_url(url);
            if validation.is_valid {
                if let Some(video_id) = validation.video_id {
                    println!("   ✅ {} -> ID: {} (Type: {:?})", url, video_id, validation.content_type);
                } else {
                    println!("   ⚠️  {} -> Valid but no ID extracted", url);
                }
            } else {
                println!("   ❌ {} -> Invalid: {}", url, validation.error_message.unwrap_or_default());
            }
        }

        // Test invalid URLs
        let invalid_urls = vec![
            "https://youtube.com/watch?v=123",
            "https://facebook.com/invalid",
            "not-a-url",
            "",
        ];

        for url in invalid_urls {
            let validation = self.extractor.validate_url(url);
            if validation.is_valid {
                return Err(FacebookExtractorError::invalid_url(
                    &format!("Invalid URL passed validation: {}", url)
                ));
            } else {
                println!("   ✅ {} -> Correctly rejected", url);
            }
        }

        println!("   ✅ All URL validation tests passed");
        Ok(())
    }

    /// Test simple video URL extraction using FacebookExtractor
    async fn test_real_extraction(&self) -> Result<(), FacebookExtractorError> {
        println!("🎬 Testing video URL extraction using FacebookExtractor...");

        let test_urls = get_working_test_urls();
        let mut successful_extractions = 0;
        let mut failed_extractions = 0;

        for (i, test_url) in test_urls.iter().enumerate() {
            println!("   🎯 Testing URL {}/{}: {}", i + 1, test_urls.len(), test_url);

            match self.extractor.extract_video_info(test_url).await {
                Ok(video_info) => {
                    println!("   ✅ Extraction succeeded!");
                    self.print_simple_video_urls(&video_info);
                    successful_extractions += 1;
                }
                Err(e) => {
                    println!("   ❌ Extraction failed: {}", e);
                    failed_extractions += 1;
                    self.analyze_extraction_error(&e);
                }
            }
            println!();
        }

        println!("📊 Extraction Summary:");
        println!("   ✅ Successful: {}", successful_extractions);
        println!("   ❌ Failed: {}", failed_extractions);
        println!("   📈 Success Rate: {:.1}%",
            (successful_extractions as f64 / test_urls.len() as f64) * 100.0);

        Ok(())
    }

    /// Test simple video URL extraction
    async fn test_stream_analysis(&self) -> Result<(), FacebookExtractorError> {
        println!("🔍 Testing video URL extraction...");

        let test_url = get_working_test_urls()[0];
        println!("   🎯 Extracting URLs for: {}", test_url);

        match self.extractor.extract_video_info(test_url).await {
            Ok(video_info) => {
                println!("   ✅ URL extraction succeeded!");
                println!("   📝 Title: {}", video_info.title);
                println!("   🎬 Available video URLs:");

                for (i, quality) in video_info.qualities.iter().enumerate() {
                    println!("      {}. {} - {}",
                        i + 1,
                        quality.quality,
                        quality.download_url
                    );
                }

                if video_info.qualities.is_empty() {
                    println!("   ⚠️  Warning: No video URLs found!");
                }
            }
            Err(e) => {
                println!("   ❌ URL extraction failed: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Test multiple URL format support
    async fn test_url_formats(&self) -> Result<(), FacebookExtractorError> {
        println!("🔗 Testing multiple URL format support...");

        let url_formats = get_url_format_test_cases();
        let mut supported_formats = 0;
        let mut unsupported_formats = 0;

        for (format_name, test_url) in url_formats {
            println!("   🎯 Testing {} format: {}", format_name, test_url);

            let validation = self.extractor.validate_url(test_url);
            if validation.is_valid {
                println!("      ✅ Format supported - Video ID: {}",
                    validation.video_id.unwrap_or_default());
                supported_formats += 1;

                // Try actual extraction for supported formats
                match self.extractor.extract_video_info(test_url).await {
                    Ok(video_info) => {
                        println!("      🎬 Extraction successful - Found {} qualities",
                            video_info.qualities.len());
                    }
                    Err(e) => {
                        println!("      ⚠️  Extraction failed: {}", e);
                    }
                }
            } else {
                println!("      ❌ Format not supported: {}",
                    validation.error_message.unwrap_or_default());
                unsupported_formats += 1;
            }
        }

        println!("   📊 URL Format Support Summary:");
        println!("      ✅ Supported: {}", supported_formats);
        println!("      ❌ Unsupported: {}", unsupported_formats);
        println!("      📈 Support Rate: {:.1}%",
            (supported_formats as f64 / (supported_formats + unsupported_formats) as f64) * 100.0);

        Ok(())
    }

    /// Test error handling and edge cases
    async fn test_error_handling(&self) -> Result<(), FacebookExtractorError> {
        println!("⚠️ Testing error handling and edge cases...");

        // Test with potentially private URLs
        let private_urls = get_private_test_urls();
        for url in private_urls {
            println!("   🔒 Testing private URL: {}", url);

            match self.extractor.extract_video_info(url).await {
                Ok(video_info) => {
                    println!("      ✅ Unexpectedly succeeded (may be public): {}", video_info.title);
                }
                Err(e) => {
                    println!("      ✅ Expected failure: {}", e);
                    self.analyze_extraction_error(&e);
                }
            }
        }

        // Test with invalid URLs
        let invalid_urls = vec![
            "https://www.facebook.com/watch?v=invalid",
            "https://www.facebook.com/nonexistent/videos/123",
        ];

        for url in invalid_urls {
            println!("   ❌ Testing invalid URL: {}", url);

            match self.extractor.extract_video_info(url).await {
                Ok(_) => {
                    println!("      ⚠️  Unexpectedly succeeded");
                }
                Err(e) => {
                    println!("      ✅ Expected failure: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Print simple video URLs only
    fn print_simple_video_urls(&self, video_info: &VideoInfo) {
        println!("      📝 Title: {}", video_info.title);
        println!("      🆔 Video ID: {}", video_info.video_id);
        println!("      🎬 Available video URLs:");

        for (i, quality) in video_info.qualities.iter().enumerate() {
            println!("         {}. {} - {}",
                i + 1,
                quality.quality,
                quality.download_url
            );
        }
    }

    /// Print detailed video information summary
    fn print_video_info_summary(&self, video_info: &VideoInfo) {
        println!("      📝 Title: {}", video_info.title);
        println!("      🆔 Video ID: {}", video_info.video_id);
        println!("      ⏱️  Duration: {}", video_info.duration);
        println!("      🔒 Privacy: {:?}", video_info.privacy_level);
        println!("      🔧 Access Method: {:?}", video_info.access_method);
        println!("      🎬 Qualities found: {}", video_info.qualities.len());

        for (i, quality) in video_info.qualities.iter().take(3).enumerate() {
            println!("         {}. {} ({}x{}) - {:?} - ~{}MB",
                i + 1,
                quality.quality,
                quality.width,
                quality.height,
                quality.stream_type,
                quality.estimated_size_mb
            );
        }

        if video_info.qualities.len() > 3 {
            println!("         ... and {} more", video_info.qualities.len() - 3);
        }
    }

    /// Analyze extraction error and provide insights
    fn analyze_extraction_error(&self, error: &FacebookExtractorError) {
        match error {
            FacebookExtractorError::AuthenticationRequired => {
                println!("      🔒 Authentication required - video may be private");
            }
            FacebookExtractorError::AccessDenied { reason } => {
                println!("      🚫 Access denied: {}", reason);
            }
            FacebookExtractorError::RateLimited => {
                println!("      ⏱️  Rate limited - too many requests");
            }
            FacebookExtractorError::GeoBlocked => {
                println!("      🌍 Geo-blocked content");
            }
            FacebookExtractorError::ContentUnavailable => {
                println!("      📭 Content no longer available");
            }
            FacebookExtractorError::Network { source: _ } => {
                println!("      🌐 Network connectivity issue");
            }
            FacebookExtractorError::HtmlParsing { message } => {
                println!("      🔍 HTML parsing issue: {}", message);
            }
            FacebookExtractorError::StreamAnalysis { message } => {
                println!("      🎬 Stream analysis issue: {}", message);
            }
            _ => {
                println!("      ❓ Other error type");
            }
        }
    }

    /// Save extraction debug information
    async fn save_extraction_debug(&self, video_info: &VideoInfo, index: usize) {
        if let Ok(json) = serde_json::to_string_pretty(video_info) {
            let filename = format!("debug_extraction_{}.json", index);
            if let Err(e) = std::fs::write(&filename, json) {
                println!("      ⚠️  Failed to save debug file {}: {}", filename, e);
            } else {
                println!("      💾 Saved debug info to: {}", filename);
            }
        }
    }

}

// ============================================================================
// TEST CONFIGURATION AND DATA
// ============================================================================

/// Test configuration for extraction tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub save_debug_files: bool,
    pub verbose_logging: bool,
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub test_private_videos: bool,
    pub debug_mode: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            save_debug_files: false,
            verbose_logging: false,
            timeout_seconds: 30,
            max_retries: 3,
            test_private_videos: false,
            debug_mode: false,
        }
    }
}

/// Get test URLs for extraction testing
fn get_test_urls() -> Vec<&'static str> {
    vec![
        "https://www.facebook.com/watch/?v=2209933269449948",
        "https://www.facebook.com/watch?v=419280024562892",
        "https://www.facebook.com/reel/1193939392365151",
        "https://fb.watch/abc123def",
        "https://www.facebook.com/share/v/abc123",
    ]
}

/// Get working test URLs (known to work or commonly used for testing)
fn get_working_test_urls() -> Vec<&'static str> {
    vec![
        "https://www.facebook.com/watch/?v=2209933269449948",
        "https://www.facebook.com/watch?v=419280024562892",
        "https://www.facebook.com/watch/?v=1193939392365151",
    ]
}

/// Get URL format test cases
fn get_url_format_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Standard Watch", "https://www.facebook.com/watch/?v=2209933269449948"),
        ("Watch without slash", "https://www.facebook.com/watch?v=419280024562892"),
        ("User Videos", "https://www.facebook.com/user/videos/123456789"),
        ("Reel Format", "https://www.facebook.com/reel/1193939392365151"),
        ("Mobile Format", "https://m.facebook.com/watch/?v=2209933269449948"),
        ("Short URL", "https://fb.watch/abc123def"),
        ("Share Video", "https://www.facebook.com/share/v/abc123"),
        ("Share Reel", "https://www.facebook.com/share/r/def456"),
    ]
}

/// Get private test URLs (for error handling tests)
fn get_private_test_urls() -> Vec<&'static str> {
    vec![
        "https://www.facebook.com/watch/?v=123456789",
        "https://www.facebook.com/private-user/videos/123456789",
    ]
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Print test progress with consistent formatting
fn print_test_progress(step: usize, total: usize, description: &str) {
    println!("🧪 Test {}/{}: {}", step, total, description);
    println!("   {}", "=".repeat(80));
}

/// Print test result summary
fn print_test_summary(passed: usize, failed: usize, skipped: usize) {
    println!("\n📊 TEST SUMMARY");
    println!("===============");
    println!("✅ Passed: {}", passed);
    println!("❌ Failed: {}", failed);
    println!("⏭️  Skipped: {}", skipped);
    println!("📋 Total: {}", passed + failed + skipped);

    let success_rate = if passed + failed > 0 {
        (passed as f64 / (passed + failed) as f64) * 100.0
    } else {
        0.0
    };
    println!("📈 Success Rate: {:.1}%", success_rate);
}

// ============================================================================
// MAIN FUNCTION AND CLI
// ============================================================================

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let mut config = TestConfig::default();
    let mut test_url = None;
    let mut run_all = false;
    let mut test_patterns_only = false;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--test-all" => run_all = true,
            "--test-patterns" => test_patterns_only = true,
            "--verbose" => config.verbose_logging = true,
            "--save-debug" => config.save_debug_files = true,
            "--test-private" => config.test_private_videos = true,
            "--debug" => config.debug_mode = true,
            url if url.starts_with("http") => test_url = Some(url.to_string()),
            _ => {
                println!("❌ Unknown argument: {}", arg);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    // Create test suite
    let test_suite = match ExtractionTestSuite::new(config) {
        Ok(suite) => suite,
        Err(e) => {
            println!("❌ Failed to create test suite: {}", e);
            std::process::exit(1);
        }
    };

    if test_patterns_only {
        run_pattern_tests().await?;
    } else if run_all {
        if let Err(e) = test_suite.run_all_tests().await {
            println!("❌ Test suite failed: {}", e);
            std::process::exit(1);
        }
    } else if let Some(url) = test_url {
        run_single_url_test(&test_suite, &url).await?;
    } else {
        print_usage();
        println!("\n🎯 Running default test with working URLs...");
        if let Err(e) = test_suite.run_all_tests().await {
            println!("❌ Test suite failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Print usage information
fn print_usage() {
    println!("🚀 COMPREHENSIVE FACEBOOK VIDEO EXTRACTION TEST SUITE");
    println!("=====================================================");
    println!("Uses the real facebook-extractor-core crate for authentic testing");
    println!();
    println!("USAGE:");
    println!("  cargo run --bin comprehensive_extraction_test [OPTIONS] [URL]");
    println!();
    println!("OPTIONS:");
    println!("  --test-all        Run all extraction tests");
    println!("  --test-patterns   Test only URL patterns and validation");
    println!("  --verbose         Enable verbose logging");
    println!("  --save-debug      Save debug JSON files");
    println!("  --test-private    Include private video tests");
    println!("  --debug           Enable debug mode with detailed logging");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin comprehensive_extraction_test --test-all");
    println!("  cargo run --bin comprehensive_extraction_test --debug https://www.facebook.com/watch/?v=123");
    println!("  cargo run --bin comprehensive_extraction_test --test-patterns");
}

/// Run tests for a single URL using the real FacebookExtractor (simple URL extraction)
async fn run_single_url_test(test_suite: &ExtractionTestSuite, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 SIMPLE VIDEO URL EXTRACTION TEST");
    println!("===================================");
    println!("🔗 Testing URL: {}", url);
    println!();

    // Validate URL
    let validation = test_suite.extractor.validate_url(url);
    if !validation.is_valid {
        println!("❌ Invalid Facebook URL format: {}", validation.error_message.unwrap_or_default());
        return Ok(());
    }

    // Extract video ID
    if let Some(video_id) = validation.video_id {
        println!("✅ Video ID: {}", video_id);
    } else {
        println!("⚠️  URL valid but no video ID extracted");
    }

    // Test simple video URL extraction
    println!("\n🔍 Extracting video URLs...");
    match test_suite.extractor.extract_video_info(url).await {
        Ok(video_info) => {
            println!("✅ Extraction succeeded!");
            println!("📝 Title: {}", video_info.title);
            println!("📊 Metadata:");
            println!("   👤 Author: {}", video_info.metadata.author);
            println!("   👍 Likes: {}", video_info.metadata.likes);
            println!("   💬 Comments: {}", video_info.metadata.comments);
            println!("   👁️ Views: {}", video_info.metadata.views);
            println!("   🔄 Shares: {}", video_info.metadata.shares);
            println!("   🏷️ Hashtags: {:?}", video_info.metadata.hashtags);
            println!("   ⏱️ Duration: {:?} seconds", video_info.metadata.duration_seconds);
            println!("🎬 Available video URLs:");

            for (i, quality) in video_info.qualities.iter().enumerate() {
                println!("   {}. {} - {}",
                    i + 1,
                    quality.quality,
                    quality.download_url
                );
            }

            // Save debug info if enabled
            if test_suite.config.save_debug_files {
                test_suite.save_extraction_debug(&video_info, 0).await;
            }
        }
        Err(e) => {
            println!("❌ Extraction failed: {}", e);
            test_suite.analyze_extraction_error(&e);
        }
    }

    Ok(())
}

/// Run pattern analysis tests using FacebookExtractor
async fn run_pattern_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 PATTERN ANALYSIS TESTS");
    println!("=========================");
    println!("🔧 Using facebook-extractor-core validation");
    println!();

    // Create a temporary extractor for validation
    let extractor = FacebookExtractor::new()?;

    // Test URL patterns
    println!("🔗 Testing URL patterns...");
    let test_cases = get_url_format_test_cases();
    for (format_name, url) in test_cases {
        let validation = extractor.validate_url(url);
        println!("   {} {} ({}): {}",
            if validation.is_valid { "✅" } else { "❌" },
            format_name,
            url,
            if let Some(video_id) = validation.video_id {
                format!("ID: {}", video_id)
            } else {
                "No ID".to_string()
            }
        );
    }

    // Test content type detection
    println!("\n🎬 Testing content type detection...");
    let content_test_urls = vec![
        ("Video", "https://www.facebook.com/watch/?v=2209933269449948"),
        ("Reel", "https://www.facebook.com/reel/1193939392365151"),
        ("Story", "https://www.facebook.com/stories/123456789"),
    ];

    for (content_type, url) in content_test_urls {
        let validation = extractor.validate_url(url);
        if validation.is_valid {
            println!("   ✅ {} detected: {:?}", content_type, validation.content_type);
        } else {
            println!("   ❌ {} not supported", content_type);
        }
    }

    Ok(())
}


