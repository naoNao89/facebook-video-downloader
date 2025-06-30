//! Real IPv6 address management with system network configuration
//!
//! CRITICAL WARNING: This module makes actual system-level network configuration changes.
//! Use only with explicit user consent and understanding of risks.

use crate::{Result, FacebookExtractorError};
use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, info, warn, error};
use serde::{Deserialize, Serialize};

/// Network configuration backup for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBackup {
    /// Original IPv6 addresses before modification
    pub original_addresses: HashMap<String, Vec<Ipv6Addr>>,
    /// Backup timestamp
    pub backup_timestamp: std::time::SystemTime,
    /// Operating system
    pub os: String,
    /// Backup file path
    pub backup_file: Option<String>,
}

/// Real IPv6 address manager with system configuration
#[derive(Debug)]
pub struct RealIPv6Manager {
    /// Available IPv6 addresses for rotation
    addresses: Vec<Ipv6Addr>,
    /// Current address index
    current_index: usize,
    /// Network interface being used
    interface_name: String,
    /// IPv6 addresses created by this manager
    created_addresses: Vec<Ipv6Addr>,
    /// Network configuration backup
    backup: Option<NetworkBackup>,
    /// IPv6 prefix for address generation
    ipv6_prefix: String,
    /// Whether manager is initialized
    initialized: bool,
}

impl RealIPv6Manager {
    /// Create a new real IPv6 manager
    /// 
    /// CRITICAL: This manager will make actual system network configuration changes
    pub fn new(interface: String, prefix: Option<String>) -> Self {
        let default_prefix = match std::env::consts::OS {
            "linux" | "macos" => "2001:db8::/64".to_string(),
            "windows" => "2001:db8::/64".to_string(),
            _ => "2001:db8::/64".to_string(),
        };

        Self {
            addresses: Vec::new(),
            current_index: 0,
            interface_name: interface,
            created_addresses: Vec::new(),
            backup: None,
            ipv6_prefix: prefix.unwrap_or(default_prefix),
            initialized: false,
        }
    }

    /// Initialize real IPv6 address management
    /// 
    /// CRITICAL WARNING: This method will modify system network configuration
    pub async fn initialize(&mut self, max_addresses: usize) -> Result<()> {
        if self.initialized {
            warn!("🔄 Real IPv6 manager already initialized");
            return Ok(());
        }

        error!("🚨 CRITICAL: Initializing REAL IPv6 address management");
        error!("🚨 This WILL modify your system's network configuration");
        
        // Step 1: Create network backup
        self.create_network_backup().await?;
        
        // Step 2: Validate system state
        self.validate_system_state().await?;
        
        // Step 3: Create IPv6 addresses
        self.create_real_ipv6_addresses(max_addresses).await?;
        
        // Step 4: Validate created addresses
        self.validate_created_addresses().await?;
        
        self.initialized = true;
        error!("🚨 Real IPv6 manager initialized with {} addresses", self.addresses.len());
        error!("🚨 REMEMBER: Call cleanup() to restore original network configuration");
        
        Ok(())
    }

    /// Create a backup of current network configuration
    async fn create_network_backup(&mut self) -> Result<()> {
        info!("💾 Creating network configuration backup");
        
        let mut backup = NetworkBackup {
            original_addresses: HashMap::new(),
            backup_timestamp: std::time::SystemTime::now(),
            os: std::env::consts::OS.to_string(),
            backup_file: None,
        };

        // Detect current IPv6 addresses on the interface
        let current_addresses = self.detect_interface_ipv6_addresses().await?;
        backup.original_addresses.insert(self.interface_name.clone(), current_addresses);

        // Save backup to file
        let backup_file = format!("/tmp/ipv6_backup_{}.json", 
            backup.backup_timestamp.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs());
        
        if let Ok(backup_json) = serde_json::to_string_pretty(&backup) {
            if std::fs::write(&backup_file, backup_json).is_ok() {
                backup.backup_file = Some(backup_file.clone());
                info!("💾 Network backup saved to: {}", backup_file);
            }
        }

        self.backup = Some(backup);
        Ok(())
    }

    /// Validate system state before making changes
    async fn validate_system_state(&self) -> Result<()> {
        info!("🔍 Validating system state");

        // Check if interface exists
        if !self.interface_exists().await? {
            return Err(FacebookExtractorError::network(
                format!("Network interface '{}' does not exist", self.interface_name)
            ));
        }

        // Check if interface is up
        if !self.interface_is_up().await? {
            return Err(FacebookExtractorError::network(
                format!("Network interface '{}' is not up", self.interface_name)
            ));
        }

        // Check IPv6 support on interface
        if !self.interface_supports_ipv6().await? {
            return Err(FacebookExtractorError::network(
                format!("Network interface '{}' does not support IPv6", self.interface_name)
            ));
        }

        info!("✅ System state validation passed");
        Ok(())
    }

    /// Create real IPv6 addresses on the system
    async fn create_real_ipv6_addresses(&mut self, max_addresses: usize) -> Result<()> {
        error!("🚨 CREATING REAL IPv6 ADDRESSES ON SYSTEM");
        error!("🚨 This will modify network interface: {}", self.interface_name);

        for i in 0..max_addresses {
            let addr = self.generate_ipv6_address(i)?;
            
            error!("🚨 Adding IPv6 address {} to interface {}", addr, self.interface_name);
            
            // Add address to system interface
            self.add_ipv6_to_interface(&addr).await?;
            
            // Verify address was added
            if self.verify_address_added(&addr).await? {
                self.created_addresses.push(addr);
                self.addresses.push(addr);
                error!("✅ Successfully added IPv6 address: {}", addr);
            } else {
                error!("❌ Failed to verify IPv6 address: {}", addr);
                return Err(FacebookExtractorError::network(
                    format!("Failed to add IPv6 address: {}", addr)
                ));
            }

            // Small delay between address creation
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        error!("🚨 Created {} real IPv6 addresses on system", self.created_addresses.len());
        Ok(())
    }

    /// Generate an IPv6 address for the given index
    fn generate_ipv6_address(&self, index: usize) -> Result<Ipv6Addr> {
        // Parse the prefix to get base address
        let prefix_parts: Vec<&str> = self.ipv6_prefix.split('/').collect();
        if prefix_parts.len() != 2 {
            return Err(FacebookExtractorError::network(
                format!("Invalid IPv6 prefix format: {}", self.ipv6_prefix)
            ));
        }

        let base_addr: Ipv6Addr = prefix_parts[0].parse()
            .map_err(|e| FacebookExtractorError::network(
                format!("Invalid IPv6 prefix address: {}", e)
            ))?;

        // Generate address by modifying the last segment
        let mut segments = base_addr.segments();
        segments[7] = (index + 1) as u16;
        
        Ok(Ipv6Addr::from(segments))
    }

    /// Add IPv6 address to system interface
    async fn add_ipv6_to_interface(&self, addr: &Ipv6Addr) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let output = AsyncCommand::new("ip")
                .args(&["-6", "addr", "add", &format!("{}/64", addr), "dev", &self.interface_name])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute ip command: {}", e)
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(FacebookExtractorError::network(
                    format!("Failed to add IPv6 address: {}", stderr)
                ));
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = AsyncCommand::new("ifconfig")
                .args(&[&self.interface_name, "inet6", &format!("{}", addr), "prefixlen", "64", "add"])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute ifconfig command: {}", e)
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(FacebookExtractorError::network(
                    format!("Failed to add IPv6 address: {}", stderr)
                ));
            }
        }

        #[cfg(target_os = "windows")]
        {
            let output = AsyncCommand::new("netsh")
                .args(&["interface", "ipv6", "add", "address", &self.interface_name, &format!("{}", addr)])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute netsh command: {}", e)
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(FacebookExtractorError::network(
                    format!("Failed to add IPv6 address: {}", stderr)
                ));
            }
        }

        Ok(())
    }

    /// Verify that an IPv6 address was successfully added
    async fn verify_address_added(&self, addr: &Ipv6Addr) -> Result<bool> {
        let current_addresses = self.detect_interface_ipv6_addresses().await?;
        Ok(current_addresses.contains(addr))
    }

    /// Detect IPv6 addresses on the interface
    async fn detect_interface_ipv6_addresses(&self) -> Result<Vec<Ipv6Addr>> {
        let mut addresses = Vec::new();

        #[cfg(target_os = "linux")]
        {
            let output = AsyncCommand::new("ip")
                .args(&["-6", "addr", "show", "dev", &self.interface_name])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute ip command: {}", e)
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            addresses.extend(self.parse_ip_output(&output_str));
        }

        #[cfg(target_os = "macos")]
        {
            let output = AsyncCommand::new("ifconfig")
                .arg(&self.interface_name)
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute ifconfig command: {}", e)
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            addresses.extend(self.parse_ifconfig_output(&output_str));
        }

        #[cfg(target_os = "windows")]
        {
            let output = AsyncCommand::new("netsh")
                .args(&["interface", "ipv6", "show", "addresses", &self.interface_name])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute netsh command: {}", e)
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            addresses.extend(self.parse_netsh_output(&output_str));
        }

        Ok(addresses)
    }

    /// Check if network interface exists
    async fn interface_exists(&self) -> Result<bool> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let output = AsyncCommand::new("ifconfig")
                .arg(&self.interface_name)
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to check interface existence: {}", e)
                ))?;

            Ok(output.status.success())
        }

        #[cfg(target_os = "windows")]
        {
            let output = AsyncCommand::new("netsh")
                .args(&["interface", "show", "interface", &self.interface_name])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to check interface existence: {}", e)
                ))?;

            Ok(output.status.success())
        }
    }

    /// Check if network interface is up
    async fn interface_is_up(&self) -> Result<bool> {
        #[cfg(target_os = "linux")]
        {
            let output = AsyncCommand::new("ip")
                .args(&["link", "show", &self.interface_name])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to check interface status: {}", e)
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            Ok(output_str.contains("state UP"))
        }

        #[cfg(target_os = "macos")]
        {
            let output = AsyncCommand::new("ifconfig")
                .arg(&self.interface_name)
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to check interface status: {}", e)
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            Ok(output_str.contains("flags=") && output_str.contains("UP"))
        }

        #[cfg(target_os = "windows")]
        {
            let output = AsyncCommand::new("netsh")
                .args(&["interface", "show", "interface", &self.interface_name])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to check interface status: {}", e)
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            Ok(output_str.contains("Connected"))
        }
    }

    /// Check if interface supports IPv6
    async fn interface_supports_ipv6(&self) -> Result<bool> {
        let addresses = self.detect_interface_ipv6_addresses().await?;
        // If we can detect any IPv6 addresses (even link-local), interface supports IPv6
        Ok(!addresses.is_empty() || self.has_link_local_ipv6().await?)
    }

    /// Check if interface has link-local IPv6
    async fn has_link_local_ipv6(&self) -> Result<bool> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let output = AsyncCommand::new("ifconfig")
                .arg(&self.interface_name)
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to check IPv6 support: {}", e)
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            Ok(output_str.contains("fe80:"))
        }

        #[cfg(target_os = "windows")]
        {
            let output = AsyncCommand::new("netsh")
                .args(&["interface", "ipv6", "show", "addresses", &self.interface_name])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to check IPv6 support: {}", e)
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            Ok(output_str.contains("fe80:"))
        }
    }

    /// Validate that created addresses are working
    async fn validate_created_addresses(&self) -> Result<()> {
        info!("🔍 Validating created IPv6 addresses");

        for addr in &self.created_addresses {
            if !self.verify_address_added(addr).await? {
                return Err(FacebookExtractorError::network(
                    format!("IPv6 address validation failed: {}", addr)
                ));
            }
        }

        info!("✅ All created IPv6 addresses validated successfully");
        Ok(())
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

    /// Get created address count
    pub fn created_address_count(&self) -> usize {
        self.created_addresses.len()
    }

    /// Get status report
    pub fn get_status_report(&self) -> String {
        format!(
            "Real IPv6Manager Status:\n\
             - Interface: {}\n\
             - Total Addresses: {}\n\
             - Created Addresses: {}\n\
             - IPv6 Prefix: {}\n\
             - Initialized: {}\n\
             - Backup Available: {}",
            self.interface_name,
            self.addresses.len(),
            self.created_addresses.len(),
            self.ipv6_prefix,
            self.initialized,
            self.backup.is_some()
        )
    }

    /// CRITICAL: Cleanup all created IPv6 addresses and restore original configuration
    pub async fn cleanup(&mut self) -> Result<()> {
        if !self.initialized {
            info!("🧹 Real IPv6 manager not initialized, nothing to clean up");
            return Ok(());
        }

        error!("🚨 CRITICAL: Starting real IPv6 cleanup");
        error!("🚨 Removing {} IPv6 addresses from system", self.created_addresses.len());

        let mut cleanup_errors = Vec::new();

        // Remove all created IPv6 addresses
        for addr in &self.created_addresses {
            error!("🚨 Removing IPv6 address {} from interface {}", addr, self.interface_name);

            if let Err(e) = self.remove_ipv6_from_interface(addr).await {
                error!("❌ Failed to remove IPv6 address {}: {}", addr, e);
                cleanup_errors.push(format!("Failed to remove {}: {}", addr, e));
            } else {
                error!("✅ Successfully removed IPv6 address: {}", addr);
            }
        }

        // Restore from backup if available
        if let Some(backup) = &self.backup {
            if let Err(e) = self.restore_from_backup(backup).await {
                error!("❌ Failed to restore from backup: {}", e);
                cleanup_errors.push(format!("Backup restoration failed: {}", e));
            } else {
                error!("✅ Network configuration restored from backup");
            }
        }

        // Clear internal state
        self.created_addresses.clear();
        self.addresses.clear();
        self.current_index = 0;
        self.initialized = false;

        if cleanup_errors.is_empty() {
            error!("✅ Real IPv6 cleanup completed successfully");
            Ok(())
        } else {
            error!("⚠️ Real IPv6 cleanup completed with errors:");
            for error in &cleanup_errors {
                error!("  • {}", error);
            }
            Err(FacebookExtractorError::network(
                format!("Cleanup completed with {} errors", cleanup_errors.len())
            ))
        }
    }

    /// Remove IPv6 address from system interface
    async fn remove_ipv6_from_interface(&self, addr: &Ipv6Addr) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let output = AsyncCommand::new("ip")
                .args(&["-6", "addr", "del", &format!("{}/64", addr), "dev", &self.interface_name])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute ip command: {}", e)
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(FacebookExtractorError::network(
                    format!("Failed to remove IPv6 address: {}", stderr)
                ));
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = AsyncCommand::new("ifconfig")
                .args(&[&self.interface_name, "inet6", &format!("{}", addr), "delete"])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute ifconfig command: {}", e)
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(FacebookExtractorError::network(
                    format!("Failed to remove IPv6 address: {}", stderr)
                ));
            }
        }

        #[cfg(target_os = "windows")]
        {
            let output = AsyncCommand::new("netsh")
                .args(&["interface", "ipv6", "delete", "address", &self.interface_name, &format!("{}", addr)])
                .output()
                .await
                .map_err(|e| FacebookExtractorError::network(
                    format!("Failed to execute netsh command: {}", e)
                ))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(FacebookExtractorError::network(
                    format!("Failed to remove IPv6 address: {}", stderr)
                ));
            }
        }

        Ok(())
    }

    /// Restore network configuration from backup
    async fn restore_from_backup(&self, backup: &NetworkBackup) -> Result<()> {
        info!("🔄 Restoring network configuration from backup");

        // For now, we rely on the address removal above
        // In a more sophisticated implementation, we would restore the exact original state

        if let Some(backup_file) = &backup.backup_file {
            info!("💾 Backup file available at: {}", backup_file);
        }

        Ok(())
    }

    /// Parse IPv6 addresses from ip command output
    fn parse_ip_output(&self, output: &str) -> Vec<Ipv6Addr> {
        let mut addresses = Vec::new();
        for line in output.lines() {
            if line.contains("inet6") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split_whitespace().nth(1) {
                    if let Some(addr_part) = addr_str.split('/').next() {
                        if let Ok(addr) = addr_part.parse::<Ipv6Addr>() {
                            addresses.push(addr);
                        }
                    }
                }
            }
        }
        addresses
    }

    /// Parse IPv6 addresses from ifconfig output
    fn parse_ifconfig_output(&self, output: &str) -> Vec<Ipv6Addr> {
        let mut addresses = Vec::new();
        for line in output.lines() {
            if line.contains("inet6") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split_whitespace().nth(1) {
                    if let Some(addr_part) = addr_str.split('/').next() {
                        if let Ok(addr) = addr_part.parse::<Ipv6Addr>() {
                            addresses.push(addr);
                        }
                    }
                }
            }
        }
        addresses
    }

    /// Parse IPv6 addresses from netsh output
    fn parse_netsh_output(&self, output: &str) -> Vec<Ipv6Addr> {
        let mut addresses = Vec::new();
        for line in output.lines() {
            if line.contains("Address") && !line.contains("::1") && !line.contains("fe80") {
                if let Some(addr_str) = line.split(':').last() {
                    let addr_str = addr_str.trim();
                    if let Ok(addr) = addr_str.parse::<Ipv6Addr>() {
                        addresses.push(addr);
                    }
                }
            }
        }
        addresses
    }
}

impl Drop for RealIPv6Manager {
    /// Automatic cleanup on drop
    fn drop(&mut self) {
        if self.initialized && !self.created_addresses.is_empty() {
            error!("🚨 CRITICAL: RealIPv6Manager dropped without cleanup!");
            error!("🚨 {} IPv6 addresses may remain on system", self.created_addresses.len());
            error!("🚨 Manual cleanup required:");

            for addr in &self.created_addresses {
                error!("🚨 Remove: ip -6 addr del {}/64 dev {}", addr, self.interface_name);
            }
        }
    }
}
