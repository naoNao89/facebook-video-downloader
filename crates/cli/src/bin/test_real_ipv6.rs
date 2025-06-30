//! Test program for real IPv6 implementation
//!
//! This program demonstrates the real IPv6 address rotation functionality
//! with proper user consent and safety measures.

use facebook_video_downloader_core::network::anti_blocking::{
    AntiBlockingConfig, AntiBlockingManager, IPv6ConsentManager, IPv6Mode
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚨 REAL IPv6 ADDRESS ROTATION TEST");
    println!("==================================");
    println!();
    println!("This program will test the REAL IPv6 implementation that makes");
    println!("actual system-level network configuration changes.");
    println!();

    // Step 1: Check system requirements
    println!("🔍 Step 1: Checking system requirements...");
    let mut consent_manager = IPv6ConsentManager::new();
    
    match consent_manager.check_system_requirements().await {
        Ok(requirements) => {
            println!("✅ System requirements check completed");
            println!("Compatibility Score: {}/100", requirements.compatibility_score);
            
            if requirements.compatibility_score < 70 {
                println!("⚠️ Low compatibility score. Real IPv6 implementation may not work properly.");
                println!("Falling back to simulation mode for safety.");
                test_simulation_mode().await?;
                return Ok(());
            }

            // Step 2: Request user consent
            println!("\n🔐 Step 2: Requesting user consent...");
            let consent = consent_manager.request_user_consent(&requirements).await?;
            
            if !consent.consent_given {
                println!("❌ User consent not given. Falling back to simulation mode.");
                test_simulation_mode().await?;
                return Ok(());
            }

            // Step 3: Test real IPv6 implementation
            println!("\n🚨 Step 3: Testing REAL IPv6 implementation...");
            test_real_mode(consent).await?;
        }
        Err(e) => {
            println!("❌ System requirements check failed: {}", e);
            println!("Falling back to simulation mode for safety.");
            test_simulation_mode().await?;
        }
    }

    Ok(())
}

async fn test_simulation_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️ Testing Simulation Mode (Safe)");
    println!("==================================");

    let config = AntiBlockingConfig::default(); // Uses simulation mode by default
    let manager = AntiBlockingManager::new(config);

    match manager.initialize().await {
        Ok(_) => {
            println!("✅ Simulation mode initialized successfully");
            
            let stats = manager.get_stats().await;
            println!("IPv6 Addresses Available: {}", stats.ipv6_addresses_available);
            
            // Test a simple request
            println!("\n📡 Testing HTTP request with simulation mode...");
            match manager.make_request("https://httpbin.org/ip").await {
                Ok(response) => {
                    let text = response.text().await?;
                    println!("✅ Request successful: {}", text);
                }
                Err(e) => {
                    println!("❌ Request failed: {}", e);
                }
            }

            // Show safety report
            println!("\n📋 Safety Report:");
            let safety_report = manager.get_safety_report().await;
            println!("{}", safety_report);

            // Cleanup
            manager.cleanup().await?;
            println!("✅ Simulation mode cleanup completed");
        }
        Err(e) => {
            println!("❌ Simulation mode initialization failed: {}", e);
        }
    }

    Ok(())
}

async fn test_real_mode(consent: facebook_video_downloader_core::network::anti_blocking::IPv6ConsentStatus) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚨 Testing REAL IPv6 Mode (CRITICAL)");
    println!("====================================");
    println!("🚨 WARNING: This will modify your system's network configuration!");

    // Create configuration for real mode
    let mut config = AntiBlockingConfig::default();
    config.ipv6_mode = IPv6Mode::Real;
    config.ipv6_consent = consent;
    config.max_ipv6_addresses = 3; // Start with a small number for testing

    let manager = AntiBlockingManager::new(config);

    println!("🚨 Initializing REAL IPv6 mode...");
    match manager.initialize().await {
        Ok(_) => {
            println!("✅ REAL IPv6 mode initialized successfully");
            
            let stats = manager.get_stats().await;
            println!("Real IPv6 Addresses Created: {}", stats.ipv6_addresses_available);
            
            // Test requests with real IPv6 rotation
            println!("\n📡 Testing HTTP requests with REAL IPv6 rotation...");
            for i in 1..=3 {
                println!("Request {}/3:", i);
                match manager.make_request("https://httpbin.org/ip").await {
                    Ok(response) => {
                        let text = response.text().await?;
                        println!("  ✅ Success: {}", text);
                    }
                    Err(e) => {
                        println!("  ❌ Failed: {}", e);
                    }
                }
                
                if i < 3 {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }

            // Show safety report
            println!("\n📋 Safety Report:");
            let safety_report = manager.get_safety_report().await;
            println!("{}", safety_report);

            // Critical cleanup
            println!("\n🚨 CRITICAL: Performing cleanup to restore network configuration...");
            manager.cleanup().await?;
            println!("✅ REAL IPv6 mode cleanup completed - network configuration restored");
        }
        Err(e) => {
            println!("❌ REAL IPv6 mode initialization failed: {}", e);
            println!("This is expected if you don't have administrator privileges or IPv6 support.");
        }
    }

    Ok(())
}
