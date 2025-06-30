//! # Debug Test for Specific "Failing" Facebook URLs
//!
//! ## Purpose
//! Test the specific URLs reported as failing to verify their actual status
//! and identify any issues with extraction quality or metadata.
//!
//! ## URLs Under Test
//! 1. https://www.facebook.com/share/r/1EjZyJz8Ex/ (Share link)
//! 2. https://www.facebook.com/reel/1267969104819279 (Reel link)
//!
//! ## Usage
//! ```bash
//! cargo run --features="debug-tools" --bin debug_specific_failing_urls
//! ```

use facebook_extractor_core::FacebookExtractor;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 DEBUG TEST: Specific 'Failing' Facebook URLs");
    println!("===============================================");
    println!("Testing URLs reported as failing to determine actual status");
    println!();

    let extractor = FacebookExtractor::new()?;
    
    let test_urls = vec![
        (
            "Share Link",
            "https://www.facebook.com/share/r/1EjZyJz8Ex/",
            "Facebook share format - should extract video data"
        ),
        (
            "Reel Link",
            "https://www.facebook.com/reel/1267969104819279",
            "Facebook reel format - should extract video data"
        ),
        (
            "App Failing URL 1",
            "https://www.facebook.com/share/r/1EjZyJz8Ex/",
            "URL from app error - Thời thế thay đổi #j2team_relax"
        ),
        (
            "App Failing URL 2",
            "https://www.facebook.com/reel/1267969104819279",
            "URL from app error - Cine Gear Day 1. Welcome to the movie..."
        ),
    ];

    let mut total_tests = 0;
    let mut successful_extractions = 0;
    let mut failed_extractions = 0;

    for (test_name, url, description) in test_urls {
        total_tests += 1;
        println!("🎯 Test {}: {}", total_tests, test_name);
        println!("   URL: {}", url);
        println!("   Expected: {}", description);
        println!("   {}", "=".repeat(80));

        // Step 1: URL Validation
        println!("📋 Step 1: URL Validation");
        let validation = extractor.validate_url(url);
        if validation.is_valid {
            println!("   ✅ URL format is valid");
            if let Some(video_id) = &validation.video_id {
                println!("   🆔 Video ID extracted: {}", video_id);
            }
            if let Some(content_type) = &validation.content_type {
                println!("   🎬 Content type: {:?}", content_type);
            }
        } else {
            println!("   ❌ URL validation failed: {}", 
                validation.error_message.unwrap_or_default());
            failed_extractions += 1;
            println!();
            continue;
        }

        // Step 2: Video Extraction
        println!("\n📋 Step 2: Video Extraction");
        let start_time = Instant::now();
        
        match extractor.extract_video_info(url).await {
            Ok(video_info) => {
                let extraction_time = start_time.elapsed();
                println!("   ✅ Extraction SUCCESSFUL in {:?}", extraction_time);
                successful_extractions += 1;

                // Step 3: Detailed Analysis
                println!("\n📋 Step 3: Extraction Quality Analysis");
                analyze_extraction_quality(&video_info, test_name);
                
            }
            Err(e) => {
                let extraction_time = start_time.elapsed();
                println!("   ❌ Extraction FAILED in {:?}", extraction_time);
                println!("   Error: {}", e);
                failed_extractions += 1;
                
                // Analyze the error
                analyze_extraction_error(&e);
            }
        }
        
        println!("\n{}\n", "=".repeat(100));
    }

    // Final Summary
    println!("📊 FINAL TEST RESULTS");
    println!("=====================");
    println!("Total URLs tested: {}", total_tests);
    println!("✅ Successful extractions: {}", successful_extractions);
    println!("❌ Failed extractions: {}", failed_extractions);
    println!("📈 Success rate: {:.1}%", 
        (successful_extractions as f64 / total_tests as f64) * 100.0);
    
    if successful_extractions == total_tests {
        println!("\n🎉 CONCLUSION: All URLs are working correctly!");
        println!("   The reported 'failures' appear to be false positives.");
        println!("   Both URL formats are supported and extracting successfully.");
    } else if successful_extractions > 0 {
        println!("\n⚠️  CONCLUSION: Partial success - some URLs working");
        println!("   {} out of {} URLs are working correctly.", successful_extractions, total_tests);
    } else {
        println!("\n❌ CONCLUSION: All URLs are genuinely failing");
        println!("   Investigation needed for extraction issues.");
    }

    Ok(())
}

fn analyze_extraction_quality(video_info: &facebook_extractor_core::VideoInfo, test_name: &str) {
    println!("   📝 Title: {}", video_info.title);
    println!("   🆔 Video ID: {}", video_info.video_id);
    println!("   ⏱️  Duration: {:?}", video_info.duration);
    
    // Analyze metadata quality
    println!("\n   📊 Metadata Quality:");
    println!("      👤 Author: {}", video_info.metadata.author);
    println!("      👍 Likes: {}", video_info.metadata.likes);
    println!("      💬 Comments: {}", video_info.metadata.comments);
    println!("      👁️  Views: {}", video_info.metadata.views);
    println!("      🔄 Shares: {}", video_info.metadata.shares);
    println!("      🏷️  Hashtags: {:?}", video_info.metadata.hashtags);
    
    // Analyze video quality
    println!("\n   🎬 Video Quality Analysis:");
    println!("      📊 Total streams found: {}", video_info.qualities.len());
    
    if video_info.qualities.is_empty() {
        println!("      ❌ ISSUE: No video streams found!");
    } else {
        let mut has_hd = false;
        let mut has_reasonable_sizes = false;
        let mut total_size = 0;
        
        for (i, quality) in video_info.qualities.iter().enumerate() {
            println!("         {}. {} ({}x{}) - {} MB", 
                i + 1, 
                quality.quality, 
                quality.width, 
                quality.height,
                quality.estimated_size_mb
            );
            
            if quality.width >= 1280 || quality.height >= 720 {
                has_hd = true;
            }
            if quality.estimated_size_mb >= 5 {
                has_reasonable_sizes = true;
            }
            total_size += quality.estimated_size_mb;
        }
        
        // Quality assessment
        println!("\n   🔍 Quality Assessment:");
        if has_hd {
            println!("      ✅ HD quality streams available");
        } else {
            println!("      ⚠️  No HD quality streams found");
        }
        
        if has_reasonable_sizes {
            println!("      ✅ Reasonable file sizes detected");
        } else {
            println!("      ⚠️  All file sizes seem unusually small (possible detection issue)");
        }
        
        if total_size > 0 {
            println!("      📊 Total estimated size: {} MB", total_size);
        } else {
            println!("      ⚠️  File size detection may be failing");
        }
    }
    
    // Thumbnail analysis
    if !video_info.thumbnail.is_empty() {
        println!("\n   🖼️  Thumbnail: Available ({} chars)", video_info.thumbnail.len());
        if video_info.thumbnail.len() < 50 {
            println!("      ⚠️  Thumbnail URL seems unusually short");
        }
    } else {
        println!("\n   🖼️  Thumbnail: ❌ Not available");
    }
}

fn analyze_extraction_error(error: &facebook_extractor_core::FacebookExtractorError) {
    println!("\n   🔍 Error Analysis:");

    match error {
        facebook_extractor_core::FacebookExtractorError::AuthenticationRequired => {
            println!("      🔒 Authentication required - video may be private");
            println!("      💡 Suggestion: Check if video is publicly accessible");
        }
        facebook_extractor_core::FacebookExtractorError::AccessDenied { reason } => {
            println!("      🚫 Access denied: {}", reason);
            println!("      💡 Suggestion: Video may have privacy restrictions");
        }
        facebook_extractor_core::FacebookExtractorError::RateLimited => {
            println!("      ⏱️  Rate limited - too many requests");
            println!("      💡 Suggestion: Wait before retrying");
        }
        facebook_extractor_core::FacebookExtractorError::GeoBlocked => {
            println!("      🌍 Geo-blocked content");
            println!("      💡 Suggestion: Content may not be available in this region");
        }
        facebook_extractor_core::FacebookExtractorError::ContentUnavailable => {
            println!("      📭 Content no longer available");
            println!("      💡 Suggestion: Video may have been deleted or made private");
        }
        facebook_extractor_core::FacebookExtractorError::Network { source: _ } => {
            println!("      🌐 Network connectivity issue");
            println!("      💡 Suggestion: Check internet connection");
        }
        facebook_extractor_core::FacebookExtractorError::HtmlParsing { message } => {
            println!("      🔍 HTML parsing issue: {}", message);
            println!("      💡 Suggestion: Facebook may have changed their page structure");
        }
        facebook_extractor_core::FacebookExtractorError::StreamAnalysis { message } => {
            println!("      🎬 Stream analysis issue: {}", message);
            println!("      💡 Suggestion: Video format may not be supported");
        }
        _ => {
            println!("      ❓ Other error type: {}", error);
        }
    }
}
