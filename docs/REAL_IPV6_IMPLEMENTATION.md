# Real IPv6 Address Rotation Implementation

## Overview

This document describes the real IPv6 address rotation functionality that makes actual system-level network configuration changes. This is a **CRITICAL** feature that requires explicit user consent and understanding of risks.

## ⚠️ CRITICAL WARNINGS

### System Impact
- **MODIFIES SYSTEM NETWORK CONFIGURATION**: Creates actual IPv6 addresses on network interfaces
- **REQUIRES ADMINISTRATOR PRIVILEGES**: Must run with root/admin privileges
- **POTENTIAL NETWORK DISRUPTION**: May affect existing network connectivity
- **LEGAL IMPLICATIONS**: May violate terms of service of some websites

### Safety Requirements
- Explicit user consent with risk acknowledgment required
- Administrator privileges verification
- System compatibility checks
- Comprehensive backup and rollback functionality
- Automatic cleanup on application exit

## Architecture

### Core Components

1. **IPv6ConsentManager**: Handles user consent and system requirements
2. **RealIPv6Manager**: Manages actual IPv6 addresses on system interfaces
3. **IPv6Manager**: Unified interface supporting both simulation and real modes
4. **AntiBlockingManager**: High-level manager with safety controls

### Implementation Modes

#### Simulation Mode (Default - Safe)
```rust
let config = AntiBlockingConfig::default(); // Uses IPv6Mode::Simulation
```
- IPv6 addresses exist only in application memory
- No system network configuration changes
- Safe for testing and demonstration
- No administrator privileges required

#### Real Mode (Critical - System Changes)
```rust
let mut config = AntiBlockingConfig::default();
config.ipv6_mode = IPv6Mode::Real;
config.ipv6_consent = consent_status; // Must be obtained through consent process
```
- Creates actual IPv6 addresses on network interfaces
- Requires explicit user consent and administrator privileges
- Makes real system network configuration changes
- Automatic cleanup and rollback functionality

## User Consent Process

### System Requirements Check
```rust
let mut consent_manager = IPv6ConsentManager::new();
let requirements = consent_manager.check_system_requirements().await?;
```

Checks:
- IPv6 support availability
- Administrator privileges
- Network interface detection
- Operating system compatibility
- Calculates compatibility score (0-100)

### Interactive Consent Dialog
```rust
let consent = consent_manager.request_user_consent(&requirements).await?;
```

User must:
1. Acknowledge understanding of risks
2. Confirm administrator privileges
3. Type exact phrase: "I ACCEPT THE RISKS"
4. Accept legal and terms-of-service implications

### Consent Requirements
- `consent_given`: User explicitly agreed
- `risks_acknowledged`: User understands system modification risks
- `admin_privileges_confirmed`: User has and accepts admin privilege usage
- `session_id`: Unique identifier for consent session

## Technical Implementation

### IPv6 Address Creation

#### Linux
```bash
ip -6 addr add 2001:db8::1/64 dev eth0
```

#### macOS
```bash
ifconfig en0 inet6 2001:db8::1 prefixlen 64 add
```

#### Windows
```cmd
netsh interface ipv6 add address "Ethernet" 2001:db8::1
```

### Address Generation
- Uses configurable IPv6 prefix (default: 2001:db8::/64)
- Generates sequential addresses by incrementing last segment
- Avoids reserved and documentation ranges
- Validates generated addresses before system application

### Network Interface Management
- Auto-detects suitable network interfaces
- Validates interface existence and status
- Checks IPv6 support on selected interface
- Monitors interface state during operation

## Safety Mechanisms

### Backup and Rollback
```rust
// Automatic backup creation
let backup = NetworkBackup {
    original_addresses: detected_addresses,
    backup_timestamp: SystemTime::now(),
    backup_file: Some("/tmp/ipv6_backup_123456.json"),
};
```

### Automatic Cleanup
- Cleanup on normal application exit
- Cleanup on error conditions
- Drop trait implementation for emergency cleanup
- Manual cleanup commands provided if automatic cleanup fails

### Error Handling
- Comprehensive error checking at each step
- Graceful fallback to simulation mode on failures
- Detailed error logging and user feedback
- System state validation before and after changes

## Usage Examples

### Basic Real IPv6 Setup
```rust
use facebook_video_downloader_core::network::anti_blocking::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Check system requirements
    let mut consent_manager = IPv6ConsentManager::new();
    let requirements = consent_manager.check_system_requirements().await?;
    
    if requirements.compatibility_score < 70 {
        println!("Low compatibility - using simulation mode");
        return Ok(());
    }
    
    // Step 2: Get user consent
    let consent = consent_manager.request_user_consent(&requirements).await?;
    
    if !consent.consent_given {
        println!("Consent not given - using simulation mode");
        return Ok(());
    }
    
    // Step 3: Configure real IPv6 mode
    let mut config = AntiBlockingConfig::default();
    config.ipv6_mode = IPv6Mode::Real;
    config.ipv6_consent = consent;
    config.max_ipv6_addresses = 3;
    
    // Step 4: Initialize and use
    let manager = AntiBlockingManager::new(config);
    manager.initialize().await?;
    
    // Make requests with real IPv6 rotation
    let response = manager.make_request("https://httpbin.org/ip").await?;
    println!("Response: {}", response.text().await?);
    
    // Step 5: Critical cleanup
    manager.cleanup().await?;
    
    Ok(())
}
```

### Testing Real IPv6 Implementation
```bash
# Run the test program
cd crates/cli
cargo run --bin test-real-ipv6

# The program will:
# 1. Check system requirements
# 2. Request user consent (if compatible)
# 3. Test real IPv6 implementation (if consented)
# 4. Fall back to simulation mode (if not compatible/consented)
# 5. Perform comprehensive cleanup
```

## Platform Support

### Linux
- **Commands**: `ip` command for IPv6 management
- **Privileges**: Requires root privileges
- **Interfaces**: Auto-detects via `ip link show`
- **Cleanup**: `ip -6 addr del` for address removal

### macOS
- **Commands**: `ifconfig` for IPv6 management
- **Privileges**: Requires sudo privileges
- **Interfaces**: Auto-detects via `ifconfig -l`
- **Cleanup**: `ifconfig <interface> inet6 <address> delete`

### Windows
- **Commands**: `netsh` for IPv6 management
- **Privileges**: Requires administrator privileges
- **Interfaces**: Auto-detects via `netsh interface show interface`
- **Cleanup**: `netsh interface ipv6 delete address`

## Security Considerations

### Network Security
- IPv6 addresses are created in documentation ranges by default
- No modification of routing tables or firewall rules
- Addresses are temporary and automatically cleaned up
- No persistent network configuration changes

### System Security
- Requires explicit administrator privilege confirmation
- All network changes are logged and tracked
- Backup files contain original configuration
- Emergency cleanup procedures available

### Legal Considerations
- User must acknowledge potential ToS violations
- Recommended for legitimate testing only
- Rate limiting and website policies must be respected
- Legal implications vary by jurisdiction

## Troubleshooting

### Common Issues

#### "Administrator privileges required"
```bash
# Linux/macOS
sudo cargo run --bin test-real-ipv6

# Windows (run as Administrator)
cargo run --bin test-real-ipv6
```

#### "IPv6 not supported"
- Check if IPv6 is enabled on the system
- Verify network interface supports IPv6
- Check firewall settings

#### "Interface not found"
- Specify interface manually: `config.network_interface = Some("eth0".to_string())`
- Check available interfaces: `ip link show` (Linux) or `ifconfig -l` (macOS)

#### "Cleanup failed"
Manual cleanup commands are provided in error messages:
```bash
# Example manual cleanup
ip -6 addr del 2001:db8::1/64 dev eth0
```

## Best Practices

1. **Always test in simulation mode first**
2. **Use minimal number of IPv6 addresses**
3. **Ensure proper cleanup in all code paths**
4. **Monitor system network state during operation**
5. **Have manual cleanup procedures ready**
6. **Respect website rate limits and policies**
7. **Use only for legitimate testing purposes**

## Future Enhancements

- IPv6 address pool management
- Dynamic interface selection
- Enhanced error recovery
- Integration with system network managers
- Support for additional operating systems
- Advanced backup and restore functionality
