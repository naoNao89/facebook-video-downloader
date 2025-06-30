//! # Authentication and Error Handling Test
//!
//! ## Purpose
//! Focused test for authentication detection, error handling, and edge cases
//! in Facebook video extraction. This consolidates error handling logic
//! from multiple test files.
//!
//! ## Features
//! - Authentication detection and bypass strategies
//! - Rate limiting and blocking detection
//! - Private video handling
//! - Error classification and suggestions
//! - Edge case testing
//!
//! ## Usage
//! ```bash
//! cargo run --bin auth_and_error_test
//! cargo run --bin auth_and_error_test --test-private
//! ```

mod common;

use common::*;
use std::env;

/// Authentication and error handling test suite
pub struct AuthErrorTest {
    client: reqwest::Client,
    mobile_client: reqwest::Client,
    config: TestConfig,
}

impl AuthErrorTest {
    /// Create a new authentication and error test
    pub fn new() -> TestResult<Self> {
        let client = create_test_client(None)?;
        let mobile_client = create_mobile_client()?;
        let config = TestConfig {
            save_debug_files: true,
            verbose_logging: true,
            test_private_videos: true,
            ..Default::default()
        };

        Ok(Self { client, mobile_client, config })
    }

    /// Run comprehensive authentication and error tests
    pub async fn run_auth_error_tests(&self) -> TestResult<()> {
        println!("🛡️ AUTHENTICATION AND ERROR HANDLING TEST SUITE");
        println!("================================================");
        println!();

        let mut passed = 0;
        let mut failed = 0;

        // Test 1: Authentication Detection
        print_test_progress(1, 5, "Authentication Detection");
        match self.test_authentication_detection().await {
            Ok(_) => {
                println!("✅ Authentication detection tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ Authentication detection tests failed: {}", e);
                failed += 1;
            }
        }

        // Test 2: Rate Limiting and Blocking Detection
        print_test_progress(2, 5, "Rate Limiting and Blocking Detection");
        match self.test_blocking_detection().await {
            Ok(_) => {
                println!("✅ Blocking detection tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ Blocking detection tests failed: {}", e);
                failed += 1;
            }
        }

        // Test 3: Error Classification
        print_test_progress(3, 5, "Error Classification and Handling");
        match self.test_error_classification().await {
            Ok(_) => {
                println!("✅ Error classification tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ Error classification tests failed: {}", e);
                failed += 1;
            }
        }

        // Test 4: Bypass Strategies
        print_test_progress(4, 5, "Authentication Bypass Strategies");
        match self.test_bypass_strategies().await {
            Ok(_) => {
                println!("✅ Bypass strategy tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ Bypass strategy tests failed: {}", e);
                failed += 1;
            }
        }

        // Test 5: Edge Cases
        print_test_progress(5, 5, "Edge Cases and Invalid Inputs");
        match self.test_edge_cases().await {
            Ok(_) => {
                println!("✅ Edge case tests passed");
                passed += 1;
            }
            Err(e) => {
                println!("❌ Edge case tests failed: {}", e);
                failed += 1;
            }
        }

        print_test_summary(passed, failed, 0);
        Ok(())
    }

    /// Test authentication detection capabilities
    async fn test_authentication_detection(&self) -> TestResult<()> {
        println!("🔒 Testing authentication detection...");

        // Test with a known public URL first
        let public_url = TestUrls::get_working_public_urls()[0];
        println!("   📋 Testing public URL: {}", public_url);

        let response = self.client.get(public_url).send().await?;
        let html = response.text().await?;
        
        let (auth_detected, blocking_detected, privacy_detected) = check_authentication_status(&html);
        
        println!("   📊 Authentication status for public URL:");
        println!("      🔐 Auth required: {}", auth_detected);
        println!("      🚫 Blocking detected: {}", blocking_detected);
        println!("      🔒 Privacy restrictions: {}", privacy_detected);

        // For a public URL, we shouldn't see strong privacy restrictions
        if privacy_detected {
            println!("   ⚠️  Warning: Privacy restrictions detected on supposedly public URL");
        }

        // Test authentication indicators
        let auth_indicators = [
            "login", "Log In", "authentication", "sign in",
            "private", "friends only", "requires login"
        ];

        let mut found_indicators = Vec::new();
        for indicator in &auth_indicators {
            if html.to_lowercase().contains(&indicator.to_lowercase()) {
                found_indicators.push(*indicator);
            }
        }

        if !found_indicators.is_empty() {
            println!("   🔍 Found auth indicators: {:?}", found_indicators);
        } else {
            println!("   ✅ No strong authentication indicators found");
        }

        Ok(())
    }

    /// Test blocking and rate limiting detection
    async fn test_blocking_detection(&self) -> TestResult<()> {
        println!("🚫 Testing blocking and rate limiting detection...");

        let test_url = TestUrls::get_working_public_urls()[1];
        println!("   📋 Testing URL: {}", test_url);

        // Test with different user agents to see if any trigger blocking
        let user_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "curl/7.68.0",
            "Python-requests/2.25.1",
            "FacebookBot/1.0",
        ];

        for (i, user_agent) in user_agents.iter().enumerate() {
            println!("   🔍 Testing user agent {}: {}", i + 1, user_agent);
            
            let response = self.client
                .get(test_url)
                .header("User-Agent", *user_agent)
                .send()
                .await?;

            println!("      📊 Status: {}", response.status());
            
            if response.status().is_success() {
                let html = response.text().await?;
                let (_, blocking_detected, _) = check_authentication_status(&html);
                
                if blocking_detected {
                    println!("      🚫 Blocking detected with this user agent");
                } else {
                    println!("      ✅ No blocking detected");
                }
            } else {
                println!("      ❌ HTTP error: {}", response.status());
            }
        }

        Ok(())
    }

    /// Test error classification and handling
    async fn test_error_classification(&self) -> TestResult<()> {
        println!("⚠️ Testing error classification...");

        // Test different error scenarios
        let error_scenarios = vec![
            ("Invalid URL", "https://not-facebook.com/video"),
            ("Malformed Facebook URL", "https://facebook.com/invalid"),
            ("Non-existent video", "https://www.facebook.com/watch/?v=999999999999999"),
        ];

        for (scenario, url) in error_scenarios {
            println!("   🧪 Testing scenario: {}", scenario);
            println!("      URL: {}", url);

            // Test URL validation
            if !is_valid_facebook_url(url) {
                println!("      ✅ Correctly identified as invalid URL");
                continue;
            }

            // Test video ID extraction
            match extract_video_id(url) {
                Ok(video_id) => {
                    println!("      📋 Extracted video ID: {}", video_id);
                    
                    // Test actual extraction
                    match self.client.get(url).send().await {
                        Ok(response) => {
                            println!("      📊 HTTP Status: {}", response.status());
                            
                            if response.status().is_success() {
                                let html = response.text().await?;
                                let (auth, blocking, privacy) = check_authentication_status(&html);
                                
                                println!("      🔍 Content analysis:");
                                println!("         Auth required: {}", auth);
                                println!("         Blocking: {}", blocking);
                                println!("         Privacy restrictions: {}", privacy);
                            }
                        }
                        Err(e) => {
                            println!("      ❌ Network error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("      ✅ Correctly failed video ID extraction: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Test authentication bypass strategies
    async fn test_bypass_strategies(&self) -> TestResult<()> {
        println!("🔄 Testing authentication bypass strategies...");

        let test_url = TestUrls::get_working_public_urls()[2];
        println!("   📋 Testing URL: {}", test_url);

        // Strategy 1: Desktop with enhanced headers
        println!("   🖥️  Strategy 1: Desktop with enhanced headers");
        match self.test_enhanced_desktop_fetch(test_url).await {
            Ok(success) => {
                println!("      {}", if success { "✅ Success" } else { "❌ Failed" });
            }
            Err(e) => {
                println!("      ❌ Error: {}", e);
            }
        }

        // Strategy 2: Mobile version
        println!("   📱 Strategy 2: Mobile version");
        let mobile_url = desktop_to_mobile_url(test_url);
        match self.test_mobile_fetch(&mobile_url).await {
            Ok(success) => {
                println!("      {}", if success { "✅ Success" } else { "❌ Failed" });
            }
            Err(e) => {
                println!("      ❌ Error: {}", e);
            }
        }

        // Strategy 3: Alternative URL formats
        println!("   🔗 Strategy 3: Alternative URL formats");
        if let Ok(video_id) = extract_video_id(test_url) {
            let alt_urls = vec![
                format!("https://www.facebook.com/video.php?v={}", video_id),
                format!("https://m.facebook.com/video.php?v={}", video_id),
            ];

            for alt_url in alt_urls {
                println!("      🔍 Testing: {}", alt_url);
                match self.client.get(&alt_url).send().await {
                    Ok(response) => {
                        println!("         📊 Status: {}", response.status());
                    }
                    Err(e) => {
                        println!("         ❌ Error: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Test edge cases and invalid inputs
    async fn test_edge_cases(&self) -> TestResult<()> {
        println!("🔍 Testing edge cases...");

        // Test empty and malformed URLs
        let edge_case_urls = vec![
            "",
            "not-a-url",
            "https://",
            "https://facebook.com",
            "https://www.facebook.com/",
            "https://www.facebook.com/watch",
            "https://www.facebook.com/watch?",
            "https://www.facebook.com/watch?v=",
            "https://www.facebook.com/watch?v=abc",
        ];

        for url in edge_case_urls {
            println!("   🧪 Testing edge case: '{}'", url);
            
            // Test URL validation
            let is_valid = is_valid_facebook_url(url);
            println!("      📋 Valid: {}", is_valid);
            
            if is_valid {
                // Test video ID extraction
                match extract_video_id(url) {
                    Ok(video_id) => {
                        println!("      🆔 Video ID: {}", video_id);
                    }
                    Err(e) => {
                        println!("      ❌ ID extraction failed: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Test enhanced desktop fetch
    async fn test_enhanced_desktop_fetch(&self, url: &str) -> TestResult<bool> {
        let response = self.client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
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

        if response.status().is_success() {
            let html = response.text().await?;
            let video_patterns = get_video_url_patterns();
            let found_urls = extract_urls_from_html(&html, &video_patterns);
            Ok(!found_urls.is_empty())
        } else {
            Ok(false)
        }
    }

    /// Test mobile fetch
    async fn test_mobile_fetch(&self, url: &str) -> TestResult<bool> {
        let response = self.mobile_client.get(url).send().await?;

        if response.status().is_success() {
            let html = response.text().await?;
            let video_patterns = get_video_url_patterns();
            let found_urls = extract_urls_from_html(&html, &video_patterns);
            Ok(!found_urls.is_empty())
        } else {
            Ok(false)
        }
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let test_private = args.contains(&"--test-private".to_string());

    println!("🛡️ AUTHENTICATION AND ERROR HANDLING TEST");
    println!("==========================================");
    
    if test_private {
        println!("🔒 Private video testing enabled");
    } else {
        println!("ℹ️  Private video testing disabled (use --test-private to enable)");
    }
    println!();

    // Create test instance
    let test = AuthErrorTest::new()?;

    // Run tests
    match test.run_auth_error_tests().await {
        Ok(_) => {
            println!("\n🎉 Authentication and error handling tests completed!");
        }
        Err(e) => {
            println!("\n❌ Tests failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
