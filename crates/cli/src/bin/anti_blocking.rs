//! CLI tool for testing anti-blocking functionality with public IP addresses
//!
//! This tool allows testing the anti-blocking system against real public endpoints
//! to validate IPv6 rotation, user-agent rotation, header variation, and timing
//! randomization in a real-world scenario.

use facebook_video_downloader_core::network::{AntiBlockingConfig, AntiBlockingManager, AntiBlockingStats};
use std::time::{Duration, Instant};
use tokio;
use clap::{Parser, Subcommand};
use serde_json;

#[derive(Parser)]
#[command(name = "anti-blocking-cli")]
#[command(about = "Test anti-blocking functionality with public IP addresses")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test basic anti-blocking functionality
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
        
        /// Base delay between requests (ms)
        #[arg(long, default_value = "1000")]
        delay: u64,
        
        /// Maximum jitter (ms)
        #[arg(long, default_value = "500")]
        jitter: u64,
    },
    
    /// Show current public IP address
    Ip {
        /// Service to use for IP detection
        #[arg(short, long, default_value = "httpbin")]
        service: String,
    },
    
    /// Test IPv6 capabilities
    Ipv6Test {
        /// Number of IPv6 addresses to attempt
        #[arg(short, long, default_value = "3")]
        addresses: usize,
    },
    
    /// Performance benchmark
    Benchmark {
        /// Number of concurrent requests
        #[arg(short, long, default_value = "10")]
        concurrent: usize,

        /// Total number of requests
        #[arg(short, long, default_value = "50")]
        total: usize,
    },

    /// Show comprehensive safety report
    SafetyReport,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Test { 
            url, 
            requests, 
            ipv6, 
            user_agent, 
            headers, 
            timing, 
            delay, 
            jitter 
        } => {
            test_anti_blocking(url, requests, ipv6, user_agent, headers, timing, delay, jitter).await?;
        }
        Commands::Ip { service } => {
            show_public_ip(service).await?;
        }
        Commands::Ipv6Test { addresses } => {
            test_ipv6_capabilities(addresses).await?;
        }
        Commands::Benchmark { concurrent, total } => {
            run_benchmark(concurrent, total).await?;
        }
        Commands::SafetyReport => {
            show_safety_report().await?;
        }
    }
    
    Ok(())
}

async fn test_anti_blocking(
    url: String,
    requests: usize,
    ipv6: bool,
    user_agent: bool,
    headers: bool,
    timing: bool,
    delay: u64,
    jitter: u64,
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
    config.base_delay_ms = delay;
    config.max_jitter_ms = jitter;
    
    // Initialize manager
    let manager = AntiBlockingManager::new(config.clone());
    
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
                        
                        // Try to extract IP information if it's an IP service
                        if url.contains("httpbin.org/ip") || url.contains("ip") {
                            if let Ok(text) = response.text().await {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(origin) = json.get("origin") {
                                        println!("  🌍 Origin IP: {}", origin);
                                    }
                                }
                            }
                        }
                        
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
            println!("Average Time: {:?}", total_duration / results.len() as u32);
            
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
            
            // Try with IPv4 only
            if ipv6 {
                println!("\n🔄 Retrying with IPv4 only...");
                let mut fallback_config = config.clone();
                fallback_config.enable_ipv6_rotation = false;
                
                let fallback_manager = AntiBlockingManager::new(fallback_config);
                match fallback_manager.initialize().await {
                    Ok(_) => {
                        println!("✅ IPv4 fallback successful");
                        let stats = fallback_manager.get_stats().await;
                        print_stats(&stats);
                        let _ = fallback_manager.cleanup().await;
                    }
                    Err(e) => {
                        println!("❌ IPv4 fallback also failed: {}", e);
                    }
                }
            }
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

async fn run_benchmark(concurrent: usize, total: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Anti-Blocking Performance Benchmark");
    println!("=======================================");
    println!("Concurrent requests: {}", concurrent);
    println!("Total requests: {}", total);
    println!();
    
    let mut config = AntiBlockingConfig::default();
    config.enable_ipv6_rotation = false; // Disable for consistent benchmarking
    config.enable_user_agent_rotation = true;
    config.enable_header_variation = true;
    config.enable_timing_randomization = false; // Disable for accurate timing
    config.max_retry_attempts = 1;
    
    let manager = AntiBlockingManager::new(config);
    
    match manager.initialize().await {
        Ok(_) => {
            println!("✅ Benchmark setup complete");
            
            let start_time = Instant::now();
            let mut handles = Vec::new();
            
            // Launch concurrent requests
            for batch in 0..(total / concurrent) {
                let batch_handles: Vec<_> = (0..concurrent).map(|i| {
                    let manager_clone = manager.clone();
                    let request_id = batch * concurrent + i + 1;
                    
                    tokio::spawn(async move {
                        let request_start = Instant::now();
                        let result = manager_clone.make_request("https://httpbin.org/ip").await;
                        let duration = request_start.elapsed();
                        (request_id, result.is_ok(), duration)
                    })
                }).collect();
                
                handles.extend(batch_handles);
                
                // Small delay between batches
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            // Collect results
            let mut successful = 0;
            let mut total_request_time = Duration::ZERO;
            
            for handle in handles {
                if let Ok((id, success, duration)) = handle.await {
                    if success {
                        successful += 1;
                    }
                    total_request_time += duration;
                    
                    if id % 10 == 0 {
                        println!("Completed {} requests...", id);
                    }
                }
            }
            
            let total_duration = start_time.elapsed();
            
            // Print benchmark results
            println!("\n📊 Benchmark Results");
            println!("====================");
            println!("Total Requests: {}", total);
            println!("Successful: {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
            println!("Total Time: {:?}", total_duration);
            println!("Average Request Time: {:?}", total_request_time / total as u32);
            println!("Requests per Second: {:.2}", total as f64 / total_duration.as_secs_f64());
            
            let final_stats = manager.get_stats().await;
            println!("\nFinal Statistics:");
            print_stats(&final_stats);
            
            let _ = manager.cleanup().await;
        }
        Err(e) => {
            println!("❌ Benchmark setup failed: {}", e);
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
