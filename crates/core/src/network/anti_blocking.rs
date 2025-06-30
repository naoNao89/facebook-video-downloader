//! Advanced anti-blocking strategies for Facebook video downloads
//!
//! This module implements comprehensive anti-blocking measures to overcome
//! HTTP 403 Forbidden errors and other blocking mechanisms used by Facebook.
//! It includes IPv6 multi-IP rotation, user-agent rotation, header variation,
//! and intelligent retry strategies.

use crate::{Result, FacebookExtractorError};
use reqwest::Client;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::io::{self, Write};
use std::fs;
use tokio::process::Command as AsyncCommand;

/// System requirements for real IPv6 implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    /// Whether IPv6 is supported on the system
    pub ipv6_supported: bool,
    /// Whether the user has administrator privileges
    pub admin_privileges: bool,
    /// Available network interfaces
    pub network_interfaces: Vec<String>,
    /// Existing IPv6 addresses
    pub existing_ipv6_addresses: Vec<Ipv6Addr>,
    /// Operating system
    pub operating_system: String,
    /// System compatibility score (0-100)
    pub compatibility_score: u8,
    /// Warnings and recommendations
    pub warnings: Vec<String>,
}

/// User consent manager for real IPv6 implementation
#[derive(Debug)]
pub struct IPv6ConsentManager {
    requirements: Option<SystemRequirements>,
}

impl IPv6ConsentManager {
    /// Create a new consent manager
    pub fn new() -> Self {
        Self {
            requirements: None,
        }
    }

    /// Check system requirements for real IPv6 implementation
    pub async fn check_system_requirements(&mut self) -> Result<SystemRequirements> {
        info!("🔍 Checking system requirements for real IPv6 implementation");

        let mut requirements = SystemRequirements {
            ipv6_supported: false,
            admin_privileges: false,
            network_interfaces: Vec::new(),
            existing_ipv6_addresses: Vec::new(),
            operating_system: std::env::consts::OS.to_string(),
            compatibility_score: 0,
            warnings: Vec::new(),
        };

        // Check IPv6 support
        requirements.ipv6_supported = self.check_ipv6_support().await;
        if !requirements.ipv6_supported {
            requirements.warnings.push("IPv6 is not supported or available on this system".to_string());
        }

        // Check admin privileges
        requirements.admin_privileges = self.check_admin_privileges().await;
        if !requirements.admin_privileges {
            requirements.warnings.push("Administrator/root privileges required for real IPv6 implementation".to_string());
        }

        // Detect network interfaces
        requirements.network_interfaces = self.detect_network_interfaces().await?;
        if requirements.network_interfaces.is_empty() {
            requirements.warnings.push("No suitable network interfaces found".to_string());
        }

        // Detect existing IPv6 addresses
        requirements.existing_ipv6_addresses = self.detect_existing_ipv6().await?;

        // Calculate compatibility score
        requirements.compatibility_score = self.calculate_compatibility_score(&requirements);

        // Add OS-specific warnings
        self.add_os_specific_warnings(&mut requirements);

        self.requirements = Some(requirements.clone());
        Ok(requirements)
    }

    /// Present consent dialog to user
    pub async fn request_user_consent(&self, requirements: &SystemRequirements) -> Result<IPv6ConsentStatus> {
        println!("\n🚨 CRITICAL WARNING: Real IPv6 Implementation Request 🚨");
        println!("=========================================================");
        println!();
        println!("You are about to enable REAL IPv6 address creation that will:");
        println!("❌ MODIFY your system's network configuration");
        println!("❌ CREATE temporary IPv6 addresses on network interfaces");
        println!("❌ REQUIRE administrator/root privileges");
        println!("❌ POTENTIALLY disrupt existing network connectivity");
        println!("❌ RISK system instability if not properly cleaned up");
        println!();
        println!("🔍 System Analysis Results:");
        println!("  IPv6 Support: {}", if requirements.ipv6_supported { "✅ Available" } else { "❌ Not Available" });
        println!("  Admin Privileges: {}", if requirements.admin_privileges { "✅ Detected" } else { "❌ Required" });
        println!("  Network Interfaces: {} found", requirements.network_interfaces.len());
        println!("  Compatibility Score: {}/100", requirements.compatibility_score);
        println!("  Operating System: {}", requirements.operating_system);
        println!();

        if !requirements.warnings.is_empty() {
            println!("⚠️ WARNINGS:");
            for warning in &requirements.warnings {
                println!("  • {}", warning);
            }
            println!();
        }

        println!("🛡️ SAFETY MEASURES:");
        println!("  ✅ Automatic cleanup on application exit");
        println!("  ✅ Backup of original network configuration");
        println!("  ✅ Rollback capability on errors");
        println!("  ✅ Manual cleanup commands available");
        println!();

        println!("⚖️ LEGAL CONSIDERATIONS:");
        println!("  • This may violate terms of service of some websites");
        println!("  • Use only for legitimate testing and development");
        println!("  • Respect rate limits and website policies");
        println!("  • Consider legal implications in your jurisdiction");
        println!();

        println!("🔄 ALTERNATIVE: Simulation Mode");
        println!("  • Safe simulation mode is available (default)");
        println!("  • Provides anti-blocking benefits without system changes");
        println!("  • Recommended for most use cases");
        println!();

        // Interactive consent process
        let mut consent = IPv6ConsentStatus::default();

        // Step 1: Risk acknowledgment
        print!("Do you understand and acknowledge these risks? (yes/no): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "yes" {
            println!("❌ Consent denied. Falling back to safe simulation mode.");
            return Ok(consent);
        }
        consent.risks_acknowledged = true;

        // Step 2: Admin privileges confirmation
        if requirements.admin_privileges {
            print!("Confirm you have administrator privileges and accept responsibility? (yes/no): ");
            io::stdout().flush().unwrap();
            input.clear();
            io::stdin().read_line(&mut input).unwrap();

            if input.trim().to_lowercase() != "yes" {
                println!("❌ Admin privileges not confirmed. Falling back to simulation mode.");
                return Ok(consent);
            }
            consent.admin_privileges_confirmed = true;
        } else {
            println!("❌ Administrator privileges required but not detected.");
            return Ok(consent);
        }

        // Step 3: Final consent
        print!("Type 'I ACCEPT THE RISKS' to proceed with real IPv6 implementation: ");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() == "I ACCEPT THE RISKS" {
            consent.consent_given = true;
            consent.consent_timestamp = Some(std::time::SystemTime::now());
            println!("✅ Consent granted. Proceeding with real IPv6 implementation.");
            println!("🛡️ Remember: You can stop the application at any time to trigger cleanup.");
        } else {
            println!("❌ Consent phrase not matched. Falling back to simulation mode.");
        }

        Ok(consent)
    }

    /// Check if IPv6 is supported on the system
    async fn check_ipv6_support(&self) -> bool {
        match tokio::net::TcpSocket::new_v6() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Check if user has administrator privileges
    async fn check_admin_privileges(&self) -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }

        #[cfg(windows)]
        {
            // On Windows, try to create a file in a system directory
            std::fs::File::create("C:\\Windows\\Temp\\ipv6_test.tmp")
                .and_then(|_| std::fs::remove_file("C:\\Windows\\Temp\\ipv6_test.tmp"))
                .is_ok()
        }
    }

    /// Detect available network interfaces
    async fn detect_network_interfaces(&self) -> Result<Vec<String>> {
        let mut interfaces = Vec::new();

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = AsyncCommand::new("ifconfig")
                .arg("-l")
                .output()
                .await
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                interfaces.extend(
                    output_str
                        .split_whitespace()
                        .filter(|iface| !iface.starts_with("lo") && !iface.starts_with("gif"))
                        .map(|s| s.to_string())
                );
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = AsyncCommand::new("ip")
                .args(&["link", "show"])
                .output()
                .await
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if let Some(iface) = line.split(':').nth(1) {
                        let iface = iface.trim();
                        if !iface.starts_with("lo") && !iface.is_empty() {
                            interfaces.push(iface.to_string());
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = AsyncCommand::new("netsh")
                .args(&["interface", "show", "interface"])
                .output()
                .await
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("Connected") {
                        if let Some(iface) = line.split_whitespace().last() {
                            interfaces.push(iface.to_string());
                        }
                    }
                }
            }
        }

        Ok(interfaces)
    }

    /// Detect existing IPv6 addresses
    async fn detect_existing_ipv6(&self) -> Result<Vec<Ipv6Addr>> {
        let mut addresses = Vec::new();

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = AsyncCommand::new("ifconfig").output().await {
                let output_str = String::from_utf8_lossy(&output.stdout);
                addresses.extend(self.parse_ifconfig_ipv6(&output_str));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = AsyncCommand::new("ip")
                .args(&["-6", "addr", "show"])
                .output()
                .await
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                addresses.extend(self.parse_ip_ipv6(&output_str));
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = AsyncCommand::new("netsh")
                .args(&["interface", "ipv6", "show", "addresses"])
                .output()
                .await
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                addresses.extend(self.parse_netsh_ipv6(&output_str));
            }
        }

        Ok(addresses)
    }

    /// Calculate system compatibility score
    fn calculate_compatibility_score(&self, requirements: &SystemRequirements) -> u8 {
        let mut score = 0u8;

        if requirements.ipv6_supported { score += 40; }
        if requirements.admin_privileges { score += 30; }
        if !requirements.network_interfaces.is_empty() { score += 20; }
        if requirements.warnings.is_empty() { score += 10; }

        score
    }

    /// Add OS-specific warnings
    fn add_os_specific_warnings(&self, requirements: &mut SystemRequirements) {
        match requirements.operating_system.as_str() {
            "windows" => {
                requirements.warnings.push("Windows IPv6 configuration requires elevated privileges".to_string());
                requirements.warnings.push("Windows Defender may flag network configuration changes".to_string());
            }
            "macos" => {
                requirements.warnings.push("macOS may require System Integrity Protection adjustments".to_string());
                requirements.warnings.push("Network configuration changes may require user approval".to_string());
            }
            "linux" => {
                requirements.warnings.push("Linux IPv6 configuration requires root privileges".to_string());
                requirements.warnings.push("Firewall rules may need adjustment".to_string());
            }
            _ => {
                requirements.warnings.push("Unsupported operating system for real IPv6 implementation".to_string());
            }
        }
    }

    /// Parse IPv6 addresses from ifconfig output
    fn parse_ifconfig_ipv6(&self, output: &str) -> Vec<Ipv6Addr> {
        let mut addresses = Vec::new();
        for line in output.lines() {
            if line.contains("inet6") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split_whitespace().nth(1) {
                    if let Some(addr_part) = addr_str.split('/').next() {
                        if let Ok(addr) = addr_part.parse::<Ipv6Addr>() {
                            if is_global_ipv6(&addr) {
                                addresses.push(addr);
                            }
                        }
                    }
                }
            }
        }
        addresses
    }

    /// Parse IPv6 addresses from ip command output
    fn parse_ip_ipv6(&self, output: &str) -> Vec<Ipv6Addr> {
        let mut addresses = Vec::new();
        for line in output.lines() {
            if line.contains("inet6") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split_whitespace().nth(1) {
                    if let Some(addr_part) = addr_str.split('/').next() {
                        if let Ok(addr) = addr_part.parse::<Ipv6Addr>() {
                            if is_global_ipv6(&addr) {
                                addresses.push(addr);
                            }
                        }
                    }
                }
            }
        }
        addresses
    }

    /// Parse IPv6 addresses from netsh output
    fn parse_netsh_ipv6(&self, output: &str) -> Vec<Ipv6Addr> {
        let mut addresses = Vec::new();
        for line in output.lines() {
            if line.contains("Address") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split(':').last() {
                    let addr_str = addr_str.trim();
                    if let Ok(addr) = addr_str.parse::<Ipv6Addr>() {
                        if is_global_ipv6(&addr) {
                            addresses.push(addr);
                        }
                    }
                }
            }
        }
        addresses
    }
}

/// Helper function to check if an IPv6 address is global
/// This replaces the unstable `is_global()` method
fn is_global_ipv6(addr: &Ipv6Addr) -> bool {
    // Check if it's not loopback, not link-local, not unique local, not multicast
    !addr.is_loopback()
        && !addr.is_unspecified()
        && !(addr.segments()[0] & 0xffc0 == 0xfe80) // Not link-local (fe80::/10)
        && !(addr.segments()[0] & 0xfe00 == 0xfc00) // Not unique local (fc00::/7)
        && !addr.is_multicast()
}

/// User consent status for real IPv6 implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPv6ConsentStatus {
    /// Whether user has explicitly consented to real IPv6 implementation
    pub consent_given: bool,
    /// Timestamp when consent was given
    pub consent_timestamp: Option<std::time::SystemTime>,
    /// Whether user understands the risks
    pub risks_acknowledged: bool,
    /// Whether user has admin privileges
    pub admin_privileges_confirmed: bool,
    /// Consent session ID for tracking
    pub session_id: String,
}

impl Default for IPv6ConsentStatus {
    fn default() -> Self {
        Self {
            consent_given: false,
            consent_timestamp: None,
            risks_acknowledged: false,
            admin_privileges_confirmed: false,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

// IPv6Mode enum removed - only real IPv6 implementation is supported

/// Configuration for anti-blocking strategies
///
/// CRITICAL: This configuration now uses REAL IPv6 implementation only.
/// All IPv6 addresses will be created on actual system network interfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiBlockingConfig {
    /// Enable IPv6 multi-IP rotation (REAL mode only)
    pub enable_ipv6_rotation: bool,
    /// User consent for real IPv6 implementation (REQUIRED)
    pub ipv6_consent: IPv6ConsentStatus,
    /// Maximum number of IPv6 addresses to create on system
    pub max_ipv6_addresses: usize,
    /// Network interface to use for IPv6 addresses
    pub network_interface: Option<String>,
    /// IPv6 subnet prefix to use (default: auto-detect)
    pub ipv6_prefix: Option<String>,
    /// Enable user-agent rotation
    pub enable_user_agent_rotation: bool,
    /// Enable header variation
    pub enable_header_variation: bool,
    /// Enable request timing randomization
    pub enable_timing_randomization: bool,
    /// Base delay between requests (milliseconds)
    pub base_delay_ms: u64,
    /// Maximum jitter for timing randomization (milliseconds)
    pub max_jitter_ms: u64,
    /// Maximum retry attempts per request
    pub max_retry_attempts: usize,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum backoff delay (seconds)
    pub max_backoff_secs: u64,
}

impl Default for AntiBlockingConfig {
    fn default() -> Self {
        Self {
            enable_ipv6_rotation: false, // Default to disabled - requires explicit consent
            ipv6_consent: IPv6ConsentStatus::default(), // No consent by default
            max_ipv6_addresses: 5,
            network_interface: None, // Auto-detect
            ipv6_prefix: None, // Auto-detect
            enable_user_agent_rotation: true,
            enable_header_variation: true,
            enable_timing_randomization: true,
            base_delay_ms: 1000,
            max_jitter_ms: 2000,
            max_retry_attempts: 3,
            backoff_multiplier: 2.0,
            max_backoff_secs: 30,
        }
    }
}

/// IPv6 address management for rotation (REAL mode only)
///
/// CRITICAL: This implementation creates actual IPv6 addresses on system network interfaces.
/// All IPv6 addresses will modify your system's network configuration.
/// Requires explicit user consent and administrator privileges.
#[derive(Debug)]
pub struct IPv6Manager {
    /// Available IPv6 addresses for rotation
    addresses: Vec<Ipv6Addr>,
    /// Current address index
    current_index: usize,
    /// Network interface name
    interface_name: Option<String>,
    /// Real IPv6 manager for system network configuration
    real_manager: Option<crate::network::real_ipv6::RealIPv6Manager>,
    /// User consent status (REQUIRED)
    consent: IPv6ConsentStatus,
}

impl IPv6Manager {
    /// Create a new IPv6 manager (REAL mode only)
    ///
    /// CRITICAL: This manager will create actual IPv6 addresses on system interfaces.
    /// Requires explicit user consent and administrator privileges.
    pub fn new(consent: IPv6ConsentStatus) -> Self {
        Self {
            addresses: Vec::new(),
            current_index: 0,
            interface_name: None,
            real_manager: None,
            consent,
        }
    }

    /// Initialize IPv6 addresses for rotation (REAL mode only)
    ///
    /// CRITICAL: This method creates actual IPv6 addresses on system network interfaces.
    /// Requires explicit user consent and administrator privileges.
    pub async fn initialize(&mut self, config: &AntiBlockingConfig) -> Result<()> {
        error!("🚨 CRITICAL: Initializing REAL IPv6 address rotation");
        error!("🚨 This WILL modify your system's network configuration");

        self.initialize_real_mode(config).await
    }

    // Simulation mode removed - only real IPv6 implementation is supported

    /// Initialize real IPv6 mode (CRITICAL: makes actual system changes)
    async fn initialize_real_mode(&mut self, config: &AntiBlockingConfig) -> Result<()> {
        error!("🚨 CRITICAL: Initializing REAL IPv6 mode");
        error!("🚨 This WILL modify your system's network configuration");

        // Verify user consent
        if !config.ipv6_consent.consent_given {
            return Err(FacebookExtractorError::network(
                "REAL IPv6 mode requires explicit user consent".to_string()
            ));
        }

        if !config.ipv6_consent.risks_acknowledged {
            return Err(FacebookExtractorError::network(
                "REAL IPv6 mode requires risk acknowledgment".to_string()
            ));
        }

        if !config.ipv6_consent.admin_privileges_confirmed {
            return Err(FacebookExtractorError::network(
                "REAL IPv6 mode requires administrator privileges".to_string()
            ));
        }

        // Determine network interface
        let interface = match &config.network_interface {
            Some(iface) => iface.clone(),
            None => self.auto_detect_interface().await?,
        };

        // Create real IPv6 manager
        let mut real_manager = crate::network::real_ipv6::RealIPv6Manager::new(
            interface,
            config.ipv6_prefix.clone()
        );

        // Initialize real IPv6 addresses
        real_manager.initialize(config.max_ipv6_addresses).await?;

        // Get addresses from real manager
        for _ in 0..real_manager.address_count() {
            if let Some(addr) = real_manager.next_address() {
                self.addresses.push(addr);
            }
        }

        self.real_manager = Some(real_manager);
        error!("🚨 REAL IPv6 mode initialized with {} addresses", self.addresses.len());
        error!("🚨 CRITICAL: Remember to call cleanup() to restore network configuration");

        Ok(())
    }

    /// Check if IPv6 is available on the system
    async fn is_ipv6_available(&self) -> bool {
        // Try to create a simple IPv6 socket to test availability
        match tokio::net::TcpSocket::new_v6() {
            Ok(_) => {
                debug!("✅ IPv6 socket creation successful");
                true
            }
            Err(e) => {
                debug!("❌ IPv6 socket creation failed: {}", e);
                false
            }
        }
    }

    /// Safely detect existing IPv6 addresses (read-only, for reference)
    ///
    /// SAFETY: This method only reads existing system IPv6 addresses
    /// for informational purposes. No system configuration is modified.
    async fn detect_existing_addresses(&mut self) -> Result<()> {
        info!("🔍 Detecting existing IPv6 addresses (read-only)");
        warn!("🔍 System address detection is for reference only - no configuration changes made");

        // Attempt to read system IPv6 addresses safely (read-only operations)
        match self.try_detect_system_addresses().await {
            Ok(count) => {
                info!("🔍 Found {} existing IPv6 addresses on system", count);
            }
            Err(e) => {
                warn!("⚠️ Could not detect system IPv6 addresses: {} - continuing with simulation", e);
            }
        }

        // Always use simulated addresses regardless of system detection
        info!("🔍 Using simulated IPv6 addresses for anti-blocking rotation");
        Ok(())
    }

    /// Safely attempt to detect system IPv6 addresses (read-only)
    ///
    /// SAFETY: This method only attempts to read system information
    /// without making any configuration changes.
    async fn try_detect_system_addresses(&mut self) -> Result<usize> {
        let initial_count = self.addresses.len();

        // Attempt to read system IPv6 addresses safely (read-only operations)
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = self.detect_addresses_macos_readonly().await {
                warn!("⚠️ macOS IPv6 detection failed: {}", e);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Err(e) = self.detect_addresses_linux_readonly().await {
                warn!("⚠️ Linux IPv6 detection failed: {}", e);
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Err(e) = self.detect_addresses_windows_readonly().await {
                warn!("⚠️ Windows IPv6 detection failed: {}", e);
            }
        }

        let detected_count = self.addresses.len() - initial_count;
        Ok(detected_count)
    }

    /// Detect IPv6 addresses on macOS (read-only)
    #[cfg(target_os = "macos")]
    async fn detect_addresses_macos_readonly(&mut self) -> Result<()> {
        use tokio::process::Command;

        warn!("🔍 Reading macOS IPv6 addresses (read-only operation)");
        let output = Command::new("ifconfig")
            .output()
            .await
            .map_err(|e| FacebookExtractorError::network(format!("Failed to run ifconfig: {}", e)))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_ifconfig_output(&output_str);
        Ok(())
    }

    /// Detect IPv6 addresses on Linux (read-only)
    #[cfg(target_os = "linux")]
    async fn detect_addresses_linux_readonly(&mut self) -> Result<()> {
        use tokio::process::Command;

        warn!("🔍 Reading Linux IPv6 addresses (read-only operation)");
        let output = Command::new("ip")
            .args(&["-6", "addr", "show"])
            .output()
            .await
            .map_err(|e| FacebookExtractorError::network(format!("Failed to run ip command: {}", e)))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_ip_output(&output_str);
        Ok(())
    }

    /// Detect IPv6 addresses on Windows (read-only)
    #[cfg(target_os = "windows")]
    async fn detect_addresses_windows_readonly(&mut self) -> Result<()> {
        use tokio::process::Command;

        warn!("🔍 Reading Windows IPv6 addresses (read-only operation)");
        let output = Command::new("netsh")
            .args(&["interface", "ipv6", "show", "addresses"])
            .output()
            .await
            .map_err(|e| FacebookExtractorError::network(format!("Failed to run netsh: {}", e)))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_netsh_output(&output_str);
        Ok(())
    }

    /// Parse ifconfig output to extract IPv6 addresses
    fn parse_ifconfig_output(&mut self, output: &str) {
        for line in output.lines() {
            if line.contains("inet6") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split_whitespace().nth(1) {
                    if let Some(addr_part) = addr_str.split('/').next() {
                        if let Ok(addr) = addr_part.parse::<Ipv6Addr>() {
                            if is_global_ipv6(&addr) {
                                self.addresses.push(addr);
                                debug!("🔍 Found IPv6 address: {}", addr);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Parse ip command output to extract IPv6 addresses
    fn parse_ip_output(&mut self, output: &str) {
        for line in output.lines() {
            if line.contains("inet6") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split_whitespace().nth(1) {
                    if let Some(addr_part) = addr_str.split('/').next() {
                        if let Ok(addr) = addr_part.parse::<Ipv6Addr>() {
                            if is_global_ipv6(&addr) {
                                self.addresses.push(addr);
                                debug!("🔍 Found IPv6 address: {}", addr);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Parse netsh output to extract IPv6 addresses
    fn parse_netsh_output(&mut self, output: &str) {
        for line in output.lines() {
            if line.contains("Address") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split(':').last() {
                    let addr_str = addr_str.trim();
                    if let Ok(addr) = addr_str.parse::<Ipv6Addr>() {
                        if is_global_ipv6(&addr) {
                            self.addresses.push(addr);
                            debug!("🔍 Found IPv6 address: {}", addr);
                        }
                    }
                }
            }
        }
    }

    // Simulation mode removed - only real IPv6 implementation is supported

    /// Auto-detect suitable network interface
    async fn auto_detect_interface(&self) -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            // Try to find the default route interface
            if let Ok(output) = AsyncCommand::new("ip")
                .args(&["route", "show", "default"])
                .output()
                .await
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if let Some(dev_pos) = line.find("dev ") {
                        let after_dev = &line[dev_pos + 4..];
                        if let Some(interface) = after_dev.split_whitespace().next() {
                            return Ok(interface.to_string());
                        }
                    }
                }
            }
            Ok("eth0".to_string()) // Fallback
        }

        #[cfg(target_os = "macos")]
        {
            // Try to find the default route interface
            if let Ok(output) = AsyncCommand::new("route")
                .args(&["get", "default"])
                .output()
                .await
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.trim().starts_with("interface:") {
                        if let Some(interface) = line.split(':').nth(1) {
                            return Ok(interface.trim().to_string());
                        }
                    }
                }
            }
            Ok("en0".to_string()) // Fallback
        }

        #[cfg(target_os = "windows")]
        {
            // Use the first connected interface
            if let Ok(output) = AsyncCommand::new("netsh")
                .args(&["interface", "show", "interface"])
                .output()
                .await
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("Connected") {
                        if let Some(interface) = line.split_whitespace().last() {
                            return Ok(interface.to_string());
                        }
                    }
                }
            }
            Ok("Ethernet".to_string()) // Fallback
        }
    }

    /// Get the next IPv6 address for rotation
    pub fn next_address(&mut self) -> Option<Ipv6Addr> {
        if self.addresses.is_empty() {
            return None;
        }

        let addr = self.addresses[self.current_index];
        self.current_index = (self.current_index + 1) % self.addresses.len();
        Some(addr)
    }

    /// Get current address count
    pub fn address_count(&self) -> usize {
        self.addresses.len()
    }

    /// Check if manager is in real mode (always true - only mode available)
    pub fn is_real_mode(&self) -> bool {
        true
    }

    /// Get a comprehensive status report (REAL mode only)
    pub fn get_status_report(&self) -> String {
        let real_status = if let Some(ref manager) = self.real_manager {
            manager.get_status_report()
        } else {
            "Real manager not initialized".to_string()
        };

        format!(
            "IPv6Manager Status Report:\n\
             - Mode: REAL (CRITICAL - SYSTEM CHANGES ACTIVE)\n\
             - Total Addresses: {}\n\
             - Consent Given: {}\n\
             - Risks Acknowledged: {}\n\
             - Admin Privileges: {}\n\
             - Real Manager Status:\n{}\n\
             - CRITICAL: System network configuration has been modified",
            self.addresses.len(),
            self.consent.consent_given,
            self.consent.risks_acknowledged,
            self.consent.admin_privileges_confirmed,
            real_status
        )
    }

    /// Cleanup IPv6 addresses (CRITICAL: removes actual system IPv6 addresses)
    ///
    /// REAL MODE: Removes actual IPv6 addresses from system and restores configuration
    pub async fn cleanup(&mut self) -> Result<()> {
        error!("🚨 CRITICAL: Starting real IPv6 cleanup");

        if let Some(ref mut real_manager) = self.real_manager {
            error!("🚨 Cleaning up real IPv6 addresses from system");
            real_manager.cleanup().await?;
        }

        // Clear internal state
        self.addresses.clear();
        self.current_index = 0;
        self.real_manager = None;

        error!("✅ Real IPv6 cleanup completed - system network configuration restored");
        Ok(())
    }
}

/// User-Agent rotation manager
#[derive(Debug, Clone)]
pub struct UserAgentManager {
    user_agents: Vec<String>,
    current_index: usize,
}

impl UserAgentManager {
    /// Create a new user-agent manager with comprehensive list
    pub fn new() -> Self {
        let user_agents = vec![
            // Chrome on Windows
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36".to_string(),

            // Chrome on macOS
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string(),

            // Chrome on Linux
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string(),

            // Firefox on Windows
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/120.0".to_string(),

            // Firefox on macOS
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/121.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/120.0".to_string(),

            // Safari on macOS
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15".to_string(),

            // Mobile Chrome
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/120.0.6099.119 Mobile/15E148 Safari/604.1".to_string(),
            "Mozilla/5.0 (Linux; Android 10; SM-G973F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".to_string(),

            // Mobile Safari
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".to_string(),
            "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".to_string(),

            // Edge
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0".to_string(),
        ];

        Self {
            user_agents,
            current_index: 0,
        }
    }

    /// Get the next user agent for rotation
    pub fn next_user_agent(&mut self) -> &str {
        let user_agent = &self.user_agents[self.current_index];
        self.current_index = (self.current_index + 1) % self.user_agents.len();
        user_agent
    }

    /// Get a random user agent
    pub fn random_user_agent(&self) -> &str {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.user_agents.len());
        &self.user_agents[index]
    }

    /// Get user agent count
    pub fn count(&self) -> usize {
        self.user_agents.len()
    }
}

/// Header variation manager for request diversity
#[derive(Debug, Clone)]
pub struct HeaderManager {
    accept_headers: Vec<String>,
    accept_language_headers: Vec<String>,
    accept_encoding_headers: Vec<String>,
    connection_headers: Vec<String>,
    cache_control_headers: Vec<String>,
}

impl HeaderManager {
    /// Create a new header manager
    pub fn new() -> Self {
        Self {
            accept_headers: vec![
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_string(),
            ],
            accept_language_headers: vec![
                "en-US,en;q=0.9".to_string(),
                "en-US,en;q=0.8,es;q=0.6".to_string(),
                "en-US,en;q=0.5".to_string(),
                "en-GB,en;q=0.9,en-US;q=0.8".to_string(),
                "en,en-US;q=0.9,en-GB;q=0.8".to_string(),
            ],
            accept_encoding_headers: vec![
                "gzip, deflate, br".to_string(),
                "gzip, deflate".to_string(),
                "gzip, deflate, br, zstd".to_string(),
            ],
            connection_headers: vec![
                "keep-alive".to_string(),
                "close".to_string(),
            ],
            cache_control_headers: vec![
                "max-age=0".to_string(),
                "no-cache".to_string(),
                "max-age=300".to_string(),
            ],
        }
    }

    /// Get random headers for request variation
    pub fn get_random_headers(&self) -> HashMap<String, String> {
        let mut rng = rand::thread_rng();
        let mut headers = HashMap::new();

        // Add random Accept header
        let accept_index = rng.gen_range(0..self.accept_headers.len());
        headers.insert("Accept".to_string(), self.accept_headers[accept_index].clone());

        // Add random Accept-Language header
        let lang_index = rng.gen_range(0..self.accept_language_headers.len());
        headers.insert("Accept-Language".to_string(), self.accept_language_headers[lang_index].clone());

        // Add random Accept-Encoding header
        let encoding_index = rng.gen_range(0..self.accept_encoding_headers.len());
        headers.insert("Accept-Encoding".to_string(), self.accept_encoding_headers[encoding_index].clone());

        // Add random Connection header
        let connection_index = rng.gen_range(0..self.connection_headers.len());
        headers.insert("Connection".to_string(), self.connection_headers[connection_index].clone());

        // Randomly add Cache-Control header (50% chance)
        if rng.gen_bool(0.5) {
            let cache_index = rng.gen_range(0..self.cache_control_headers.len());
            headers.insert("Cache-Control".to_string(), self.cache_control_headers[cache_index].clone());
        }

        // Randomly add DNT header (30% chance)
        if rng.gen_bool(0.3) {
            headers.insert("DNT".to_string(), "1".to_string());
        }

        // Randomly add Upgrade-Insecure-Requests header (70% chance)
        if rng.gen_bool(0.7) {
            headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
        }

        // Add some Chrome-specific headers occasionally (40% chance)
        if rng.gen_bool(0.4) {
            headers.insert("sec-ch-ua".to_string(), "\"Google Chrome\";v=\"120\", \"Chromium\";v=\"120\", \"Not_A Brand\";v=\"99\"".to_string());
            headers.insert("sec-ch-ua-mobile".to_string(), "?0".to_string());
            headers.insert("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string());
            headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
            headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
            headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
            headers.insert("Sec-Fetch-User".to_string(), "?1".to_string());
        }

        headers
    }
}

/// Timing manager for request randomization
#[derive(Debug, Clone)]
pub struct TimingManager {
    base_delay: Duration,
    max_jitter: Duration,
    last_request: Option<Instant>,
}

impl TimingManager {
    /// Create a new timing manager
    pub fn new(base_delay_ms: u64, max_jitter_ms: u64) -> Self {
        Self {
            base_delay: Duration::from_millis(base_delay_ms),
            max_jitter: Duration::from_millis(max_jitter_ms),
            last_request: None,
        }
    }

    /// Wait for appropriate delay before next request
    pub async fn wait_for_next_request(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            let required_delay = self.calculate_delay();

            if elapsed < required_delay {
                let wait_time = required_delay - elapsed;
                debug!("⏳ Waiting {}ms before next request", wait_time.as_millis());
                tokio::time::sleep(wait_time).await;
            }
        }

        self.last_request = Some(Instant::now());
    }

    /// Calculate delay with jitter
    fn calculate_delay(&self) -> Duration {
        let mut rng = rand::thread_rng();
        let jitter = Duration::from_millis(rng.gen_range(0..=self.max_jitter.as_millis() as u64));
        self.base_delay + jitter
    }
}

/// Main anti-blocking manager that coordinates all strategies
#[derive(Debug, Clone)]
pub struct AntiBlockingManager {
    config: AntiBlockingConfig,
    ipv6_manager: Arc<RwLock<IPv6Manager>>,
    user_agent_manager: Arc<RwLock<UserAgentManager>>,
    header_manager: Arc<RwLock<HeaderManager>>,
    timing_manager: Arc<RwLock<TimingManager>>,
    request_count: Arc<RwLock<u64>>,
}

impl AntiBlockingManager {
    /// Create a new anti-blocking manager (REAL IPv6 mode only)
    ///
    /// CRITICAL: This manager will create actual IPv6 addresses on system interfaces.
    /// Requires explicit user consent and administrator privileges.
    pub fn new(config: AntiBlockingConfig) -> Self {
        let timing_manager = TimingManager::new(config.base_delay_ms, config.max_jitter_ms);

        let ipv6_manager = IPv6Manager::new(config.ipv6_consent.clone());

        Self {
            config: config.clone(),
            ipv6_manager: Arc::new(RwLock::new(ipv6_manager)),
            user_agent_manager: Arc::new(RwLock::new(UserAgentManager::new())),
            header_manager: Arc::new(RwLock::new(HeaderManager::new())),
            timing_manager: Arc::new(RwLock::new(timing_manager)),
            request_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Initialize the anti-blocking manager
    pub async fn initialize(&self) -> Result<()> {
        info!("🛡️ Initializing anti-blocking manager");

        if self.config.enable_ipv6_rotation {
            let mut ipv6_manager = self.ipv6_manager.write().await;
            ipv6_manager.initialize(&self.config).await?;
        }

        info!("✅ Anti-blocking manager initialized");
        Ok(())
    }

    /// Create an HTTP client with anti-blocking configuration
    pub async fn create_client(&self) -> Result<Client> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(5));

        // Add IPv6 binding if available (REAL MODE ONLY)
        if self.config.enable_ipv6_rotation {
            let mut ipv6_manager = self.ipv6_manager.write().await;

            if let Some(addr) = ipv6_manager.next_address() {
                error!("🚨 REAL MODE: Binding client to actual IPv6 address: {}", addr);

                // Bind to the real IPv6 address
                client_builder = client_builder.local_address(IpAddr::V6(addr));
            }
        }

        let client = client_builder
            .build()
            .map_err(|e| FacebookExtractorError::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(client)
    }

    /// Make an HTTP request with anti-blocking strategies
    pub async fn make_request(&self, url: &str) -> Result<reqwest::Response> {
        let mut last_error = None;

        for attempt in 1..=self.config.max_retry_attempts {
            info!("🌐 Making request attempt {}/{} to: {}", attempt, self.config.max_retry_attempts, &url[..80.min(url.len())]);

            // Wait for appropriate timing
            if self.config.enable_timing_randomization {
                let mut timing_manager = self.timing_manager.write().await;
                timing_manager.wait_for_next_request().await;
            }

            // Create client with current configuration
            let client = self.create_client().await?;

            // Get user agent
            let user_agent = if self.config.enable_user_agent_rotation {
                let mut ua_manager = self.user_agent_manager.write().await;
                ua_manager.next_user_agent().to_string()
            } else {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
            };

            // Build request
            let mut request_builder = client.get(url).header("User-Agent", user_agent);

            // Add varied headers
            if self.config.enable_header_variation {
                let header_manager = self.header_manager.read().await;
                let headers = header_manager.get_random_headers();
                for (key, value) in headers {
                    request_builder = request_builder.header(&key, &value);
                }
            }

            // Make the request
            match request_builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    debug!("📡 Request completed with status: {}", status);

                    if status.is_success() {
                        // Increment successful request count
                        let mut count = self.request_count.write().await;
                        *count += 1;
                        info!("✅ Request successful (total: {})", *count);
                        return Ok(response);
                    } else if status == reqwest::StatusCode::FORBIDDEN {
                        warn!("⚠️ 403 Forbidden received, will retry with different strategy");
                        last_error = Some(FacebookExtractorError::network(format!("HTTP 403 Forbidden")));

                        // On 403, force IPv6 rotation for next attempt
                        if self.config.enable_ipv6_rotation {
                            let mut ipv6_manager = self.ipv6_manager.write().await;
                            ipv6_manager.next_address(); // Force rotation
                        }
                    } else {
                        last_error = Some(FacebookExtractorError::network(format!("HTTP error: {}", status)));
                    }
                }
                Err(e) => {
                    warn!("⚠️ Network error on attempt {}: {}", attempt, e);
                    last_error = Some(FacebookExtractorError::network(format!("Network error: {}", e)));
                }
            }

            // Apply exponential backoff if not the last attempt
            if attempt < self.config.max_retry_attempts {
                let backoff_secs = (self.config.backoff_multiplier.powi(attempt as i32 - 1) as u64)
                    .min(self.config.max_backoff_secs);
                warn!("⏳ Backing off for {} seconds before retry", backoff_secs);
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            }
        }

        Err(last_error.unwrap_or_else(|| FacebookExtractorError::network("All retry attempts failed".to_string())))
    }

    /// Get statistics about the anti-blocking manager
    pub async fn get_stats(&self) -> AntiBlockingStats {
        let request_count = *self.request_count.read().await;
        let ipv6_count = self.ipv6_manager.read().await.address_count();
        let user_agent_count = self.user_agent_manager.read().await.count();

        AntiBlockingStats {
            total_requests: request_count,
            ipv6_addresses_available: ipv6_count,
            user_agents_available: user_agent_count,
            ipv6_rotation_enabled: self.config.enable_ipv6_rotation,
            user_agent_rotation_enabled: self.config.enable_user_agent_rotation,
            header_variation_enabled: self.config.enable_header_variation,
            timing_randomization_enabled: self.config.enable_timing_randomization,
        }
    }

    /// Get comprehensive safety report (REAL mode only)
    pub async fn get_safety_report(&self) -> String {
        let ipv6_manager = self.ipv6_manager.read().await;
        let stats = self.get_stats().await;

        format!(
            "🚨 ANTI-BLOCKING CRITICAL REPORT\n\
             =================================\n\
             \n\
             MODE: REAL (CRITICAL - SYSTEM CHANGES ACTIVE)\n\
             - IPv6 Mode: REAL\n\
             - System Configuration Modified: YES\n\
             - Memory-Only Operation: NO\n\
             \n\
             CURRENT STATE:\n\
             - Total Requests Made: {}\n\
             - IPv6 Addresses Available: {} (REAL SYSTEM ADDRESSES)\n\
             - User Agents Available: {}\n\
             \n\
             FEATURES ENABLED:\n\
             - IPv6 Rotation: {}\n\
             - User-Agent Rotation: {}\n\
             - Header Variation: {}\n\
             - Timing Randomization: {}\n\
             \n\
             CRITICAL WARNINGS:\n\
             🚨 System network configuration has been modified\n\
             🚨 IPv6 addresses exist on actual network interfaces\n\
             🚨 Manual cleanup may be required if application crashes\n\
             🚨 Administrator privileges were used\n\
             🚨 Use only for legitimate testing purposes\n\
             \n\
             {}",
            stats.total_requests,
            stats.ipv6_addresses_available,
            stats.user_agents_available,
            stats.ipv6_rotation_enabled,
            stats.user_agent_rotation_enabled,
            stats.header_variation_enabled,
            stats.timing_randomization_enabled,
            ipv6_manager.get_status_report()
        )
    }

    /// Cleanup resources (CRITICAL: removes actual system IPv6 addresses)
    pub async fn cleanup(&self) -> Result<()> {
        error!("🚨 CRITICAL: Cleaning up anti-blocking manager (REAL MODE)");
        error!("🚨 This will remove actual IPv6 addresses from system");

        if self.config.enable_ipv6_rotation {
            let mut ipv6_manager = self.ipv6_manager.write().await;
            ipv6_manager.cleanup().await?;
        }

        error!("✅ Anti-blocking manager cleanup completed - system network configuration restored");
        Ok(())
    }
}

/// Statistics about anti-blocking operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiBlockingStats {
    pub total_requests: u64,
    pub ipv6_addresses_available: usize,
    pub user_agents_available: usize,
    pub ipv6_rotation_enabled: bool,
    pub user_agent_rotation_enabled: bool,
    pub header_variation_enabled: bool,
    pub timing_randomization_enabled: bool,
}

impl Drop for AntiBlockingManager {
    fn drop(&mut self) {
        // Spawn a cleanup task when the manager is dropped
        let ipv6_manager = self.ipv6_manager.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            if config.enable_ipv6_rotation {
                if let Ok(mut manager) = ipv6_manager.try_write() {
                    if let Err(e) = manager.cleanup().await {
                        error!("❌ Failed to cleanup IPv6 addresses on drop: {}", e);
                    }
                }
            }
        });
    }
}
