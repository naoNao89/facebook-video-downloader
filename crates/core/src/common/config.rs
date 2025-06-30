//! Configuration management for the Facebook extractor

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::{Result, FacebookExtractorError, USER_AGENTS};

/// Main extractor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorConfig {
    pub network: NetworkConfig,
    pub download: DownloadConfig,
    pub output: OutputConfig,
    pub privacy: PrivacyConfig,
    pub caching: CachingConfig,
    pub advanced: AdvancedConfig,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub timeout_secs: u64,
    pub connection_timeout_secs: u64,
    pub pool_idle_timeout_secs: u64,
    pub pool_max_idle_per_host: usize,
    pub tcp_keepalive_secs: u64,
    pub max_redirects: usize,
    pub user_agents: Vec<String>,
    pub max_retry_attempts: usize,
    pub retry_delay_secs: u64,
}

/// Download configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub max_concurrent_downloads: usize,
    pub chunk_size_mb: u64,
    pub progress_update_interval_ms: u64,
    pub enable_resume: bool,
    pub verify_downloads: bool,
    pub min_file_size_mb: u64,
    pub max_file_size_mb: u64,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub default_download_dir: String,
    pub filename_template: String,
    pub max_filename_length: usize,
    pub sanitize_filenames: bool,
    pub create_subdirectories: bool,
    pub organize_by_date: bool,
    pub organize_by_author: bool,
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub enable_view_source_extraction: bool,
    pub respect_privacy_settings: bool,
    pub require_explicit_consent: bool,
    pub log_extraction_attempts: bool,
    pub anonymize_logs: bool,
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    pub enable_caching: bool,
    pub cache_directory: String,
    pub cache_expiry_hours: u32,
    pub max_cache_size_mb: u64,
    pub auto_cleanup: bool,
}

/// Advanced configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub enable_ffmpeg: bool,
    pub ffmpeg_path: Option<String>,
    pub enable_debug_logging: bool,
    pub save_debug_files: bool,
    pub experimental_features: bool,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            download: DownloadConfig::default(),
            output: OutputConfig::default(),
            privacy: PrivacyConfig::default(),
            caching: CachingConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 600,
            connection_timeout_secs: 30,
            pool_idle_timeout_secs: 90,
            pool_max_idle_per_host: 10,
            tcp_keepalive_secs: 60,
            max_redirects: 5,
            user_agents: USER_AGENTS.iter().map(|s| s.to_string()).collect(),
            max_retry_attempts: 5,
            retry_delay_secs: 5,
        }
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 6,
            chunk_size_mb: 1,
            progress_update_interval_ms: 500,
            enable_resume: true,
            verify_downloads: true,
            min_file_size_mb: 1,
            max_file_size_mb: 2048, // 2GB
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_download_dir: "downloads".to_string(),
            filename_template: "{title} - {quality} - {author}".to_string(),
            max_filename_length: 95,
            sanitize_filenames: true,
            create_subdirectories: true,
            organize_by_date: false,
            organize_by_author: false,
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            enable_view_source_extraction: true,
            respect_privacy_settings: true,
            require_explicit_consent: false,
            log_extraction_attempts: true,
            anonymize_logs: false,
        }
    }
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_directory: "cache".to_string(),
            cache_expiry_hours: 24,
            max_cache_size_mb: 100,
            auto_cleanup: true,
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            enable_ffmpeg: true,
            ffmpeg_path: None, // Auto-detect
            enable_debug_logging: false,
            save_debug_files: false,
            experimental_features: false,
        }
    }
}

impl ExtractorConfig {
    /// Load configuration from file with fallback to defaults
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = std::fs::read_to_string(path)?;
            let config: Self = toml::from_str(&content).map_err(|e| {
                FacebookExtractorError::Config {
                    message: format!("Failed to parse config file: {}", e),
                }
            })?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            FacebookExtractorError::Config {
                message: format!("Failed to serialize config: {}", e),
            }
        })?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<()> {
        // Validate network settings
        if self.network.timeout_secs == 0 {
            return Err(FacebookExtractorError::Config {
                message: "Network timeout cannot be zero".to_string(),
            });
        }

        if self.network.user_agents.is_empty() {
            return Err(FacebookExtractorError::Config {
                message: "At least one user agent must be specified".to_string(),
            });
        }

        // Validate download settings
        if self.download.max_concurrent_downloads == 0 {
            return Err(FacebookExtractorError::Config {
                message: "Max concurrent downloads cannot be zero".to_string(),
            });
        }

        if self.download.chunk_size_mb == 0 {
            return Err(FacebookExtractorError::Config {
                message: "Chunk size cannot be zero".to_string(),
            });
        }

        // Validate output settings
        if self.output.max_filename_length < 10 {
            return Err(FacebookExtractorError::Config {
                message: "Max filename length must be at least 10 characters".to_string(),
            });
        }

        Ok(())
    }

    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        let mut config = Self::default();
        config.network.max_retry_attempts = 3;
        config.network.timeout_secs = 300;
        config.download.max_concurrent_downloads = 8;
        config.download.chunk_size_mb = 2;
        config.caching.enable_caching = true;
        config.caching.cache_expiry_hours = 6;
        config
    }

    /// Create a configuration optimized for privacy
    pub fn privacy_optimized() -> Self {
        let mut config = Self::default();
        config.privacy.respect_privacy_settings = true;
        config.privacy.require_explicit_consent = true;
        config.privacy.anonymize_logs = true;
        config.caching.enable_caching = false;
        config.advanced.save_debug_files = false;
        config
    }

    /// Create a configuration for development/debugging
    pub fn debug_optimized() -> Self {
        let mut config = Self::default();
        config.advanced.enable_debug_logging = true;
        config.advanced.save_debug_files = true;
        config.advanced.experimental_features = true;
        config.network.max_retry_attempts = 1;
        config.caching.enable_caching = false;
        config
    }
}
