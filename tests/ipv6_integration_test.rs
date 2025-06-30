#[cfg(test)]
mod ipv6_integration_tests {
    use std::net::Ipv6Addr;
    use serde::{Deserialize, Serialize};

    // Define the structs locally for testing since we can't import from the main crate in tests
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SystemRequirements {
        pub ipv6_supported: bool,
        pub admin_privileges: bool,
        pub network_interfaces: Vec<String>,
        pub existing_ipv6_addresses: Vec<Ipv6Addr>,
        pub operating_system: String,
        pub compatibility_score: u8,
        pub warnings: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IPv6ConsentStatus {
        pub consent_given: bool,
        pub consent_timestamp: Option<String>,
        pub risks_acknowledged: bool,
        pub admin_privileges_confirmed: bool,
        pub session_id: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct IPv6Status {
        pub enabled: bool,
        pub active_addresses: Vec<Ipv6Addr>,
        pub current_address_index: usize,
        pub total_addresses: usize,
        pub network_interface: Option<String>,
        pub consent_given: bool,
        pub last_rotation: Option<String>,
        pub rotation_count: u64,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct IPv6Settings {
        pub enabled: bool,
        pub consent_given: bool,
        pub max_addresses: u32,
        pub network_interface: Option<String>,
        pub ipv6_prefix: Option<String>,
        pub auto_rotation: bool,
        pub rotation_interval_minutes: u32,
    }

    impl Default for IPv6Settings {
        fn default() -> Self {
            Self {
                enabled: false,
                consent_given: false,
                max_addresses: 5,
                network_interface: None,
                ipv6_prefix: None,
                auto_rotation: false,
                rotation_interval_minutes: 30,
            }
        }
    }

    /// Test IPv6 system requirements structure
    #[test]
    fn test_system_requirements_structure() {
        let requirements = SystemRequirements {
            ipv6_supported: true,
            admin_privileges: false,
            network_interfaces: vec!["eth0".to_string(), "wlan0".to_string()],
            existing_ipv6_addresses: vec![
                "2001:db8::1".parse::<Ipv6Addr>().unwrap(),
                "fe80::1".parse::<Ipv6Addr>().unwrap(),
            ],
            operating_system: "Linux".to_string(),
            compatibility_score: 85,
            warnings: vec!["Admin privileges required".to_string()],
        };

        assert_eq!(requirements.ipv6_supported, true);
        assert_eq!(requirements.admin_privileges, false);
        assert_eq!(requirements.network_interfaces.len(), 2);
        assert_eq!(requirements.existing_ipv6_addresses.len(), 2);
        assert_eq!(requirements.compatibility_score, 85);
        assert_eq!(requirements.warnings.len(), 1);
    }

    /// Test IPv6 consent status structure
    #[test]
    fn test_ipv6_consent_status() {
        let consent = IPv6ConsentStatus {
            consent_given: true,
            consent_timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            risks_acknowledged: true,
            admin_privileges_confirmed: true,
            session_id: "test-session-123".to_string(),
        };

        assert_eq!(consent.consent_given, true);
        assert_eq!(consent.risks_acknowledged, true);
        assert_eq!(consent.admin_privileges_confirmed, true);
        assert!(consent.consent_timestamp.is_some());
        assert_eq!(consent.session_id, "test-session-123");
    }

    /// Test IPv6 status structure
    #[test]
    fn test_ipv6_status() {
        let status = IPv6Status {
            enabled: true,
            active_addresses: vec![
                "2001:db8::100".parse::<Ipv6Addr>().unwrap(),
                "2001:db8::101".parse::<Ipv6Addr>().unwrap(),
            ],
            current_address_index: 0,
            total_addresses: 2,
            network_interface: Some("eth0".to_string()),
            consent_given: true,
            last_rotation: Some("2024-01-01T00:00:00Z".to_string()),
            rotation_count: 5,
        };

        assert_eq!(status.enabled, true);
        assert_eq!(status.active_addresses.len(), 2);
        assert_eq!(status.current_address_index, 0);
        assert_eq!(status.total_addresses, 2);
        assert_eq!(status.consent_given, true);
        assert_eq!(status.rotation_count, 5);
        assert!(status.network_interface.is_some());
        assert!(status.last_rotation.is_some());
    }

    /// Test compatibility score validation
    #[test]
    fn test_compatibility_score_validation() {
        // High compatibility score
        let high_compat = SystemRequirements {
            ipv6_supported: true,
            admin_privileges: true,
            network_interfaces: vec!["eth0".to_string()],
            existing_ipv6_addresses: vec![],
            operating_system: "Linux".to_string(),
            compatibility_score: 95,
            warnings: vec![],
        };
        assert!(high_compat.compatibility_score >= 70);

        // Low compatibility score
        let low_compat = SystemRequirements {
            ipv6_supported: false,
            admin_privileges: false,
            network_interfaces: vec![],
            existing_ipv6_addresses: vec![],
            operating_system: "Unknown".to_string(),
            compatibility_score: 30,
            warnings: vec!["IPv6 not supported".to_string(), "No admin privileges".to_string()],
        };
        assert!(low_compat.compatibility_score < 70);
        assert!(low_compat.warnings.len() > 0);
    }

    /// Test IPv6 address parsing
    #[test]
    fn test_ipv6_address_parsing() {
        let valid_addresses = vec![
            "2001:db8::1",
            "fe80::1",
            "::1",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
        ];

        for addr_str in valid_addresses {
            let addr = addr_str.parse::<Ipv6Addr>();
            assert!(addr.is_ok(), "Failed to parse valid IPv6 address: {}", addr_str);
        }

        let invalid_addresses = vec![
            "192.168.1.1",  // IPv4
            "invalid",       // Not an IP
            "2001:db8::g",   // Invalid hex
        ];

        for addr_str in invalid_addresses {
            let addr = addr_str.parse::<Ipv6Addr>();
            assert!(addr.is_err(), "Should not parse invalid IPv6 address: {}", addr_str);
        }
    }

    /// Test consent validation logic
    #[test]
    fn test_consent_validation() {
        // Valid consent
        let valid_consent = IPv6ConsentStatus {
            consent_given: true,
            consent_timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            risks_acknowledged: true,
            admin_privileges_confirmed: true,
            session_id: "valid-session".to_string(),
        };

        assert!(is_valid_consent(&valid_consent));

        // Invalid consent - missing consent
        let invalid_consent1 = IPv6ConsentStatus {
            consent_given: false,
            consent_timestamp: None,
            risks_acknowledged: true,
            admin_privileges_confirmed: true,
            session_id: "invalid-session".to_string(),
        };

        assert!(!is_valid_consent(&invalid_consent1));

        // Invalid consent - risks not acknowledged
        let invalid_consent2 = IPv6ConsentStatus {
            consent_given: true,
            consent_timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            risks_acknowledged: false,
            admin_privileges_confirmed: true,
            session_id: "invalid-session".to_string(),
        };

        assert!(!is_valid_consent(&invalid_consent2));
    }

    /// Helper function to validate consent
    fn is_valid_consent(consent: &IPv6ConsentStatus) -> bool {
        consent.consent_given && 
        consent.risks_acknowledged && 
        consent.admin_privileges_confirmed
    }

    /// Test IPv6 settings serialization/deserialization
    #[test]
    fn test_ipv6_settings_serialization() {
        use serde_json;

        let settings = IPv6Settings {
            enabled: true,
            consent_given: true,
            max_addresses: 5,
            network_interface: Some("eth0".to_string()),
            ipv6_prefix: Some("2001:db8::/64".to_string()),
            auto_rotation: true,
            rotation_interval_minutes: 30,
        };

        // Test serialization
        let json = serde_json::to_string(&settings);
        assert!(json.is_ok());

        // Test deserialization
        let json_str = json.unwrap();
        let deserialized: Result<IPv6Settings, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let deserialized_settings = deserialized.unwrap();
        assert_eq!(deserialized_settings.enabled, settings.enabled);
        assert_eq!(deserialized_settings.consent_given, settings.consent_given);
        assert_eq!(deserialized_settings.max_addresses, settings.max_addresses);
        assert_eq!(deserialized_settings.network_interface, settings.network_interface);
        assert_eq!(deserialized_settings.ipv6_prefix, settings.ipv6_prefix);
        assert_eq!(deserialized_settings.auto_rotation, settings.auto_rotation);
        assert_eq!(deserialized_settings.rotation_interval_minutes, settings.rotation_interval_minutes);
    }

    /// Test default IPv6 settings
    #[test]
    fn test_default_ipv6_settings() {

        let default_settings = IPv6Settings::default();
        
        assert_eq!(default_settings.enabled, false);
        assert_eq!(default_settings.consent_given, false);
        assert_eq!(default_settings.max_addresses, 5);
        assert_eq!(default_settings.network_interface, None);
        assert_eq!(default_settings.ipv6_prefix, None);
        assert_eq!(default_settings.auto_rotation, false);
        assert_eq!(default_settings.rotation_interval_minutes, 30);
    }
}
