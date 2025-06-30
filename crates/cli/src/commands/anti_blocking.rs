//! Anti-blocking command implementations
//!
//! This module contains CLI command implementations for testing and debugging
//! the anti-blocking functionality.

use facebook_video_downloader_core::network::{AntiBlockingConfig, AntiBlockingManager, AntiBlockingStats};
use facebook_video_downloader_core::network::anti_blocking::IPv6ConsentManager;
use std::time::{Duration, Instant};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "anti-blocking")]
#[command(about = "Anti-blocking system commands (REAL IPv6 mode only)")]
pub struct AntiBlockingCommand {
    #[command(subcommand)]
    pub command: AntiBlockingSubcommands,
}

#[derive(Subcommand)]
pub enum AntiBlockingSubcommands {
    /// Test anti-blocking functionality (REAL IPv6 mode - requires admin privileges)
    Test {
        /// Target URL to test against
        #[arg(short, long, default_value = "https://httpbin.org/ip")]
        url: String,

        /// Number of requests to make
        #[arg(short, long, default_value = "5")]
        requests: usize,

        /// Enable IPv6 rotation (REQUIRES ADMIN PRIVILEGES)
        #[arg(long)]
        ipv6: bool,

        /// Enable user-agent rotation
        #[arg(long, default_value = "true")]
        user_agent: bool,

        /// Enable header variation
        #[arg(long, default_value = "true")]
        headers: bool,

        /// Enable timing randomization
        #[arg(long)]
        timing: bool,
    },
    
    /// Show current public IP address
    Ip {
        /// Service to use for IP detection
        #[arg(short, long, default_value = "httpbin")]
        service: String,
    },

    /// Enable real IPv6 implementation (CRITICAL: requires user consent)
    EnableRealIpv6 {
        /// Network interface to use (auto-detect if not specified)
        #[arg(short, long)]
        interface: Option<String>,

        /// IPv6 prefix to use (default: 2001:db8::/64)
        #[arg(short, long)]
        prefix: Option<String>,

        /// Maximum number of IPv6 addresses to create
        #[arg(short, long, default_value = "3")]
        max_addresses: usize,

        /// Skip interactive consent (for automation - NOT RECOMMENDED)
        #[arg(long)]
        force_consent: bool,
    },
}

pub async fn handle_anti_blocking_command(cmd: AntiBlockingCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.command {
        AntiBlockingSubcommands::Test { 
            url, 
            requests, 
            ipv6, 
            user_agent, 
            headers, 
            timing 
        } => {
            test_anti_blocking(url, requests, ipv6, user_agent, headers, timing).await
        }
        AntiBlockingSubcommands::Ip { service } => {
            show_public_ip(service).await
        }
        AntiBlockingSubcommands::EnableRealIpv6 {
            interface,
            prefix,
            max_addresses,
            force_consent
        } => {
            enable_real_ipv6(interface, prefix, max_addresses, force_consent).await
        }
    }
}

async fn test_anti_blocking(
    url: String,
    requests: usize,
    ipv6: bool,
    user_agent: bool,
    headers: bool,
    timing: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚨 Testing Anti-Blocking System (REAL IPv6 MODE)");
    println!("=================================================");
    println!("Target URL: {}", url);
    println!("Requests: {}", requests);
    println!("IPv6 Rotation: {} {}", if ipv6 { "✅" } else { "❌" }, if ipv6 { "(REQUIRES ADMIN)" } else { "" });
    println!("User-Agent Rotation: {}", if user_agent { "✅" } else { "❌" });
    println!("Header Variation: {}", if headers { "✅" } else { "❌" });
    println!("Timing Randomization: {}", if timing { "✅" } else { "❌" });

    if ipv6 {
        println!();
        println!("🚨 WARNING: IPv6 rotation enabled - this will modify system network configuration!");
        println!("🚨 Requires administrator privileges and explicit user consent.");
    }
    println!();
    
    // Create configuration
    let mut config = AntiBlockingConfig::default();
    config.enable_ipv6_rotation = ipv6;
    config.enable_user_agent_rotation = user_agent;
    config.enable_header_variation = headers;
    config.enable_timing_randomization = timing;    
    // Initialize manager
    let manager = AntiBlockingManager::new(config);
    
    println!("🚀 Initializing anti-blocking manager...");
    match manager.initialize().await {
        Ok(_) => {
            println!("✅ Initialization successful");
            
            // Show initial stats
            let stats = manager.get_stats().await;
            print_stats(&stats);
            
            // Make test requests
            println!("\n🌐 Making {} test requests...", requests);
            let mut results = Vec::new();
            let start_time = Instant::now();
            
            for i in 1..=requests {
                println!("📡 Request {}/{}", i, requests);
                let request_start = Instant::now();
                
                match manager.make_request(&url).await {
                    Ok(response) => {
                        let duration = request_start.elapsed();
                        let status = response.status();
                        
                        println!("  ✅ Success: {} ({:?})", status, duration);
                        results.push((true, duration, status.as_u16()));
                    }
                    Err(e) => {
                        let duration = request_start.elapsed();
                        println!("  ❌ Failed: {} ({:?})", e, duration);
                        results.push((false, duration, 0));
                    }
                }
                
                // Small delay between requests for readability
                if i < requests {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            
            let total_duration = start_time.elapsed();
            
            // Print results summary
            println!("\n📊 Results Summary");
            println!("==================");
            let successful = results.iter().filter(|(success, _, _)| *success).count();
            let failed = results.len() - successful;
            
            println!("Total Requests: {}", results.len());
            println!("Successful: {} ({:.1}%)", successful, (successful as f64 / results.len() as f64) * 100.0);
            println!("Failed: {} ({:.1}%)", failed, (failed as f64 / results.len() as f64) * 100.0);
            println!("Total Time: {:?}", total_duration);
            
            if !results.is_empty() {
                let avg_duration: Duration = results.iter()
                    .map(|(_, duration, _)| *duration)
                    .sum::<Duration>() / results.len() as u32;
                println!("Average Request Time: {:?}", avg_duration);
                
                let requests_per_second = results.len() as f64 / total_duration.as_secs_f64();
                println!("Requests per Second: {:.2}", requests_per_second);
            }
            
            // Show final stats
            let final_stats = manager.get_stats().await;
            println!("\n📈 Final Statistics");
            println!("===================");
            print_stats(&final_stats);
            
            // Cleanup
            println!("\n🧹 Cleaning up...");
            manager.cleanup().await?;
            println!("✅ Cleanup completed");
            
        }
        Err(e) => {
            println!("❌ Initialization failed: {}", e);
            println!("This might be expected if IPv6 is not available on your system.");
        }
    }
    
    Ok(())
}

async fn show_public_ip(service: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Checking Public IP Address");
    println!("==============================");
    
    let url = match service.as_str() {
        "httpbin" => "https://httpbin.org/ip",
        "ipify" => "https://api.ipify.org?format=json",
        "ip-api" => "http://ip-api.com/json",
        _ => {
            println!("❌ Unknown service: {}", service);
            println!("Available services: httpbin, ipify, ip-api");
            return Ok(());
        }
    };
    
    println!("Service: {}", service);
    println!("URL: {}", url);
    println!();
    
    // Test with and without anti-blocking
    println!("📡 Standard Request (no anti-blocking):");
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(response) => {
            let text = response.text().await?;
            println!("Response: {}", text);
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
        }
    }
    
    println!("\n🛡️ Anti-Blocking Request:");
    let config = AntiBlockingConfig::default();
    let manager = AntiBlockingManager::new(config);
    
    match manager.initialize().await {
        Ok(_) => {
            match manager.make_request(url).await {
                Ok(response) => {
                    let text = response.text().await?;
                    println!("Response: {}", text);
                    
                    let stats = manager.get_stats().await;
                    println!("\nAnti-blocking stats:");
                    print_stats(&stats);
                }
                Err(e) => {
                    println!("❌ Failed: {}", e);
                }
            }
            let _ = manager.cleanup().await;
        }
        Err(e) => {
            println!("❌ Anti-blocking initialization failed: {}", e);
        }
    }
    
    Ok(())
}

async fn enable_real_ipv6(
    interface: Option<String>,
    prefix: Option<String>,
    max_addresses: usize,
    force_consent: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚨 CRITICAL: REAL IPv6 IMPLEMENTATION SETUP");
    println!("============================================");
    println!();
    println!("⚠️ WARNING: This will enable REAL IPv6 address creation that:");
    println!("❌ MODIFIES your system's network configuration");
    println!("❌ REQUIRES administrator/root privileges");
    println!("❌ POTENTIALLY disrupts existing network connectivity");
    println!("❌ RISKS system instability if not properly cleaned up");
    println!();

    if force_consent {
        println!("🚨 FORCE CONSENT MODE ENABLED - SKIPPING SAFETY CHECKS");
        println!("🚨 THIS IS NOT RECOMMENDED FOR PRODUCTION USE");
        println!();
    }

    // Step 1: Check system requirements
    println!("🔍 Step 1: Checking system requirements...");
    let mut consent_manager = IPv6ConsentManager::new();

    let requirements = match consent_manager.check_system_requirements().await {
        Ok(req) => {
            println!("✅ System requirements check completed");
            println!("Compatibility Score: {}/100", req.compatibility_score);
            println!("IPv6 Support: {}", if req.ipv6_supported { "✅" } else { "❌" });
            println!("Admin Privileges: {}", if req.admin_privileges { "✅" } else { "❌" });
            println!("Network Interfaces: {} found", req.network_interfaces.len());
            println!("Operating System: {}", req.operating_system);

            if !req.warnings.is_empty() {
                println!("\n⚠️ WARNINGS:");
                for warning in &req.warnings {
                    println!("  • {}", warning);
                }
            }

            if req.compatibility_score < 70 {
                println!("\n❌ Low compatibility score ({}/100)", req.compatibility_score);
                println!("Real IPv6 implementation is not recommended on this system.");
                println!("Please ensure you have administrator privileges and IPv6 support.");
                return Ok(());
            }

            req
        }
        Err(e) => {
            println!("❌ System requirements check failed: {}", e);
            println!("Real IPv6 implementation cannot be enabled.");
            return Ok(());
        }
    };

    // Step 2: Get user consent (unless forced)
    let consent = if force_consent {
        println!("\n🚨 FORCING CONSENT - BYPASSING SAFETY CHECKS");
        let mut consent = facebook_video_downloader_core::network::anti_blocking::IPv6ConsentStatus::default();
        consent.consent_given = true;
        consent.risks_acknowledged = true;
        consent.admin_privileges_confirmed = requirements.admin_privileges;
        consent.consent_timestamp = Some(std::time::SystemTime::now());
        consent
    } else {
        println!("\n🔐 Step 2: Requesting user consent...");
        match consent_manager.request_user_consent(&requirements).await {
            Ok(consent) => {
                if !consent.consent_given {
                    println!("❌ User consent not given. Real IPv6 implementation will not be enabled.");
                    println!("IPv6 rotation requires explicit consent due to system modification risks.");
                    return Ok(());
                }
                consent
            }
            Err(e) => {
                println!("❌ Consent process failed: {}", e);
                return Ok(());
            }
        }
    };

    // Step 3: Configure and test real IPv6 mode
    println!("\n🚨 Step 3: Configuring REAL IPv6 mode...");
    let mut config = AntiBlockingConfig::default();
    config.enable_ipv6_rotation = true; // Enable IPv6 rotation (REAL mode only)
    config.ipv6_consent = consent;
    config.max_ipv6_addresses = max_addresses;
    config.network_interface = interface;
    config.ipv6_prefix = prefix;

    println!("Configuration:");
    println!("  Mode: REAL (CRITICAL)");
    println!("  Max Addresses: {}", max_addresses);
    println!("  Interface: {:?}", config.network_interface);
    println!("  IPv6 Prefix: {:?}", config.ipv6_prefix);

    // Step 4: Initialize and test
    println!("\n🚨 Step 4: Initializing REAL IPv6 implementation...");
    let manager = AntiBlockingManager::new(config);

    match manager.initialize().await {
        Ok(_) => {
            println!("✅ REAL IPv6 mode initialized successfully");

            let stats = manager.get_stats().await;
            println!("Real IPv6 Addresses Created: {}", stats.ipv6_addresses_available);

            // Test with a simple request
            println!("\n📡 Testing HTTP request with REAL IPv6 rotation...");
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

            // Critical cleanup
            println!("\n🚨 CRITICAL: Performing cleanup to restore network configuration...");
            manager.cleanup().await?;
            println!("✅ REAL IPv6 mode cleanup completed - network configuration restored");

            println!("\n✅ Real IPv6 implementation test completed successfully");
            println!("🛡️ Your system network configuration has been restored");
        }
        Err(e) => {
            println!("❌ REAL IPv6 mode initialization failed: {}", e);
            println!("This is expected if you don't have administrator privileges or IPv6 support.");
            println!("Please ensure you have the required permissions and system support.");
        }
    }

    Ok(())
}

fn print_stats(stats: &AntiBlockingStats) {
    println!("  Total Requests: {}", stats.total_requests);
    println!("  IPv6 Addresses: {}", stats.ipv6_addresses_available);
    println!("  User Agents: {}", stats.user_agents_available);
    println!("  Features Enabled:");
    println!("    IPv6 Rotation: {}", if stats.ipv6_rotation_enabled { "✅" } else { "❌" });
    println!("    User-Agent Rotation: {}", if stats.user_agent_rotation_enabled { "✅" } else { "❌" });
    println!("    Header Variation: {}", if stats.header_variation_enabled { "✅" } else { "❌" });
    println!("    Timing Randomization: {}", if stats.timing_randomization_enabled { "✅" } else { "❌" });
}
