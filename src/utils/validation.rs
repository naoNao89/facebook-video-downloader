// Simple Facebook URL validation without regex dependency for WASM compatibility
pub fn is_valid_facebook_url(url: &str) -> bool {
    // Basic validation - check if it's a Facebook URL with video content
    let url_lower = url.to_lowercase();

    // Must be a Facebook URL
    if !url_lower.contains("facebook.com") && !url_lower.contains("fb.watch") {
        return false;
    }

    // Must be HTTP/HTTPS
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        return false;
    }

    // Check for video-related patterns
    url_lower.contains("/watch") ||
    url_lower.contains("/videos/") ||
    url_lower.contains("/reel/") ||
    url_lower.contains("/share/") ||
    url_lower.contains("fb.watch") ||
    url_lower.contains("video.php")
}

pub fn extract_video_id(url: &str) -> Option<String> {
    // Simple video ID extraction without regex for WASM compatibility

    // Try v= parameter
    if let Some(start) = url.find("v=") {
        let id_start = start + 2;
        if let Some(end) = url[id_start..].find(&['&', '#', ' '][..]) {
            let id = &url[id_start..id_start + end];
            if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric()) {
                return Some(id.to_string());
            }
        } else {
            let id = &url[id_start..];
            if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric()) {
                return Some(id.to_string());
            }
        }
    }

    // Try /videos/ format
    if let Some(start) = url.find("/videos/") {
        let id_start = start + 8;
        if let Some(end) = url[id_start..].find(&['/', '?', '#', ' '][..]) {
            let id = &url[id_start..id_start + end];
            if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric()) {
                return Some(id.to_string());
            }
        } else {
            let id = &url[id_start..];
            if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric()) {
                return Some(id.to_string());
            }
        }
    }

    // Try fb.watch format
    if let Some(start) = url.find("fb.watch/") {
        let id_start = start + 9;
        if let Some(end) = url[id_start..].find(&['/', '?', '#', ' '][..]) {
            let id = &url[id_start..id_start + end];
            if !id.is_empty() {
                return Some(id.to_string());
            }
        } else {
            let id = &url[id_start..];
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }

    None
}

pub fn validate_download_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Download path cannot be empty".to_string());
    }

    // Basic path validation
    if path.contains("..") {
        return Err("Invalid path: contains '..'".to_string());
    }

    Ok(())
}

pub fn validate_concurrent_downloads(count: i32) -> Result<u32, String> {
    if count < 1 {
        return Err("Concurrent downloads must be at least 1".to_string());
    }

    if count > 10 {
        return Err("Concurrent downloads cannot exceed 10".to_string());
    }

    Ok(count as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facebook_url_validation() {
        assert!(is_valid_facebook_url("https://www.facebook.com/watch?v=123456789"));
        assert!(is_valid_facebook_url("https://www.facebook.com/watch/?v=900186485344685")); // Test the problematic URL format
        assert!(is_valid_facebook_url("https://www.facebook.com/watch?v=718400947385071")); // Test the specific URL format
        assert!(is_valid_facebook_url("https://facebook.com/username/videos/123456789"));
        assert!(is_valid_facebook_url("https://fb.watch/abc123"));
        assert!(is_valid_facebook_url("https://www.facebook.com/reel/123456789"));
        assert!(!is_valid_facebook_url("https://youtube.com/watch?v=123"));
        assert!(!is_valid_facebook_url("not a url"));
    }

    #[test]
    fn test_video_id_extraction() {
        assert_eq!(
            extract_video_id("https://www.facebook.com/watch?v=123456789"),
            Some("123456789".to_string())
        );
        assert_eq!(
            extract_video_id("https://facebook.com/username/videos/987654321"),
            Some("987654321".to_string())
        );
        assert_eq!(
            extract_video_id("https://fb.watch/abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(extract_video_id("invalid url"), None);
    }
}
