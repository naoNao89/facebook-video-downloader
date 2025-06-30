//! Anti-blocking command implementations
//!
//! This module contains CLI command implementations for testing and debugging
//! the anti-blocking functionality.

use facebook_video_downloader_core::{AntiBlockingConfig, AntiBlockingManager, AntiBlockingStats};
use std::time::{Duration, Instant};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "anti-blocking")]
#[command(about = "Anti-blocking system commands")]
pub struct AntiBlockingCommand {
    #[command(subcommand)]
    pub command: AntiBlockingSubcommands,
}

#[derive(Subcommand)]
pub enum AntiBlockingSubcommands {
    /// Test anti-blocking functionality
    Test {
        /// Target URL to test against
        #[arg(short, long, default_value = "https://httpbin.org/ip")]
        url: String,
        
        /// Number of requests to make
        #[arg(short, long, default_value = "5")]
        requests: usize,
        
        /// Enable IPv6 rotation
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

    /// Show comprehensive safety report
    SafetyReport,
    
    /// Test IPv6 capabilities
    Ipv6Test {
        /// Number of IPv6 addresses to attempt
        #[arg(short, long, default_value = "3")]
        addresses: usize,
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
        AntiBlockingSubcommands::SafetyReport => {
            show_safety_report().await
        }
        AntiBlockingSubcommands::Ipv6Test { addresses } => {
            test_ipv6_capabilities(addresses).await
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
    println!("🛡️ Testing Anti-Blocking System");
    println!("================================");
    println!("Target URL: {}", url);
    println!("Requests: {}", requests);
    println!("IPv6 Rotation: {}", if ipv6 { "✅" } else { "❌" });
    println!("User-Agent Rotation: {}", if user_agent { "✅" } else { "❌" });
    println!("Header Variation: {}", if headers { "✅" } else { "❌" });
    println!("Timing Randomization: {}", if timing { "✅" } else { "❌" });
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

async fn test_ipv6_capabilities(addresses: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Testing IPv6 Capabilities");
    println!("=============================");
    println!("Requested addresses: {}", addresses);
    println!();
    
    let mut config = AntiBlockingConfig::default();
    config.enable_ipv6_rotation = true;
    config.max_ipv6_addresses = addresses;
    config.enable_user_agent_rotation = false;
    config.enable_header_variation = false;
    config.enable_timing_randomization = false;
    config.max_retry_attempts = 1;
    
    let manager = AntiBlockingManager::new(config);
    
    match manager.initialize().await {
        Ok(_) => {
            let stats = manager.get_stats().await;
            println!("✅ IPv6 initialization successful");
            println!("Available IPv6 addresses: {}", stats.ipv6_addresses_available);
            
            if stats.ipv6_addresses_available > 0 {
                println!("\n🧪 Testing IPv6 requests...");
                
                // Test multiple requests to see IP rotation
                for i in 1..=std::cmp::min(addresses, 3) {
                    println!("Request {}: ", i);
                    match manager.make_request("https://httpbin.org/ip").await {
                        Ok(response) => {
                            let text = response.text().await?;
                            println!("  Response: {}", text);
                        }
                        Err(e) => {
                            println!("  ❌ Failed: {}", e);
                        }
                    }
                    
                    if i < 3 {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            } else {
                println!("⚠️ No IPv6 addresses available for rotation");
            }
            
            let _ = manager.cleanup().await;
        }
        Err(e) => {
            println!("❌ IPv6 test failed: {}", e);
            println!("This is expected if your system doesn't support IPv6");
        }
    }
    
    Ok(())
}

async fn show_safety_report() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ Anti-Blocking System Safety Report");
    println!("======================================");
    println!();

    let config = AntiBlockingConfig::default();
    let manager = AntiBlockingManager::new(config);

    match manager.initialize().await {
        Ok(_) => {
            let safety_report = manager.get_safety_report().await;
            println!("{}", safety_report);

            let _ = manager.cleanup().await;
        }
        Err(e) => {
            println!("❌ Failed to initialize anti-blocking manager: {}", e);
            println!();
            println!("🛡️ SAFETY GUARANTEES (Even on Failure):");
            println!("✅ No system network configuration changes attempted");
            println!("✅ All operations are memory-only");
            println!("✅ No administrator privileges required");
            println!("✅ Safe for testing and demonstration");
        }
    }

    Ok(())
}

fn print_stats(stats: &AntiBlockingStats) {
    println!("  Total Requests: {}", stats.total_requests);
    println!("  IPv6 Addresses: {} (simulated)", stats.ipv6_addresses_available);
    println!("  User Agents: {}", stats.user_agents_available);
    println!("  Features Enabled:");
    println!("    IPv6 Rotation: {}", if stats.ipv6_rotation_enabled { "✅" } else { "❌" });
    println!("    User-Agent Rotation: {}", if stats.user_agent_rotation_enabled { "✅" } else { "❌" });
    println!("    Header Variation: {}", if stats.header_variation_enabled { "✅" } else { "❌" });
    println!("    Timing Randomization: {}", if stats.timing_randomization_enabled { "✅" } else { "❌" });
}
