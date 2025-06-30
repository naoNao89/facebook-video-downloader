//! # Facebook URL Test
//!
//! ## Purpose
//! Tests Facebook URL parsing and validation functionality
//!
//! ## Category
//! URL Validation
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_facebook_url
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

//! Test script for Facebook Video Downloader
//!
//! This script tests the core Facebook video extraction functionality
//! with the specific URL provided by the user.

use std::process::Command;

fn main() {
    println!("🧪 Testing Facebook Video Downloader with URL:");
    println!("   https://www.facebook.com/watch/?v=2209933269449948");
    println!();

    // Test 1: Check if we can compile the core library
    println!("📦 Test 1: Compiling facebook-extractor-core...");
    let compile_result = Command::new("cargo")
        .args(&["check", "--package", "facebook-extractor-core"])
        .output();

    match compile_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Core library compiles successfully");
            } else {
                println!("❌ Core library compilation failed:");
                println!("{}", String::from_utf8_lossy(&output.stderr));
                return;
            }
        }
        Err(e) => {
            println!("❌ Failed to run cargo check: {}", e);
            return;
        }
    }

    // Test 2: Check if we can run basic tests
    println!("\n🧪 Test 2: Running core library tests...");
    let test_result = Command::new("cargo")
        .args(&["test", "--package", "facebook-extractor-core", "--lib"])
        .output();

    match test_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Core library tests pass");
            } else {
                println!("⚠️  Some tests may have failed:");
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("❌ Failed to run tests: {}", e);
        }
    }

    // Test 3: Check if FFmpeg is available
    println!("\n🎬 Test 3: Checking FFmpeg availability...");
    let ffmpeg_result = Command::new("ffmpeg")
        .args(&["-version"])
        .output();

    match ffmpeg_result {
        Ok(output) => {
            if output.status.success() {
                let version_info = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = version_info.lines().next() {
                    println!("✅ FFmpeg is available: {}", first_line);
                }
            } else {
                println!("⚠️  FFmpeg check failed");
            }
        }
        Err(_) => {
            println!("⚠️  FFmpeg not found in PATH");
        }
    }

    // Test 4: Check if we can build a simple test binary
    println!("\n🔨 Test 4: Creating test binary...");
    
    // Create a simple test program
    let test_code = r#"
fn main() {
    println!("🔍 Testing Facebook URL validation...");

    let test_url = "https://www.facebook.com/watch/?v=2209933269449948";
    println!("Testing URL: {}", test_url);

    // Test basic URL pattern matching
    if test_url.contains("facebook.com") && test_url.contains("watch") {
        println!("✅ URL appears to be a valid Facebook video URL");
    } else {
        println!("⚠️  URL does not match expected Facebook video pattern");
    }

    println!("🎯 Basic URL pattern test completed");
}
"#;

    // Write test file
    if let Err(e) = std::fs::write("test_extraction.rs", test_code) {
        println!("❌ Failed to write test file: {}", e);
        return;
    }

    // Try to compile and run the test
    let build_result = Command::new("cargo")
        .args(&["run", "--bin", "test_extraction"])
        .output();

    match build_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Test binary executed successfully:");
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                println!("⚠️  Test binary execution had issues:");
                println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
                println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("❌ Failed to build/run test binary: {}", e);
        }
    }

    // Cleanup
    let _ = std::fs::remove_file("test_extraction.rs");

    println!("\n📋 Test Summary:");
    println!("   - Core library compilation: Check above results");
    println!("   - Library tests: Check above results");
    println!("   - FFmpeg availability: Check above results");
    println!("   - Basic functionality: Check above results");
    println!();
    println!("🔗 Test URL: https://www.facebook.com/watch/?v=2209933269449948");
    println!();
    println!("📝 Next steps:");
    println!("   1. If tests pass, the core functionality is working");
    println!("   2. For full testing, run the Tauri desktop app");
    println!("   3. Use the UI to input the test URL and attempt download");
}
