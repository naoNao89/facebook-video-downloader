//! File size estimation comparison test
//!
//! This test compares the old HEAD request method vs the new accurate file size detection
//! to demonstrate the improvement in accuracy and validate that the ~60MB discrepancy
//! issue has been resolved.
//!
//! Usage:
//!   cargo run --bin test_file_size_comparison --features="debug-tools"
//!   
//! Environment Variables:
//!   FB_TEST_URL - Test URL (default: the problematic URL from the issue)

use facebook_extractor_core::{FacebookExtractor, file_size::AccurateFileSizeService};
use std::env;
use std::time::Duration;
use tokio::time::timeout;

/// Old file size estimation method using HEAD request
async fn get_old_file_size_estimation(url: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    let response = client
        .head(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await?;
    
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(50_000_000); // Default fallback
    
    let estimated_size_mb = (content_length / 1024 / 1024) as u32;
    Ok(estimated_size_mb)
}

/// New accurate file size estimation method
async fn get_new_file_size_estimation(url: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let service = AccurateFileSizeService::new()?;
    let size_bytes = service.get_accurate_file_size(url).await?;
    let size_mb = facebook_extractor_core::file_size::bytes_to_mb(size_bytes);
    Ok(size_mb)
}

/// Download actual file and get real size
async fn get_actual_file_size(url: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let bytes = response.bytes().await?;
    let size_mb = (bytes.len() / 1024 / 1024) as u32;
    Ok(size_mb)
}

/// Calculate accuracy percentage
fn calculate_accuracy(estimated: u32, actual: u32) -> f64 {
    if actual == 0 {
        return if estimated == 0 { 100.0 } else { 0.0 };
    }
    
    let difference = (estimated as i32 - actual as i32).abs() as f64;
    let accuracy = ((actual as f64 - difference) / actual as f64) * 100.0;
    accuracy.max(0.0)
}

/// Calculate improvement percentage
fn calculate_improvement(old_accuracy: f64, new_accuracy: f64) -> f64 {
    new_accuracy - old_accuracy
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🔬 FILE SIZE ESTIMATION METHOD COMPARISON TEST");
    println!("==============================================");
    
    // Get test URL from environment or use the problematic URL from the issue
    let test_url = env::var("FB_TEST_URL")
        .unwrap_or_else(|_| "https://www.facebook.com/watch?v=1063517975467288".to_string());
    
    println!("🎯 Test URL: {}", test_url);
    
    // Extract video information to get available qualities
    println!("\n🔍 Extracting video metadata...");
    let extractor = FacebookExtractor::new()?;
    let video_info = timeout(
        Duration::from_secs(60),
        extractor.extract_video_info(&test_url)
    ).await??;
    
    println!("✅ Video extracted: {}", video_info.title);
    println!("   Available qualities: {}", video_info.qualities.len());
    
    // Test the highest quality (usually 1080p) which had the biggest discrepancy
    let test_quality = video_info.qualities.iter()
        .max_by_key(|q| q.width * q.height)
        .ok_or("No video qualities found")?;
    
    println!("\n🎥 Testing quality: {} ({}x{})", 
             test_quality.quality, test_quality.width, test_quality.height);
    println!("   Download URL: {}...", &test_quality.download_url[..80.min(test_quality.download_url.len())]);
    
    println!("\n📊 COMPARISON TEST RESULTS");
    println!("==========================");
    
    // Test 1: Old method (HEAD request)
    println!("\n1️⃣ OLD METHOD (HTTP HEAD Request):");
    let old_start = std::time::Instant::now();
    match get_old_file_size_estimation(&test_quality.download_url).await {
        Ok(old_estimate) => {
            let old_time = old_start.elapsed();
            println!("   📏 Estimated size: {} MB", old_estimate);
            println!("   ⏱️  Time taken: {:.2}s", old_time.as_secs_f64());
            
            // Test 2: New method (Accurate detection)
            println!("\n2️⃣ NEW METHOD (Accurate File Size Detection):");
            let new_start = std::time::Instant::now();
            match get_new_file_size_estimation(&test_quality.download_url).await {
                Ok(new_estimate) => {
                    let new_time = new_start.elapsed();
                    println!("   📏 Estimated size: {} MB", new_estimate);
                    println!("   ⏱️  Time taken: {:.2}s", new_time.as_secs_f64());
                    
                    // Test 3: Actual download
                    println!("\n3️⃣ ACTUAL DOWNLOAD (Ground Truth):");
                    let actual_start = std::time::Instant::now();
                    match get_actual_file_size(&test_quality.download_url).await {
                        Ok(actual_size) => {
                            let actual_time = actual_start.elapsed();
                            println!("   📏 Actual size: {} MB", actual_size);
                            println!("   ⏱️  Download time: {:.2}s", actual_time.as_secs_f64());
                            
                            // Calculate accuracies
                            let old_accuracy = calculate_accuracy(old_estimate, actual_size);
                            let new_accuracy = calculate_accuracy(new_estimate, actual_size);
                            let improvement = calculate_improvement(old_accuracy, new_accuracy);
                            
                            // Print comparison results
                            println!("\n📈 ACCURACY COMPARISON");
                            println!("======================");
                            println!("📊 Old method accuracy: {:.1}%", old_accuracy);
                            println!("📊 New method accuracy: {:.1}%", new_accuracy);
                            println!("📈 Improvement: {:.1} percentage points", improvement);
                            
                            // Calculate discrepancies
                            let old_discrepancy = (old_estimate as i32 - actual_size as i32).abs();
                            let new_discrepancy = (new_estimate as i32 - actual_size as i32).abs();
                            
                            println!("\n🎯 DISCREPANCY ANALYSIS");
                            println!("=======================");
                            println!("❌ Old method discrepancy: {} MB", old_discrepancy);
                            println!("✅ New method discrepancy: {} MB", new_discrepancy);
                            println!("📉 Discrepancy reduction: {} MB", old_discrepancy - new_discrepancy);
                            
                            // Performance comparison
                            println!("\n⚡ PERFORMANCE COMPARISON");
                            println!("========================");
                            println!("🚀 Old method speed: {:.2}s", old_time.as_secs_f64());
                            println!("🚀 New method speed: {:.2}s", new_time.as_secs_f64());
                            let speed_diff = new_time.as_secs_f64() - old_time.as_secs_f64();
                            if speed_diff > 0.0 {
                                println!("⏱️  New method is {:.2}s slower (acceptable for accuracy gain)", speed_diff);
                            } else {
                                println!("⚡ New method is {:.2}s faster!", speed_diff.abs());
                            }
                            
                            // Final verdict
                            println!("\n🏆 FINAL VERDICT");
                            println!("================");
                            if new_accuracy > old_accuracy {
                                println!("✅ NEW METHOD WINS!");
                                println!("   🎯 Better accuracy: {:.1}% vs {:.1}%", new_accuracy, old_accuracy);
                                println!("   📉 Reduced discrepancy: {} MB vs {} MB", new_discrepancy, old_discrepancy);
                                
                                if old_discrepancy >= 50 && new_discrepancy < 10 {
                                    println!("   🎉 ISSUE RESOLVED: The ~60MB discrepancy problem has been fixed!");
                                }
                            } else if new_accuracy == old_accuracy {
                                println!("🤝 METHODS ARE EQUIVALENT");
                                println!("   Both methods provide the same accuracy level");
                            } else {
                                println!("⚠️  OLD METHOD PERFORMS BETTER");
                                println!("   This suggests the new method needs refinement");
                            }
                            
                            // Recommendations
                            println!("\n💡 RECOMMENDATIONS");
                            println!("==================");
                            if new_accuracy >= 95.0 {
                                println!("✅ Deploy new method - excellent accuracy achieved");
                            } else if new_accuracy > old_accuracy {
                                println!("📈 Deploy new method - shows improvement over old method");
                            } else {
                                println!("🔧 Refine new method - needs improvement before deployment");
                            }
                        }
                        Err(e) => {
                            println!("   ❌ Failed to download actual file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ New method failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Old method failed: {}", e);
        }
    }
    
    Ok(())
}
