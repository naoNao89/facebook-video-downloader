//! Comprehensive integration test for accurate file size detection
//!
//! This test validates that our new accurate file size detection system
//! resolves the ~60MB discrepancy issue by comparing estimated vs actual
//! downloaded file sizes across multiple video qualities.
//!
//! Usage:
//!   cargo run --bin test_accurate_file_size_integration --features="debug-tools"
//!   
//! Environment Variables:
//!   FB_TEST_URL_1080P - Primary test URL for 1080p video
//!   FB_TEST_URL_720P  - Secondary test URL for 720p video  
//!   FB_TEST_TIMEOUT   - Timeout in seconds (default: 120)
//!   FB_TEST_TOLERANCE - Accuracy tolerance percentage (default: 5.0)

use facebook_extractor_core::FacebookExtractor;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use uuid::Uuid;

/// Test configuration loaded from environment variables
#[derive(Debug, Clone)]
struct IntegrationTestConfig {
    /// Primary test URL (1080p preferred)
    pub primary_test_url: String,
    /// Secondary test URL (720p preferred) 
    pub secondary_test_url: Option<String>,
    /// Network timeout in seconds
    pub timeout_seconds: u64,
    /// Accuracy tolerance as percentage (e.g., 5.0 for ±5%)
    pub tolerance_percentage: f64,
    /// Temporary directory for downloads
    pub temp_dir: PathBuf,
    /// Whether to keep files for debugging
    pub keep_debug_files: bool,
}

impl IntegrationTestConfig {
    /// Load configuration from environment variables with sensible defaults
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let primary_test_url = env::var("FB_TEST_URL_1080P")
            .unwrap_or_else(|_| "https://www.facebook.com/watch?v=1063517975467288".to_string());
        
        let secondary_test_url = env::var("FB_TEST_URL_720P").ok();
        
        let timeout_seconds = env::var("FB_TEST_TIMEOUT")
            .unwrap_or_else(|_| "120".to_string())
            .parse::<u64>()
            .unwrap_or(120);
        
        let tolerance_percentage = env::var("FB_TEST_TOLERANCE")
            .unwrap_or_else(|_| "5.0".to_string())
            .parse::<f64>()
            .unwrap_or(5.0);
        
        let keep_debug_files = env::var("FB_TEST_KEEP_FILES")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
        
        // Create unique temporary directory
        let temp_dir = env::temp_dir().join(format!("fb_file_size_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir)?;
        
        Ok(Self {
            primary_test_url,
            secondary_test_url,
            timeout_seconds,
            tolerance_percentage,
            temp_dir,
            keep_debug_files,
        })
    }
}

/// Results from file size accuracy testing
#[derive(Debug)]
struct AccuracyTestResult {
    pub url: String,
    pub quality: String,
    pub estimated_size_mb: u32,
    pub actual_size_mb: u32,
    pub accuracy_percentage: f64,
    pub within_tolerance: bool,
    pub download_time_seconds: f64,
    pub file_path: Option<PathBuf>,
}

/// Temporary download manager with automatic cleanup
struct TemporaryDownloadManager {
    temp_dir: PathBuf,
    keep_files: bool,
    downloaded_files: Vec<PathBuf>,
}

impl TemporaryDownloadManager {
    fn new(temp_dir: PathBuf, keep_files: bool) -> Self {
        Self {
            temp_dir,
            keep_files,
            downloaded_files: Vec::new(),
        }
    }
    
    /// Generate a unique filename for a download
    fn generate_filename(&self, quality: &str, extension: &str) -> PathBuf {
        let filename = format!("test_video_{}_{}.{}",
                             quality.replace(" ", "_").to_lowercase(),
                             Uuid::new_v4().simple(),
                             extension);
        self.temp_dir.join(filename)
    }
    
    /// Track a downloaded file
    fn track_file(&mut self, file_path: PathBuf) {
        self.downloaded_files.push(file_path);
    }
    
    /// Get file size in bytes
    fn get_file_size(&self, file_path: &Path) -> Result<u64, std::io::Error> {
        let metadata = fs::metadata(file_path)?;
        Ok(metadata.len())
    }
}

impl Drop for TemporaryDownloadManager {
    fn drop(&mut self) {
        if !self.keep_files {
            println!("🧹 Cleaning up temporary files...");
            for file_path in &self.downloaded_files {
                if file_path.exists() {
                    if let Err(e) = fs::remove_file(file_path) {
                        eprintln!("⚠️  Failed to remove {}: {}", file_path.display(), e);
                    } else {
                        println!("   🗑️  Removed: {}", file_path.display());
                    }
                }
            }
            
            // Remove temp directory if empty
            if let Err(e) = fs::remove_dir(&self.temp_dir) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("⚠️  Failed to remove temp directory {}: {}", self.temp_dir.display(), e);
                }
            } else {
                println!("   📁 Removed temp directory: {}", self.temp_dir.display());
            }
        } else {
            println!("🔍 Debug files kept in: {}", self.temp_dir.display());
            for file_path in &self.downloaded_files {
                println!("   📄 {}", file_path.display());
            }
        }
    }
}

/// Calculate accuracy percentage between estimated and actual values
fn calculate_accuracy(estimated: u32, actual: u32) -> f64 {
    if actual == 0 {
        return if estimated == 0 { 100.0 } else { 0.0 };
    }
    
    let difference = (estimated as i32 - actual as i32).abs() as f64;
    let accuracy = ((actual as f64 - difference) / actual as f64) * 100.0;
    accuracy.max(0.0)
}

/// Check if accuracy is within tolerance
fn is_within_tolerance(estimated: u32, actual: u32, tolerance_percentage: f64) -> bool {
    if actual == 0 {
        return estimated == 0;
    }
    
    let difference_percentage = ((estimated as i32 - actual as i32).abs() as f64 / actual as f64) * 100.0;
    difference_percentage <= tolerance_percentage
}

/// Format file size in human-readable format
fn format_file_size(size_mb: u32) -> String {
    if size_mb >= 1024 {
        format!("{:.1} GB", size_mb as f64 / 1024.0)
    } else {
        format!("{} MB", size_mb)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🧪 COMPREHENSIVE FILE SIZE ACCURACY INTEGRATION TEST");
    println!("====================================================");
    
    // Load configuration
    let config = IntegrationTestConfig::from_env()?;
    println!("📋 Test Configuration:");
    println!("   🎯 Primary URL: {}", config.primary_test_url);
    if let Some(ref secondary_url) = config.secondary_test_url {
        println!("   🎯 Secondary URL: {}", secondary_url);
    }
    println!("   ⏱️  Timeout: {} seconds", config.timeout_seconds);
    println!("   📊 Tolerance: ±{}%", config.tolerance_percentage);
    println!("   📁 Temp directory: {}", config.temp_dir.display());
    println!("   🔍 Keep debug files: {}", config.keep_debug_files);
    
    // Initialize services
    let extractor = FacebookExtractor::new()?;
    let mut download_manager = TemporaryDownloadManager::new(
        config.temp_dir.clone(), 
        config.keep_debug_files
    );
    
    let mut test_results = Vec::new();
    let mut test_urls = vec![config.primary_test_url.clone()];
    if let Some(ref secondary_url) = config.secondary_test_url {
        test_urls.push(secondary_url.clone());
    }
    
    println!("\n🚀 Starting integration tests...");
    
    for (url_index, test_url) in test_urls.iter().enumerate() {
        println!("\n📹 Testing URL {} of {}: {}", url_index + 1, test_urls.len(), test_url);
        println!("{}", "─".repeat(80));
        
        // Test this URL - implementation continues in next part...
        match test_video_file_size_accuracy(
            &extractor,
            &mut download_manager,
            test_url,
            &config,
        ).await {
            Ok(mut results) => {
                test_results.append(&mut results);
            }
            Err(e) => {
                eprintln!("❌ Failed to test URL {}: {}", test_url, e);
                continue;
            }
        }
    }
    
    // Print comprehensive results
    print_test_summary(&test_results, &config);
    
    Ok(())
}

/// Test file size accuracy for a single video URL
async fn test_video_file_size_accuracy(
    extractor: &FacebookExtractor,
    download_manager: &mut TemporaryDownloadManager,
    test_url: &str,
    config: &IntegrationTestConfig,
) -> Result<Vec<AccuracyTestResult>, Box<dyn std::error::Error>> {
    println!("🔍 Extracting video metadata...");
    
    // Extract video information with timeout
    let video_info = timeout(
        Duration::from_secs(config.timeout_seconds),
        extractor.extract_video_info(test_url)
    ).await??;
    
    println!("✅ Video extracted successfully:");
    println!("   📺 Title: {}", video_info.title);
    println!("   👤 Author: {}", video_info.metadata.author);
    println!("   🎬 Duration: {}s", video_info.metadata.duration_seconds.unwrap_or(0));
    println!("   🎯 Available qualities: {}", video_info.qualities.len());
    
    let mut results = Vec::new();
    
    // Test each available quality
    for (quality_index, quality) in video_info.qualities.iter().enumerate() {
        println!("\n   🎥 Testing quality {}: {} ({}x{})", 
                quality_index + 1, quality.quality, quality.width, quality.height);
        
        let start_time = Instant::now();
        
        // This is where we'll implement the actual download and comparison
        // Implementation continues in the next part...
        
        match test_single_quality(
            quality,
            download_manager,
            config,
            start_time,
        ).await {
            Ok(result) => {
                results.push(result);
            }
            Err(e) => {
                eprintln!("      ❌ Failed to test quality {}: {}", quality.quality, e);
                continue;
            }
        }
    }
    
    Ok(results)
}

/// Test a single video quality
async fn test_single_quality(
    quality: &facebook_extractor_core::types::VideoQuality,
    download_manager: &mut TemporaryDownloadManager,
    config: &IntegrationTestConfig,
    start_time: Instant,
) -> Result<AccuracyTestResult, Box<dyn std::error::Error>> {
    println!("      📊 Estimated size: {} MB", quality.estimated_size_mb);

    // Generate unique filename for this download
    let file_path = download_manager.generate_filename(&quality.quality, "mp4");

    // Download the actual video file with timeout
    println!("      ⬇️  Downloading video to verify actual size...");
    let download_result = timeout(
        Duration::from_secs(config.timeout_seconds),
        download_video_file(&quality.download_url, &file_path)
    ).await?;

    match download_result {
        Ok(_) => {
            download_manager.track_file(file_path.clone());

            // Get actual file size
            let actual_size_bytes = download_manager.get_file_size(&file_path)?;
            let actual_size_mb = (actual_size_bytes / 1024 / 1024) as u32;

            let download_time = start_time.elapsed().as_secs_f64();

            // Calculate accuracy
            let accuracy = calculate_accuracy(quality.estimated_size_mb, actual_size_mb);
            let within_tolerance = is_within_tolerance(
                quality.estimated_size_mb,
                actual_size_mb,
                config.tolerance_percentage
            );

            println!("      ✅ Download completed:");
            println!("         📏 Actual size: {} MB", actual_size_mb);
            println!("         🎯 Accuracy: {:.1}%", accuracy);
            println!("         ⏱️  Download time: {:.1}s", download_time);
            println!("         {} Within tolerance (±{}%)",
                    if within_tolerance { "✅" } else { "❌" },
                    config.tolerance_percentage);

            Ok(AccuracyTestResult {
                url: quality.download_url.clone(),
                quality: quality.quality.clone(),
                estimated_size_mb: quality.estimated_size_mb,
                actual_size_mb,
                accuracy_percentage: accuracy,
                within_tolerance,
                download_time_seconds: download_time,
                file_path: Some(file_path),
            })
        }
        Err(e) => {
            println!("      ❌ Download failed: {}", e);
            Err(e)
        }
    }
}

/// Download a video file from URL to local path
async fn download_video_file(url: &str, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300)) // 5 minute timeout for large files
        .connect_timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(5))
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
    fs::write(file_path, &bytes)?;

    Ok(())
}

/// Print comprehensive test summary
fn print_test_summary(results: &[AccuracyTestResult], config: &IntegrationTestConfig) {
    println!("\n📊 COMPREHENSIVE TEST RESULTS");
    println!("==============================");

    if results.is_empty() {
        println!("❌ No test results to display");
        return;
    }

    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.within_tolerance).count();
    let failed_tests = total_tests - passed_tests;

    println!("📈 Overall Statistics:");
    println!("   🧪 Total tests: {}", total_tests);
    println!("   ✅ Passed (within ±{}% tolerance): {}", config.tolerance_percentage, passed_tests);
    println!("   ❌ Failed: {}", failed_tests);
    println!("   📊 Success rate: {:.1}%", (passed_tests as f64 / total_tests as f64) * 100.0);

    // Calculate average accuracy
    let avg_accuracy = results.iter().map(|r| r.accuracy_percentage).sum::<f64>() / total_tests as f64;
    println!("   🎯 Average accuracy: {:.1}%", avg_accuracy);

    // Calculate total download time
    let total_download_time = results.iter().map(|r| r.download_time_seconds).sum::<f64>();
    println!("   ⏱️  Total download time: {:.1}s", total_download_time);

    println!("\n📋 Detailed Results:");
    println!("{:<20} {:<15} {:<15} {:<10} {:<10} {:<8}",
             "Quality", "Estimated", "Actual", "Accuracy", "Status", "Time");
    println!("{}", "─".repeat(85));

    for result in results {
        let status = if result.within_tolerance { "✅ PASS" } else { "❌ FAIL" };
        println!("{:<20} {:<15} {:<15} {:<10.1}% {:<10} {:<8.1}s",
                result.quality,
                format_file_size(result.estimated_size_mb),
                format_file_size(result.actual_size_mb),
                result.accuracy_percentage,
                status,
                result.download_time_seconds);
    }

    // Show improvement analysis
    println!("\n🔍 Accuracy Analysis:");
    let high_accuracy_count = results.iter().filter(|r| r.accuracy_percentage >= 95.0).count();
    let medium_accuracy_count = results.iter().filter(|r| r.accuracy_percentage >= 85.0 && r.accuracy_percentage < 95.0).count();
    let low_accuracy_count = results.iter().filter(|r| r.accuracy_percentage < 85.0).count();

    println!("   🎯 High accuracy (≥95%): {} tests", high_accuracy_count);
    println!("   📊 Medium accuracy (85-95%): {} tests", medium_accuracy_count);
    println!("   ⚠️  Low accuracy (<85%): {} tests", low_accuracy_count);

    if passed_tests == total_tests {
        println!("\n🎉 ALL TESTS PASSED! The accurate file size detection system is working correctly.");
        println!("   The new system successfully resolves the file size discrepancy issue.");
    } else if passed_tests > 0 {
        println!("\n⚠️  PARTIAL SUCCESS: {}/{} tests passed.", passed_tests, total_tests);
        println!("   The system shows improvement but may need further refinement.");
    } else {
        println!("\n❌ ALL TESTS FAILED: The file size detection needs investigation.");
    }

    println!("\n💡 Next Steps:");
    if avg_accuracy >= 95.0 {
        println!("   ✅ File size accuracy is excellent - ready for production");
    } else if avg_accuracy >= 85.0 {
        println!("   📈 File size accuracy is good - consider minor optimizations");
    } else {
        println!("   🔧 File size accuracy needs improvement - review estimation algorithms");
    }
}
