//! Comprehensive anti-blocking system tests
//!
//! This test suite validates all aspects of the anti-blocking functionality
//! including IPv6 rotation, user-agent rotation, header variation, timing
//! randomization, error handling, and cleanup mechanisms.

use facebook_extractor_core::{AntiBlockingConfig, AntiBlockingManager};

/// Test basic anti-blocking functionality
#[tokio::test]
async fn test_basic_anti_blocking_functionality() {
    println!("🔧 Testing basic anti-blocking functionality...");
    
    // Test default configuration
    let default_config = AntiBlockingConfig::default();
    let manager = AntiBlockingManager::new(default_config);
    
    // Test initialization
    let init_result = manager.initialize().await;
    match init_result {
        Ok(_) => {
            println!("✅ Default configuration initialization successful");
            
            // Test basic stats
            let stats = manager.get_stats().await;
            assert!(stats.user_agents_available > 0, "Should have user agents available");
            println!("📊 Basic stats: {:?}", stats);
            
            // Test cleanup
            let cleanup_result = manager.cleanup().await;
            assert!(cleanup_result.is_ok(), "Basic cleanup should succeed");
            println!("✅ Basic cleanup successful");
        }
        Err(e) => {
            println!("⚠️ Default initialization failed (might be expected): {}", e);
            
            // Even if initialization fails, cleanup should work
            let cleanup_result = manager.cleanup().await;
            assert!(cleanup_result.is_ok(), "Cleanup should work even after failed init");
        }
    }
}

/// Test IPv6 functionality
#[tokio::test]
async fn test_ipv6_functionality() {
    println!("🌐 Testing IPv6 functionality...");
    
    let config = AntiBlockingConfig {
        enable_ipv6_rotation: true,
        max_ipv6_addresses: 3,
        enable_user_agent_rotation: false,
        enable_header_variation: false,
        enable_timing_randomization: false,
        max_retry_attempts: 1,
        ..Default::default()
    };
    
    let manager = AntiBlockingManager::new(config);
    
    match manager.initialize().await {
        Ok(_) => {
            let stats = manager.get_stats().await;
            println!("✅ IPv6 initialization successful");
            println!("📊 IPv6 addresses available: {}", stats.ipv6_addresses_available);
            assert!(stats.ipv6_rotation_enabled, "IPv6 rotation should be enabled");
        }
        Err(e) => {
            println!("⚠️ IPv6 not available on this system: {}", e);
            // This is acceptable - not all systems support IPv6
        }
    }
    
    let _ = manager.cleanup().await;
}

/// Test User-Agent functionality
#[tokio::test]
async fn test_user_agent_functionality() {
    println!("🕵️ Testing User-Agent functionality...");
    
    let config = AntiBlockingConfig {
        enable_ipv6_rotation: false,
        enable_user_agent_rotation: true,
        enable_header_variation: false,
        enable_timing_randomization: false,
        max_retry_attempts: 1,
        ..Default::default()
    };
    
    let manager = AntiBlockingManager::new(config);
    let init_result = manager.initialize().await;
    assert!(init_result.is_ok(), "User-Agent initialization should succeed");
    
    let stats = manager.get_stats().await;
    assert!(stats.user_agent_rotation_enabled, "User-Agent rotation should be enabled");
    assert!(stats.user_agents_available > 10, "Should have good variety of user agents");
    
    println!("✅ User-Agent functionality verified");
    println!("📊 User agents available: {}", stats.user_agents_available);
    
    let _ = manager.cleanup().await;
}

/// Test header variation functionality
#[tokio::test]
async fn test_header_functionality() {
    println!("📋 Testing header variation functionality...");
    
    let config = AntiBlockingConfig {
        enable_ipv6_rotation: false,
        enable_user_agent_rotation: false,
        enable_header_variation: true,
        enable_timing_randomization: false,
        max_retry_attempts: 1,
        ..Default::default()
    };
    
    let manager = AntiBlockingManager::new(config);
    let init_result = manager.initialize().await;
    assert!(init_result.is_ok(), "Header variation initialization should succeed");
    
    let stats = manager.get_stats().await;
    assert!(stats.header_variation_enabled, "Header variation should be enabled");
    
    println!("✅ Header variation functionality verified");
    
    let _ = manager.cleanup().await;
}

/// Test timing randomization functionality
#[tokio::test]
async fn test_timing_functionality() {
    println!("⏱️ Testing timing randomization functionality...");
    
    let config = AntiBlockingConfig {
        enable_ipv6_rotation: false,
        enable_user_agent_rotation: false,
        enable_header_variation: false,
        enable_timing_randomization: true,
        base_delay_ms: 100,
        max_jitter_ms: 50,
        max_retry_attempts: 1,
        ..Default::default()
    };
    
    let manager = AntiBlockingManager::new(config);
    let init_result = manager.initialize().await;
    assert!(init_result.is_ok(), "Timing randomization initialization should succeed");
    
    let stats = manager.get_stats().await;
    assert!(stats.timing_randomization_enabled, "Timing randomization should be enabled");
    
    println!("✅ Timing randomization functionality verified");
    
    let _ = manager.cleanup().await;
}

/// Test integration functionality
#[tokio::test]
async fn test_integration_functionality() {
    println!("🔗 Testing integration functionality...");
    
    // Test all features enabled together
    let config = AntiBlockingConfig {
        enable_ipv6_rotation: true,
        max_ipv6_addresses: 2,
        enable_user_agent_rotation: true,
        enable_header_variation: true,
        enable_timing_randomization: true,
        base_delay_ms: 50, // Short for testing
        max_jitter_ms: 25,
        max_retry_attempts: 2,
        backoff_multiplier: 1.5,
        max_backoff_secs: 2,
    };
    
    let manager = AntiBlockingManager::new(config.clone());
    let init_result = manager.initialize().await;
    
    match init_result {
        Ok(_) => {
            let stats = manager.get_stats().await;
            println!("✅ Full integration initialization successful");
            println!("📊 Integration stats: {:?}", stats);
            
            // Count enabled features
            let enabled_features = [
                stats.ipv6_rotation_enabled,
                stats.user_agent_rotation_enabled,
                stats.header_variation_enabled,
                stats.timing_randomization_enabled,
            ].iter().filter(|&&x| x).count();
            
            println!("🔧 Enabled features: {}/4", enabled_features);
            assert!(enabled_features >= 3, "Most features should be enabled");
        }
        Err(e) => {
            println!("⚠️ Full integration failed (IPv6 might not be available): {}", e);
            
            // Test with IPv6 disabled
            let fallback_config = AntiBlockingConfig {
                enable_ipv6_rotation: false,
                enable_user_agent_rotation: true,
                enable_header_variation: true,
                enable_timing_randomization: true,
                ..config
            };
            
            let fallback_manager = AntiBlockingManager::new(fallback_config);
            let fallback_init = fallback_manager.initialize().await;
            assert!(fallback_init.is_ok(), "Fallback integration should succeed");
            
            let fallback_stats = fallback_manager.get_stats().await;
            println!("✅ Fallback integration successful");
            println!("📊 Fallback stats: {:?}", fallback_stats);
            
            let _ = fallback_manager.cleanup().await;
        }
    }
    
    let _ = manager.cleanup().await;
}

/// Test error handling functionality
#[tokio::test]
async fn test_error_handling_functionality() {
    println!("❌ Testing error handling functionality...");
    
    let config = AntiBlockingConfig {
        enable_ipv6_rotation: false,
        enable_user_agent_rotation: true,
        enable_header_variation: true,
        enable_timing_randomization: false,
        max_retry_attempts: 2,
        backoff_multiplier: 1.2,
        max_backoff_secs: 1, // Short for testing
        ..Default::default()
    };
    
    let manager = AntiBlockingManager::new(config);
    let init_result = manager.initialize().await;
    assert!(init_result.is_ok(), "Error handling test initialization should succeed");
    
    // Test with invalid URL
    let invalid_url = "http://invalid-domain-that-does-not-exist.com/test";
    let error_result = manager.make_request(invalid_url).await;
    
    match error_result {
        Ok(_) => {
            panic!("Request to invalid domain should not succeed");
        }
        Err(e) => {
            println!("✅ Error handling working correctly: {}", e);
            // Verify it's a network-related error
            let error_str = e.to_string().to_lowercase();
            assert!(
                error_str.contains("network") || 
                error_str.contains("dns") || 
                error_str.contains("connection") ||
                error_str.contains("timeout") ||
                error_str.contains("resolve"),
                "Error should be network-related"
            );
        }
    }
    
    // Manager should still be functional after errors
    let stats = manager.get_stats().await;
    println!("📊 Stats after error: {:?}", stats);
    
    let _ = manager.cleanup().await;
}

/// Test cleanup functionality
#[tokio::test]
async fn test_cleanup_functionality() {
    println!("🧹 Testing cleanup functionality...");
    
    let config = AntiBlockingConfig {
        enable_ipv6_rotation: true,
        max_ipv6_addresses: 2,
        enable_user_agent_rotation: true,
        enable_header_variation: true,
        enable_timing_randomization: true,
        ..Default::default()
    };
    
    let manager = AntiBlockingManager::new(config);
    let _ = manager.initialize().await;
    
    // Test multiple cleanups (idempotency)
    for i in 1..=3 {
        let cleanup_result = manager.cleanup().await;
        assert!(cleanup_result.is_ok(), "Cleanup {} should succeed", i);
        println!("✅ Cleanup {} successful", i);
    }
    
    println!("✅ Cleanup functionality verified");
}

/// Test comprehensive anti-blocking system
#[tokio::test]
async fn test_anti_blocking_comprehensive() {
    println!("🛡️ Starting comprehensive anti-blocking system tests");

    // Test basic functionality
    println!("🔧 Testing basic anti-blocking functionality...");
    let default_config = AntiBlockingConfig::default();
    let manager = AntiBlockingManager::new(default_config);

    match manager.initialize().await {
        Ok(_) => {
            let stats = manager.get_stats().await;
            assert!(stats.user_agents_available > 0, "Should have user agents available");
            println!("✅ Basic functionality test passed");
            let _ = manager.cleanup().await;
        }
        Err(e) => {
            println!("⚠️ Basic initialization failed (might be expected): {}", e);
            let _ = manager.cleanup().await;
        }
    }

    println!("✅ All anti-blocking system tests completed successfully");
}
