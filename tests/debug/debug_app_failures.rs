//! # Debug App-Specific Facebook URL Failures
//!
//! ## Purpose
//! Debug the specific failures reported in the app UI to identify
//! differences between test environment and app environment.
//!
//! ## App Error Details
//! - Status: "Failed (2)"
//! - URLs showing 0% progress and "Failed" status
//! - Titles visible but extraction failing
//!
//! ## Usage
//! ```bash
//! cargo run --features="debug-tools" --bin debug_app_failures
//! ```

use facebook_extractor_core::{FacebookExtractor, ExtractorConfig};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 DEBUG: App-Specific Facebook URL Failures");
    println!("============================================");
    println!("Investigating discrepancy between test success and app failures");
    println!();

    // Test with different configurations that might match app behavior
    let configs = vec![
        ("Default Config", ExtractorConfig::default()),
        ("Performance Optimized", ExtractorConfig::performance_optimized()),
        ("Debug Optimized", ExtractorConfig::debug_optimized()),
    ];

    // URLs that are failing in the app (based on visible titles)
    let failing_urls = vec![
        (
            "Thời thế thay đổi #j2team_relax",
            "https://www.facebook.com/share/r/1EjZyJz8Ex/", // Guessed based on pattern
            "Vietnamese content - might have geo/language restrictions"
        ),
        (
            "Cine Gear Day 1. Welcome to the movie...",
            "https://www.facebook.com/reel/1267969104819279", // Guessed based on pattern  
            "English content - movie/cinema related"
        ),
    ];

    for (config_name, config) in configs {
        println!("🔧 Testing with {}", config_name);
        println!("   {}", "=".repeat(60));
        
        let extractor = match FacebookExtractor::with_config(config) {
            Ok(e) => e,
            Err(e) => {
                println!("   ❌ Failed to create extractor: {}", e);
                continue;
            }
        };

        for (title, url, description) in &failing_urls {
            println!("\n   🎯 Testing: {}", title);
            println!("      URL: {}", url);
            println!("      Context: {}", description);
            
            let start_time = Instant::now();
            
            match extractor.extract_video_info(url).await {
                Ok(video_info) => {
                    let duration = start_time.elapsed();
                    println!("      ✅ SUCCESS in {:?}", duration);
                    println!("         📝 Extracted Title: {}", video_info.title);
                    println!("         🎬 Streams: {}", video_info.qualities.len());
                    
                    // Check for potential app-breaking issues
                    if video_info.qualities.is_empty() {
                        println!("         ⚠️  WARNING: No video streams found!");
                    }
                    
                    if video_info.qualities.iter().all(|q| q.estimated_size_mb < 1) {
                        println!("         ⚠️  WARNING: All streams show very small sizes");
                    }
                    
                    if video_info.title.contains("Facebook Video") {
                        println!("         ⚠️  WARNING: Using fallback title");
                    }
                }
                Err(e) => {
                    let duration = start_time.elapsed();
                    println!("      ❌ FAILED in {:?}", duration);
                    println!("         Error: {}", e);
                    
                    // Analyze error type for app debugging
                    analyze_app_relevant_error(&e);
                }
            }
        }
        println!();
    }

    // Test potential app-specific issues
    println!("🔍 Testing App-Specific Scenarios");
    println!("=================================");
    
    test_concurrent_requests().await;
    test_rate_limiting().await;
    test_network_conditions().await;
    
    println!("\n💡 Debugging Suggestions for App");
    println!("================================");
    println!("1. Check if app is using same ExtractorConfig as tests");
    println!("2. Verify app's error handling doesn't mask real errors");
    println!("3. Check if app has different timeout settings");
    println!("4. Verify app's network configuration (proxy, user agent, etc.)");
    println!("5. Check if app is making concurrent requests (rate limiting)");
    println!("6. Verify app's progress reporting isn't interfering with extraction");
    println!("7. Check if app has different retry logic that might be failing");

    Ok(())
}

fn analyze_app_relevant_error(error: &facebook_extractor_core::FacebookExtractorError) {
    println!("         🔍 App Impact Analysis:");
    
    match error {
        facebook_extractor_core::FacebookExtractorError::Network { source: _ } => {
            println!("            🌐 Network issue - app should show connection error");
            println!("            💡 App fix: Implement network retry logic");
        }
        facebook_extractor_core::FacebookExtractorError::RateLimited => {
            println!("            ⏱️  Rate limiting - app making too many requests");
            println!("            💡 App fix: Implement request throttling");
        }
        facebook_extractor_core::FacebookExtractorError::HtmlParsing { message } => {
            println!("            🔍 Parsing issue: {}", message);
            println!("            💡 App fix: Update extraction patterns");
        }
        facebook_extractor_core::FacebookExtractorError::AuthenticationRequired => {
            println!("            🔒 Auth required - video might be private");
            println!("            💡 App fix: Show 'private video' message");
        }
        facebook_extractor_core::FacebookExtractorError::ContentUnavailable => {
            println!("            📭 Content unavailable - video deleted/private");
            println!("            💡 App fix: Show 'video not available' message");
        }
        _ => {
            println!("            ❓ Other error - check app error handling");
        }
    }
}

async fn test_concurrent_requests() {
    println!("\n🔄 Testing Concurrent Request Handling");
    println!("   (Simulating app making multiple requests simultaneously)");
    
    let extractor = match FacebookExtractor::new() {
        Ok(e) => e,
        Err(e) => {
            println!("   ❌ Failed to create extractor: {}", e);
            return;
        }
    };
    
    let url = "https://www.facebook.com/share/r/1EjZyJz8Ex/";
    let start_time = Instant::now();
    
    // Simulate app making multiple concurrent requests
    let handles = (0..3).map(|i| {
        let url = url.to_string();
        tokio::spawn(async move {
            let extractor = match FacebookExtractor::new() {
                Ok(e) => e,
                Err(e) => {
                    println!("   ❌ Failed to create extractor for request {}: {}", i + 1, e);
                    return Err(e);
                }
            };
            println!("   🚀 Starting concurrent request {}", i + 1);
            let result = extractor.extract_video_info(&url).await;
            println!("   📋 Concurrent request {} result: {}", i + 1,
                if result.is_ok() { "✅ Success" } else { "❌ Failed" });
            result
        })
    }).collect::<Vec<_>>();
    
    let results = futures::future::join_all(handles).await;
    let duration = start_time.elapsed();
    
    let successes = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    println!("   📊 Concurrent test results: {}/3 succeeded in {:?}", successes, duration);
    
    if successes < 3 {
        println!("   ⚠️  WARNING: Concurrent requests failing - app might hit rate limits");
    }
}

async fn test_rate_limiting() {
    println!("\n⏱️  Testing Rate Limiting Behavior");
    println!("   (Simulating rapid successive requests like app might make)");
    
    let extractor = match FacebookExtractor::new() {
        Ok(e) => e,
        Err(e) => {
            println!("   ❌ Failed to create extractor: {}", e);
            return;
        }
    };
    
    let url = "https://www.facebook.com/share/r/1EjZyJz8Ex/";
    
    for i in 1..=3 {
        let start_time = Instant::now();
        match extractor.extract_video_info(url).await {
            Ok(_) => {
                let duration = start_time.elapsed();
                println!("   ✅ Request {} succeeded in {:?}", i, duration);
            }
            Err(e) => {
                let duration = start_time.elapsed();
                println!("   ❌ Request {} failed in {:?}: {}", i, duration, e);
                if format!("{}", e).contains("rate") || format!("{}", e).contains("limit") {
                    println!("   🚨 RATE LIMITING DETECTED - This could be the app issue!");
                }
            }
        }
        
        // Small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

async fn test_network_conditions() {
    println!("\n🌐 Testing Network Condition Sensitivity");
    println!("   (Testing if extraction is sensitive to network conditions)");
    
    let extractor = match FacebookExtractor::new() {
        Ok(e) => e,
        Err(e) => {
            println!("   ❌ Failed to create extractor: {}", e);
            return;
        }
    };
    
    let url = "https://www.facebook.com/share/r/1EjZyJz8Ex/";
    
    // Test with timeout pressure
    println!("   🕐 Testing with timeout pressure...");
    let start_time = Instant::now();
    
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(10), 
        extractor.extract_video_info(url)
    ).await {
        Ok(Ok(_)) => {
            let duration = start_time.elapsed();
            println!("   ✅ Extraction completed within timeout ({:?})", duration);
        }
        Ok(Err(e)) => {
            let duration = start_time.elapsed();
            println!("   ❌ Extraction failed within timeout ({:?}): {}", duration, e);
        }
        Err(_) => {
            println!("   ⏰ Extraction timed out after 10 seconds");
            println!("   🚨 TIMEOUT ISSUE - App might have shorter timeouts!");
        }
    }
}
