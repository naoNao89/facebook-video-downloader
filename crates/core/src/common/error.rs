//! Error types for the Facebook extractor library

use thiserror::Error;

/// Main error type for Facebook extraction operations
#[derive(Debug, Error)]
pub enum FacebookExtractorError {
    #[error("Invalid Facebook URL: {url}")]
    InvalidUrl { url: String },

    #[error("Video ID extraction failed: {message}")]
    VideoIdExtraction { message: String },

    #[error("Video not accessible: {reason}")]
    AccessDenied { reason: String },

    #[error("Private video requires authentication")]
    AuthenticationRequired,

    #[error("Content not available in your region")]
    GeoBlocked,

    #[error("Video has been removed or is no longer available")]
    ContentUnavailable,

    #[error("Rate limit exceeded. Please try again later")]
    RateLimited,

    #[error("Network error: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },

    #[error("HTML parsing failed: {message}")]
    HtmlParsing { message: String },

    #[error("Stream analysis failed: {message}")]
    StreamAnalysis { message: String },

    #[error("Download failed: {message}")]
    Download { message: String },

    #[error("FFmpeg error: {message}")]
    FFmpeg { message: String },

    #[error("File system error: {source}")]
    FileSystem {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON parsing failed: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    #[error("Base64 decoding failed: {source}")]
    Base64 {
        #[from]
        source: base64::DecodeError,
    },

    #[error("Timeout occurred: {message}")]
    Timeout { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("View-source parsing failed: {message}")]
    ViewSourceParsing { message: String },

    #[error("CORS proxy error: {message}")]
    CorsProxy { message: String },

    #[error("Unsupported content type: {content_type}")]
    UnsupportedContentType { content_type: String },

    #[error("Compression error: {message}")]
    Compression { message: String },

    #[error("Batch processing error: {message}")]
    Batch { message: String },
}

impl FacebookExtractorError {
    /// Create a new invalid URL error
    pub fn invalid_url(url: impl Into<String>) -> Self {
        Self::InvalidUrl { url: url.into() }
    }

    /// Create a new video ID extraction error
    pub fn video_id_extraction(message: impl Into<String>) -> Self {
        Self::VideoIdExtraction {
            message: message.into(),
        }
    }

    /// Create a new HTML parsing error
    pub fn html_parsing(message: impl Into<String>) -> Self {
        Self::HtmlParsing {
            message: message.into(),
        }
    }

    /// Create a new download error
    pub fn download(message: impl Into<String>) -> Self {
        Self::Download {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
        }
    }

    /// Create a new stream analysis error
    pub fn stream_analysis(message: impl Into<String>) -> Self {
        Self::StreamAnalysis {
            message: message.into(),
        }
    }

    /// Create a new compression error
    pub fn compression(message: impl Into<String>) -> Self {
        Self::Compression {
            message: message.into(),
        }
    }

    /// Create a new batch processing error
    pub fn batch(message: impl Into<String>) -> Self {
        Self::Batch {
            message: message.into(),
        }
    }

    /// Create a new network error (using download error for custom messages)
    pub fn network(message: impl Into<String>) -> Self {
        Self::Download {
            message: format!("Network error: {}", message.into()),
        }
    }

    /// Check if the error is recoverable (can be retried)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Network { .. }
                | Self::Timeout { .. }
                | Self::RateLimited
                | Self::CorsProxy { .. }
        )
    }

    /// Check if the error is related to network connectivity
    pub fn is_network_error(&self) -> bool {
        matches!(self, Self::Network { .. } | Self::Timeout { .. })
    }

    /// Check if the error is related to content access
    pub fn is_access_error(&self) -> bool {
        matches!(
            self,
            Self::AccessDenied { .. }
                | Self::AuthenticationRequired
                | Self::GeoBlocked
                | Self::ContentUnavailable
        )
    }
}

/// Result type alias for cleaner error handling
pub type Result<T> = std::result::Result<T, FacebookExtractorError>;

/// Convert string errors to FacebookExtractorError
impl From<String> for FacebookExtractorError {
    fn from(message: String) -> Self {
        Self::Download { message }
    }
}

impl From<&str> for FacebookExtractorError {
    fn from(message: &str) -> Self {
        Self::Download {
            message: message.to_string(),
        }
    }
}
