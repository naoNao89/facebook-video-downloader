# Facebook Extraction Test Suite - Developer Guide

## Overview for Developers

This guide is for developers who need to understand, maintain, or extend the Facebook video extraction test suite. The test suite serves as a critical validation layer for the facebook-extractor-core crate and ensures the Tauri application works reliably.

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Test Suite Architecture                   │
├─────────────────────────────────────────────────────────────┤
│  comprehensive_extraction_test.rs (Main Test Binary)        │
│  ├── ExtractionTestSuite                                    │
│  │   ├── FacebookExtractor (Real production instance)      │
│  │   └── TestConfig (Configuration options)                │
│  ├── Test Functions                                         │
│  │   ├── test_url_validation()                             │
│  │   ├── test_real_extraction()                            │
│  │   ├── test_stream_analysis()                            │
│  │   ├── test_url_formats()                                │
│  │   └── test_error_handling()                             │
│  └── Utility Functions                                      │
│      ├── print_simple_video_urls()                         │
│      ├── analyze_extraction_error()                        │
│      └── save_extraction_debug()                           │
├─────────────────────────────────────────────────────────────┤
│  facebook-extractor-core (Production Crate)                │
│  ├── FacebookExtractor                                      │
│  ├── ExtractorConfig                                        │
│  ├── VideoInfo                                              │
│  └── FacebookExtractorError                                 │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **Real-World Testing**: Uses actual FacebookExtractor, not mocks
2. **Production Parity**: Identical configuration to Tauri application
3. **Comprehensive Coverage**: Tests all major functionality areas
4. **Debugging Support**: Extensive logging and debug output
5. **Maintainable**: Clear structure for easy updates

## Code Structure Analysis

### ExtractionTestSuite Implementation

```rust
pub struct ExtractionTestSuite {
    extractor: FacebookExtractor,  // Production extractor instance
    config: TestConfig,           // Test-specific configuration
}

impl ExtractionTestSuite {
    pub fn new(config: TestConfig) -> Result<Self, FacebookExtractorError> {
        let extractor_config = if config.debug_mode {
            ExtractorConfig::debug_optimized()
        } else {
            ExtractorConfig::performance_optimized()
        };
        
        let extractor = FacebookExtractor::with_config(extractor_config)?;
        Ok(Self { extractor, config })
    }
}
```

### Test Configuration System

```rust
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub save_debug_files: bool,    // Enable debug file generation
    pub verbose_logging: bool,     // Detailed console output
    pub timeout_seconds: u64,      // Network request timeout
    pub max_retries: usize,        // Retry attempts for failures
    pub test_private_videos: bool, // Include private video testing
    pub debug_mode: bool,          // Use debug-optimized extractor
}
```

### Test Data Management

```rust
// Working URLs for baseline testing
fn get_working_test_urls() -> Vec<&'static str> {
    vec![
        "https://www.facebook.com/watch/?v=2209933269449948",
        "https://www.facebook.com/watch?v=419280024562892",
        "https://www.facebook.com/watch/?v=1193939392365151",
    ]
}

// URL format validation test cases
fn get_url_format_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Standard Watch", "https://www.facebook.com/watch/?v=VIDEO_ID"),
        ("Reel Format", "https://www.facebook.com/reel/VIDEO_ID"),
        // ... more formats
    ]
}
```

## Adding New Test Cases

### 1. Adding URL Format Tests

To add support for a new Facebook URL format:

```rust
// In get_url_format_test_cases()
fn get_url_format_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        // Existing formats...
        ("New Format", "https://www.facebook.com/new/pattern/VIDEO_ID"),
    ]
}
```

### 2. Adding Custom Validation Tests

```rust
async fn test_custom_validation(&self) -> Result<(), FacebookExtractorError> {
    println!("🔍 Testing custom validation logic...");
    
    let test_url = "https://www.facebook.com/watch/?v=TEST_ID";
    
    match self.extractor.extract_video_info(test_url).await {
        Ok(video_info) => {
            // Custom validation logic
            if self.validate_custom_criteria(&video_info) {
                println!("   ✅ Custom validation passed");
            } else {
                println!("   ❌ Custom validation failed");
            }
        }
        Err(e) => {
            println!("   ❌ Extraction failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

fn validate_custom_criteria(&self, video_info: &VideoInfo) -> bool {
    // Implement custom validation logic
    video_info.qualities.len() >= 3 && 
    !video_info.title.is_empty() &&
    video_info.duration.contains(":")
}
```

### 3. Adding Performance Tests

```rust
async fn test_extraction_performance(&self) -> Result<(), FacebookExtractorError> {
    println!("⏱️ Testing extraction performance...");
    
    let test_urls = get_working_test_urls();
    let mut total_duration = Duration::new(0, 0);
    
    for url in test_urls {
        let start_time = std::time::Instant::now();
        
        match self.extractor.extract_video_info(url).await {
            Ok(_) => {
                let duration = start_time.elapsed();
                total_duration += duration;
                println!("   ✅ {} extracted in {:?}", url, duration);
            }
            Err(e) => {
                println!("   ❌ {} failed: {}", url, e);
            }
        }
    }
    
    let average_duration = total_duration / test_urls.len() as u32;
    println!("   📊 Average extraction time: {:?}", average_duration);
    
    // Assert performance requirements
    if average_duration.as_secs() > 30 {
        return Err(FacebookExtractorError::timeout("Extraction too slow"));
    }
    
    Ok(())
}
```

## Error Handling Patterns

### Error Analysis Implementation

```rust
fn analyze_extraction_error(&self, error: &FacebookExtractorError) {
    match error {
        FacebookExtractorError::AuthenticationRequired => {
            println!("      🔒 Authentication required - video may be private");
            // Could add retry logic with authentication
        }
        FacebookExtractorError::AccessDenied { reason } => {
            println!("      🚫 Access denied: {}", reason);
            // Could add geographic detection logic
        }
        FacebookExtractorError::RateLimited => {
            println!("      ⏱️ Rate limited - too many requests");
            // Could add backoff/retry logic
        }
        FacebookExtractorError::GeoBlocked => {
            println!("      🌍 Geo-blocked content");
            // Could add VPN detection suggestions
        }
        FacebookExtractorError::ContentUnavailable => {
            println!("      📭 Content no longer available");
            // Could add URL validation suggestions
        }
        FacebookExtractorError::Network { source: _ } => {
            println!("      🌐 Network connectivity issue");
            // Could add network diagnostics
        }
        FacebookExtractorError::HtmlParsing { message } => {
            println!("      🔍 HTML parsing issue: {}", message);
            // Could add Facebook structure change detection
        }
        FacebookExtractorError::StreamAnalysis { message } => {
            println!("      🎬 Stream analysis issue: {}", message);
            // Could add stream format change detection
        }
        _ => {
            println!("      ❓ Other error type");
            // Handle new error types as they're added
        }
    }
}
```

### Custom Error Types

```rust
// If you need to add custom error handling
#[derive(Debug)]
enum TestError {
    ExtractionFailed(FacebookExtractorError),
    ValidationFailed(String),
    PerformanceIssue(Duration),
    ConfigurationError(String),
}

impl From<FacebookExtractorError> for TestError {
    fn from(error: FacebookExtractorError) -> Self {
        TestError::ExtractionFailed(error)
    }
}
```

## Debugging and Diagnostics

### Debug File Generation

```rust
async fn save_extraction_debug(&self, video_info: &VideoInfo, index: usize) {
    if !self.config.save_debug_files {
        return;
    }
    
    // Save complete extraction results
    if let Ok(json) = serde_json::to_string_pretty(video_info) {
        let filename = format!("debug_extraction_{}.json", index);
        if let Err(e) = std::fs::write(&filename, json) {
            println!("      ⚠️ Failed to save debug file {}: {}", filename, e);
        } else {
            println!("      💾 Saved debug info to: {}", filename);
        }
    }
    
    // Save additional diagnostic information
    let diagnostic_info = format!(
        "Extraction Diagnostics\n\
         =====================\n\
         URL: {}\n\
         Video ID: {}\n\
         Title: {}\n\
         Qualities: {}\n\
         Timestamp: {}\n\
         Config: {:?}\n",
        video_info.source_url,
        video_info.video_id,
        video_info.title,
        video_info.qualities.len(),
        chrono::Utc::now(),
        self.config
    );
    
    let diag_filename = format!("debug_diagnostics_{}.txt", index);
    let _ = std::fs::write(&diag_filename, diagnostic_info);
}
```

### Logging Configuration

```rust
// Enhanced logging setup
fn setup_logging(verbose: bool) {
    let log_level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .format_timestamp_secs()
        .init();
}
```

## Integration with CI/CD

### GitHub Actions Integration

```yaml
name: Facebook Extraction Tests
on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    # Run daily at 2 AM UTC
    - cron: '0 2 * * *'

jobs:
  extraction-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run URL pattern tests
      run: |
        cargo run --bin comprehensive_extraction_test --features debug-tools -- --test-patterns
    
    - name: Run full extraction tests
      run: |
        cargo run --bin comprehensive_extraction_test --features debug-tools -- --test-all
      continue-on-error: true  # Facebook blocking may cause failures
    
    - name: Upload debug files
      if: failure()
      uses: actions/upload-artifact@v3
      with:
        name: extraction-debug-files
        path: debug_*
```

### Test Result Reporting

```rust
// Enhanced test reporting
fn generate_test_report(results: &TestResults) -> String {
    format!(
        "Facebook Extraction Test Report\n\
         ===============================\n\
         Date: {}\n\
         Total Tests: {}\n\
         Passed: {} ({:.1}%)\n\
         Failed: {} ({:.1}%)\n\
         Skipped: {}\n\
         Average Duration: {:?}\n\
         \n\
         Detailed Results:\n\
         {}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        results.total(),
        results.passed,
        (results.passed as f64 / results.total() as f64) * 100.0,
        results.failed,
        (results.failed as f64 / results.total() as f64) * 100.0,
        results.skipped,
        results.average_duration(),
        results.detailed_breakdown()
    )
}
```

## Maintenance Guidelines

### Regular Maintenance Tasks

#### Weekly Checklist
- [ ] Run full test suite
- [ ] Check for new Facebook URL formats
- [ ] Verify working test URLs are still valid
- [ ] Review error patterns for new issues

#### Monthly Checklist
- [ ] Update test URL database
- [ ] Analyze performance trends
- [ ] Review and update documentation
- [ ] Check for facebook-extractor-core updates

#### Quarterly Checklist
- [ ] Comprehensive test suite review
- [ ] Performance baseline updates
- [ ] Security review of test data
- [ ] CI/CD pipeline optimization

### URL Database Maintenance

```rust
// Automated URL validation
async fn validate_test_url_database() -> Result<(), TestError> {
    let urls = get_working_test_urls();
    let mut invalid_urls = Vec::new();
    
    for url in urls {
        let validation = extractor.validate_url(url);
        if !validation.is_valid {
            invalid_urls.push(url);
        }
    }
    
    if !invalid_urls.is_empty() {
        println!("⚠️ Invalid URLs found in test database:");
        for url in &invalid_urls {
            println!("   - {}", url);
        }
        return Err(TestError::ValidationFailed(
            format!("Found {} invalid URLs", invalid_urls.len())
        ));
    }
    
    Ok(())
}
```

### Version Compatibility

```rust
// Version compatibility checks
fn check_compatibility() -> Result<(), String> {
    // Check facebook-extractor-core version
    let core_version = env!("CARGO_PKG_VERSION");
    println!("facebook-extractor-core version: {}", core_version);
    
    // Check Rust version
    let rust_version = std::env::var("RUSTC_VERSION")
        .unwrap_or_else(|_| "unknown".to_string());
    println!("Rust version: {}", rust_version);
    
    // Perform compatibility validation
    // Add specific version checks as needed
    
    Ok(())
}
```

---

## Best Practices Summary

1. **Always use real FacebookExtractor** - No mocks in production tests
2. **Maintain test URL database** - Regular validation and updates
3. **Comprehensive error handling** - Test all error scenarios
4. **Performance monitoring** - Track extraction times and success rates
5. **Debug support** - Extensive logging and debug file generation
6. **CI/CD integration** - Automated testing in deployment pipeline
7. **Documentation updates** - Keep docs current with code changes
8. **Security awareness** - Handle debug data appropriately

For complete implementation details, refer to the source code in `tests/extraction/comprehensive_extraction_test.rs`.
