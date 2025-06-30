//! Metadata extraction functionality

use crate::common::types::{VideoMetadata, ThumbnailCollection, AspectRatio, DisplayContext, PrivacyLevel, ThumbnailVariant};
use regex::Regex;

/// Metadata extractor for Facebook videos
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Create a new metadata extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract title from HTML with enhanced parsing for Facebook's title format
    pub fn extract_title_from_html(&self, html: &str, video_id: &str) -> String {
        let title_patterns = vec![
            // HTML title tag (highest priority for reels - contains full title)
            r#"<title>([^<]+)</title>"#,

            // Open Graph and meta tags (high priority for Reels)
            r#"<meta property="og:title" content="([^"]+)""#,
            r#"<meta name="twitter:title" content="([^"]+)""#,

            // Facebook Reels specific patterns - look for actual content text
            r#""text":"([^"]+)","ranges":\[\],"color_ranges":\[\],"inline_style_ranges":\[\]"#,
            r#""message":"([^"]+)","ranges":\[\]"#,
            r#""story":"([^"]+)","ranges":\[\]"#,

            // JSON patterns for video title (more specific)
            r#""video_title":"([^"]+)""#,
            r#""content_title":"([^"]+)""#,
            r#""post_title":"([^"]+)""#,
            r#""title":"([^"]+)""#,

            // Fallback patterns
            r#""name":"([^"]+)""#,
        ];

        for pattern in title_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(title_match) = capture.get(1) {
                        let raw_title = title_match.as_str().replace("\\", "");

                        // Clean and parse the title
                        let cleaned_title = self.parse_facebook_title(&raw_title);

                        // Skip if this looks like engagement data (views/reactions)
                        if self.is_engagement_data(&cleaned_title) {
                            continue;
                        }

                        // Skip JavaScript bundle names and other technical content
                        if self.is_technical_content(&cleaned_title) {
                            continue;
                        }

                        if !cleaned_title.is_empty() && cleaned_title.len() > 5 && cleaned_title != "Facebook" {
                            return cleaned_title;
                        }
                    }
                }
            }
        }

        // Enhanced fallback for share URLs
        if video_id.len() > 8 {
            format!("Facebook Video {}", &video_id[..8])
        } else {
            format!("Facebook Video {}", video_id)
        }
    }

    /// Check if a string looks like engagement data (views, reactions, etc.)
    pub fn is_engagement_data(&self, text: &str) -> bool {
        let engagement_patterns = vec![
            r"^\d+\.?\d*[KMB]?\s+views\s*[·•]",  // "1.8M views ·"
            r"^\d+\.?\d*[KMB]?\s+reactions",     // "24K reactions"
            r"^\d+\.?\d*[KMB]?\s+views\s*$",     // "1.8M views"
            r"^[\d.]+[KMB]?\s+views\s*[·•]\s*[\d.]+[KMB]?\s+reactions", // "1.8M views · 24K reactions"
        ];

        for pattern in engagement_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(text) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a string looks like technical content (JavaScript bundles, etc.)
    pub fn is_technical_content(&self, text: &str) -> bool {
        let technical_patterns = vec![
            r"^VideoPlayer.*Bundle$",           // "VideoPlayerNextgendashWorkerEntrypointBundle"
            r"^.*Worker.*Bundle$",              // Any worker bundle
            r"^.*Entrypoint.*Bundle$",          // Any entrypoint bundle
            r"^.*dash.*Worker.*$",              // Dash worker patterns
            r"^[A-Z][a-zA-Z]*Bundle$",          // General bundle patterns
            r"^[a-zA-Z]+\d+[a-zA-Z]*$",         // Mixed alphanumeric technical IDs
            r"^[A-Z]{2,}[a-z]*[A-Z]{2,}",       // CamelCase technical names
        ];

        for pattern in technical_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(text) {
                    return true;
                }
            }
        }
        false
    }

    /// Parse Facebook's title format to extract clean title
    /// Facebook often uses format: "Title | Author | Facebook" or "Title | Title | By Author"
    fn parse_facebook_title(&self, raw_title: &str) -> String {
        // First decode HTML entities and Unicode escapes
        let title = self.decode_html_entities(raw_title);
        let title = self.decode_unicode_escapes(&title);

        // Remove common Facebook suffixes and clean up
        let title = title
            .replace(" - Facebook", "")
            .replace(" | Facebook", "")
            .trim()
            .to_string();

        // Handle mobile Facebook format: "Title | Channel/Hashtags | By Author" (most common for reels)
        if title.contains(" | ") {
            let parts: Vec<&str> = title.split(" | ").collect();

            // Check for "By Author" format: "Title | Channel | By Author"
            if parts.len() >= 3 && parts[2].starts_with("By ") {
                // For this format, combine first two parts as the full title
                let full_title = format!("{} | {}", parts[0].trim(), parts[1].trim());
                if !self.is_engagement_data(parts[0]) {
                    // For reels with "By Author" format, preserve hashtags in the title
                    return self.clean_title_text_preserve_hashtags(&full_title);
                }
            }

            // If we have 2+ parts, the first part is usually the title
            if parts.len() >= 2 {
                let potential_title = parts[0].trim();
                let potential_author = parts[1].trim();

                // Check if the second part looks like an author name (contains spaces or is a page name)
                // and the first part doesn't look like engagement data
                if !self.is_engagement_data(potential_title) &&
                   (potential_author.contains(' ') || potential_author.len() > 5) {
                    return self.clean_title_text(potential_title);
                }

                // If first two parts are identical, it's likely a duplicated title
                if parts[0] == parts[1] {
                    return self.clean_title_text(parts[0]);
                }

                // Otherwise, take the first part as the title
                return self.clean_title_text(parts[0]);
            }
        }

        // Handle Facebook's "By Author" format: "Title | Title | By Author"
        if title.contains(" | By ") {
            // Split by " | By " and take the first part
            if let Some(clean_title) = title.split(" | By ").next() {
                let clean_title = clean_title.trim();

                // If the title is duplicated (e.g., "Title | Title | By Author"),
                // extract just the first occurrence
                if clean_title.contains(" | ") {
                    let parts: Vec<&str> = clean_title.split(" | ").collect();
                    if parts.len() >= 2 && parts[0] == parts[1] {
                        // Duplicated title, return just the first part
                        return self.clean_title_text(parts[0]);
                    }
                }

                return self.clean_title_text(clean_title);
            }
        }

        self.clean_title_text(&title)
    }

    /// Decode HTML entities in text
    pub fn decode_html_entities(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Handle decimal HTML entities (&#183; → ·)
        if let Ok(regex) = Regex::new(r"&#(\d+);") {
            let matches: Vec<_> = regex.captures_iter(&result).map(|cap| {
                (
                    cap.get(0).unwrap().as_str().to_string(),
                    cap.get(1).unwrap().as_str().to_string(),
                )
            }).collect();

            for (full_match, decimal_str) in matches {
                if let Ok(code_point) = decimal_str.parse::<u32>() {
                    if let Some(unicode_char) = char::from_u32(code_point) {
                        result = result.replace(&full_match, &unicode_char.to_string());
                    }
                }
            }
        }

        // Handle hexadecimal HTML entities (&#xb7; → ·, &#xf4; → ô, &#x1ecf; → ỏ)
        if let Ok(regex) = Regex::new(r"&#x([0-9a-fA-F]+);") {
            let matches: Vec<_> = regex.captures_iter(&result).map(|cap| {
                (
                    cap.get(0).unwrap().as_str().to_string(),
                    cap.get(1).unwrap().as_str().to_string(),
                )
            }).collect();

            for (full_match, hex_str) in matches {
                if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                    if let Some(unicode_char) = char::from_u32(code_point) {
                        result = result.replace(&full_match, &unicode_char.to_string());
                    }
                }
            }
        }

        // Handle common named HTML entities
        result = result
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#039;", "'")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&nbsp;", " ");

        result
    }

    /// Decode Unicode escape sequences in text
    pub fn decode_unicode_escapes(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Handle Unicode escape sequences like \u00e1 -> á (with backslash)
        if let Ok(regex) = Regex::new(r"\\u([0-9a-fA-F]{4})") {
            while let Some(capture) = regex.captures(&result) {
                if let Some(hex_match) = capture.get(1) {
                    if let Ok(code_point) = u32::from_str_radix(hex_match.as_str(), 16) {
                        if let Some(unicode_char) = char::from_u32(code_point) {
                            let full_match = capture.get(0).unwrap().as_str();
                            result = result.replace(full_match, &unicode_char.to_string());
                        }
                    }
                }
            }
        }

        // Handle Unicode escape sequences like u00e1 -> á (without backslash, common in Facebook JSON)
        // Pattern 1: letter + u + 4 hex digits + letter (like Cu00e1ch)
        if let Ok(regex) = Regex::new(r"([a-zA-Z])u([0-9a-fA-F]{4})([a-zA-Z])") {
            let matches: Vec<_> = regex.captures_iter(&result).map(|cap| {
                (
                    cap.get(0).unwrap().as_str().to_string(),
                    cap.get(1).unwrap().as_str().to_string(),
                    cap.get(2).unwrap().as_str().to_string(),
                    cap.get(3).unwrap().as_str().to_string(),
                )
            }).collect();

            for (full_match, before, hex_str, after) in matches {
                if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                    if let Some(unicode_char) = char::from_u32(code_point) {
                        let replacement = format!("{}{}{}", before, unicode_char, after);
                        result = result.replace(&full_match, &replacement);
                    }
                }
            }
        }

        // Pattern 2: letter + u + 4 hex digits + space/end (like chu00f3 )
        if let Ok(regex) = Regex::new(r"([a-zA-Z])u([0-9a-fA-F]{4})(\s|$)") {
            let matches: Vec<_> = regex.captures_iter(&result).map(|cap| {
                (
                    cap.get(0).unwrap().as_str().to_string(),
                    cap.get(1).unwrap().as_str().to_string(),
                    cap.get(2).unwrap().as_str().to_string(),
                    cap.get(3).unwrap().as_str().to_string(),
                )
            }).collect();

            for (full_match, before, hex_str, after) in matches {
                if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                    if let Some(unicode_char) = char::from_u32(code_point) {
                        let replacement = format!("{}{}{}", before, unicode_char, after);
                        result = result.replace(&full_match, &replacement);
                    }
                }
            }
        }

        // Pattern 3: space/start + u + 4 hex digits + letter (like u0111i)
        if let Ok(regex) = Regex::new(r"(\s|^)u([0-9a-fA-F]{4})([a-zA-Z])") {
            let matches: Vec<_> = regex.captures_iter(&result).map(|cap| {
                (
                    cap.get(0).unwrap().as_str().to_string(),
                    cap.get(1).unwrap().as_str().to_string(),
                    cap.get(2).unwrap().as_str().to_string(),
                    cap.get(3).unwrap().as_str().to_string(),
                )
            }).collect();

            for (full_match, before, hex_str, after) in matches {
                if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                    if let Some(unicode_char) = char::from_u32(code_point) {
                        let replacement = format!("{}{}{}", before, unicode_char, after);
                        result = result.replace(&full_match, &replacement);
                    }
                }
            }
        }

        // Pattern 4: letter + u + 4 hex digits + letter (for remaining cases like đu1ed5i)
        // This is a second pass to catch any remaining patterns
        if let Ok(regex) = Regex::new(r"([a-zA-Z])u([0-9a-fA-F]{4})([a-zA-Z])") {
            let matches: Vec<_> = regex.captures_iter(&result).map(|cap| {
                (
                    cap.get(0).unwrap().as_str().to_string(),
                    cap.get(1).unwrap().as_str().to_string(),
                    cap.get(2).unwrap().as_str().to_string(),
                    cap.get(3).unwrap().as_str().to_string(),
                )
            }).collect();

            for (full_match, before, hex_str, after) in matches {
                if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                    if let Some(unicode_char) = char::from_u32(code_point) {
                        let replacement = format!("{}{}{}", before, unicode_char, after);
                        result = result.replace(&full_match, &replacement);
                    }
                }
            }
        }

        // Handle other common escape sequences
        result = result
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\\"", "\"")
            .replace("\\'", "'")
            .replace("\\\\", "\\");

        result
    }

    /// Clean up title text by removing hashtags and extra content
    fn clean_title_text(&self, title: &str) -> String {
        let mut cleaned = title.trim().to_string();

        // If the entire string is just engagement data, return empty to skip it
        if self.is_engagement_data(&cleaned) {
            return String::new();
        }

        // Remove view and reaction counts from the beginning with enhanced patterns
        // Handles formats like: "2.6M views · 9.9K reactions | Title"
        let view_reaction_patterns = vec![
            // Pattern with decoded entities (after HTML entity decoding)
            r"^[\d.]+[KMB]?\s+views\s*[·•]\s*[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
            // Pattern with HTML entities (before decoding)
            r"^[\d.]+[KMB]?\s+views\s*&#x[0-9a-fA-F]+;\s*[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
            // More flexible pattern
            r"^[\d.]+[KMB]?\s+views\s*[^\w]*\s*[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
            // Just views at the beginning
            r"^[\d.]+[KMB]?\s+views\s*[|·•]\s*",
            // Just reactions at the beginning
            r"^[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
        ];

        for pattern in view_reaction_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&cleaned) {
                    cleaned = regex.replace(&cleaned, "").to_string();
                    break;
                }
            }
        }

        // Remove hashtags and everything after them (common in Facebook titles)
        if let Some(hashtag_pos) = cleaned.find(" #") {
            cleaned = cleaned[..hashtag_pos].trim().to_string();
        }

        // Remove hashtags at the end of lines
        if let Some(hashtag_pos) = cleaned.find("\n\n#") {
            cleaned = cleaned[..hashtag_pos].trim().to_string();
        }

        // Remove emoji sequences at the end
        if let Ok(regex) = Regex::new(r"\s*[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{1F1E0}-\u{1F1FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]+\s*$") {
            cleaned = regex.replace_all(&cleaned, "").to_string();
        }

        // Remove extra whitespace
        cleaned = cleaned.trim().to_string();

        cleaned
    }

    /// Clean up title text while preserving hashtags (for reels with channel names)
    fn clean_title_text_preserve_hashtags(&self, title: &str) -> String {
        let mut cleaned = title.trim().to_string();

        // If the entire string is just engagement data, return empty to skip it
        if self.is_engagement_data(&cleaned) {
            return String::new();
        }

        // Remove view and reaction counts from the beginning with enhanced patterns
        // Handles formats like: "2.6M views · 9.9K reactions | Title"
        let view_reaction_patterns = vec![
            // Pattern with decoded entities (after HTML entity decoding)
            r"^[\d.]+[KMB]?\s+views\s*[·•]\s*[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
            // Pattern with HTML entities (before decoding)
            r"^[\d.]+[KMB]?\s+views\s*&#x[0-9a-fA-F]+;\s*[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
            // More flexible pattern
            r"^[\d.]+[KMB]?\s+views\s*[^\w]*\s*[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
            // Just views at the beginning
            r"^[\d.]+[KMB]?\s+views\s*[|·•]\s*",
            // Just reactions at the beginning
            r"^[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
        ];

        for pattern in view_reaction_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&cleaned) {
                    cleaned = regex.replace(&cleaned, "").to_string();
                    break;
                }
            }
        }

        // DON'T remove hashtags for this version - preserve them as part of the title

        // Remove emoji sequences at the end
        if let Ok(regex) = Regex::new(r"\s*[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{1F1E0}-\u{1F1FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]+\s*$") {
            cleaned = regex.replace_all(&cleaned, "").to_string();
        }

        // Remove extra whitespace and normalize newlines to spaces
        cleaned = cleaned.replace('\n', " ").replace('\r', " ");
        cleaned = cleaned.trim().to_string();

        // Normalize multiple spaces to single spaces
        if let Ok(regex) = Regex::new(r"\s+") {
            cleaned = regex.replace_all(&cleaned, " ").to_string();
        }

        cleaned
    }

    /// Extract comprehensive video metadata from HTML
    pub fn extract_video_metadata(&self, html: &str) -> VideoMetadata {
        let author = self.extract_author_from_html(html);
        let description = self.extract_description_from_html(html);
        let publish_date = self.extract_publish_date_from_html(html);
        let likes = self.extract_likes_from_html(html);
        let comments = self.extract_comments_from_html(html);
        let views = self.extract_views_from_html(html);
        let shares = self.extract_shares_from_html(html);
        let hashtags = self.extract_hashtags_from_html(html);
        let duration_seconds = self.extract_duration_seconds_from_html(html);
        let privacy_level = self.detect_privacy_level_from_html(html);

        VideoMetadata {
            author,
            description,
            publish_date,
            likes: self.parse_engagement_count(&likes),
            comments: self.parse_engagement_count(&comments),
            views: self.parse_engagement_count(&views),
            shares: self.parse_engagement_count(&shares),
            hashtags,
            duration_seconds,
            language: None,
            category: None,
            author_url: None,
            author_verified: false,
            privacy_level,
            location: None,
            content_warnings: Vec::new(),
        }
    }

    /// Extract author/page name from HTML with enhanced parsing
    pub fn extract_author_from_html(&self, html: &str) -> String {
        // First try to extract author from title tag (most reliable for Facebook)
        if let Some(author) = self.extract_author_from_title(html) {
            if !author.is_empty() && author.len() > 2 {
                return self.clean_author_name(&author);
            }
        }

        // Fallback to JSON patterns
        let patterns = vec![
            // Meta tag patterns for author (from og:title content) - HIGHEST PRIORITY
            r#"<meta property="og:title" content="[^"]*\| By ([^"]+) \|"#,
            r#"<meta property="og:title" content="[^"]*\| By ([^"]+)""#,
            r#"<meta name="twitter:title" content="[^"]*\| By ([^"]+) \|"#,
            r#"<meta name="twitter:title" content="[^"]*\| By ([^"]+)""#,

            // Facebook Reels specific patterns (high priority) - look for full names
            r#""name":"([^"]+)","short_name":"[^"]*","id":"[0-9]+","profile_picture"#,
            r#""page_name":"([^"]+)","page_id":"[0-9]+"#,
            r#""actor":\{"name":"([^"]+)","id":"[0-9]+"#,
            r#""owner":\{"name":"([^"]+)","id":"[0-9]+"#,
            r#""page":\{"name":"([^"]+)","id":"[0-9]+"#,
            r#""user":\{"name":"([^"]+)","id":"[0-9]+"#,

            // Enhanced patterns for Facebook pages/profiles - prioritize full names
            r#""profile_name":"([^"]+)""#,
            r#""display_name":"([^"]+)""#,
            r#""page_profile":\{"name":"([^"]+)""#,
            r#""creator":\{"name":"([^"]+)""#,
            r#""channel":\{"name":"([^"]+)""#,
            r#""account":\{"name":"([^"]+)""#,

            // Look for full page names in structured data
            r#""@type":"Person","name":"([^"]+)""#,
            r#""@type":"Organization","name":"([^"]+)""#,

            // URL-based author extraction (from Facebook URLs)
            r#""url":"https://www\.facebook\.com/([^/"]+)/[^"]*""#,
            r#"facebook\.com/([^/"]+)/videos/"#,
            r#"facebook\.com/([^/"]+)/posts/"#,

            // Standard patterns (lower priority)
            r#""name":"([^"]+)","url":"https://www\.facebook\.com/[^"]+""#,
            r#""author":\{"@type":"Person","name":"([^"]+)""#,
            r#""page_name":"([^"]+)""#,
            r#""owner":\{"name":"([^"]+)""#,
            r#""publisher":\{"name":"([^"]+)""#,
            r#""uploader":"([^"]+)""#,
        ];

        let mut found_authors = Vec::new();

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(author_match) = capture.get(1) {
                        let mut author = author_match.as_str().replace("\\", "");
                        // Decode HTML entities and Unicode escapes in author name
                        author = self.decode_html_entities(&author);
                        author = self.decode_unicode_escapes(&author);

                        // Clean up common prefixes that might be included
                        author = author.trim().to_string();
                        if author.starts_with("By ") {
                            author = author[3..].trim().to_string();
                        }

                        if !author.is_empty() && author.len() > 2 && author != "Facebook" {
                            found_authors.push(author);
                        }
                    }
                }
            }
        }

        // Prefer longer, more complete names over shortened versions
        if !found_authors.is_empty() {
            // Sort by length (descending) and prefer names with spaces (full names)
            found_authors.sort_by(|a, b| {
                let a_has_space = a.contains(' ');
                let b_has_space = b.contains(' ');

                // Prefer names with spaces first
                if a_has_space && !b_has_space {
                    std::cmp::Ordering::Less
                } else if !a_has_space && b_has_space {
                    std::cmp::Ordering::Greater
                } else {
                    // If both have spaces or both don't, prefer longer names
                    b.len().cmp(&a.len())
                }
            });

            return self.clean_author_name(&found_authors[0]);
        }

        // Enhanced fallback for share URLs - try to extract from URL patterns
        if let Some(author) = self.extract_author_from_url_patterns(html) {
            return self.clean_author_name(&author);
        }

        "Unknown Author".to_string()
    }

    /// Clean author name by removing common prefixes and suffixes
    fn clean_author_name(&self, author: &str) -> String {
        let mut cleaned = author.trim().to_string();

        // Remove "By" prefix (case insensitive)
        if cleaned.to_lowercase().starts_with("by ") {
            cleaned = cleaned[3..].trim().to_string();
        }

        // Remove other common prefixes
        let prefixes_to_remove = ["Author: ", "Creator: ", "Channel: ", "Page: "];
        for prefix in &prefixes_to_remove {
            if cleaned.to_lowercase().starts_with(&prefix.to_lowercase()) {
                cleaned = cleaned[prefix.len()..].trim().to_string();
            }
        }

        // Remove common suffixes
        let suffixes_to_remove = [" - Facebook", " | Facebook", " on Facebook"];
        for suffix in &suffixes_to_remove {
            if cleaned.to_lowercase().ends_with(&suffix.to_lowercase()) {
                cleaned = cleaned[..cleaned.len() - suffix.len()].trim().to_string();
            }
        }

        // Final cleanup
        cleaned = cleaned.trim().to_string();

        // Return cleaned name or fallback
        if cleaned.is_empty() || cleaned.len() < 2 {
            "Unknown Author".to_string()
        } else {
            cleaned
        }
    }

    /// Extract author from Facebook's title format
    pub fn extract_author_from_title(&self, html: &str) -> Option<String> {
        // Look for title tag first
        if let Ok(regex) = Regex::new(r#"<title>([^<]+)</title>"#) {
            if let Some(capture) = regex.captures(html) {
                if let Some(title_match) = capture.get(1) {
                    let title = title_match.as_str();

                    // Decode HTML entities and Unicode escapes first
                    let decoded_title = self.decode_html_entities(title);
                    let decoded_title = self.decode_unicode_escapes(&decoded_title);

                    // Look for "By [Author]" pattern FIRST (highest priority for reels)
                    if decoded_title.contains(" | By ") {
                        if let Some(author_part) = decoded_title.split(" | By ").nth(1) {
                            // Clean up the author name
                            let author = author_part
                                .replace(" - Facebook", "")
                                .replace(" | Facebook", "")
                                .trim()
                                .to_string();

                            if !author.is_empty() && author != "Facebook" {
                                return Some(author);
                            }
                        }
                    }

                    // Handle mobile Facebook format: "Title | Author | Facebook" (fallback)
                    if decoded_title.contains(" | ") {
                        let parts: Vec<&str> = decoded_title.split(" | ").collect();

                        // If we have at least 2 parts, the second part might be the author
                        if parts.len() >= 2 {
                            let potential_author = parts[1].trim()
                                .replace(" - Facebook", "")
                                .replace(" | Facebook", "")
                                .trim()
                                .to_string();

                            // Check if this looks like an author name (not "Facebook" and has reasonable length)
                            if !potential_author.is_empty() &&
                               potential_author != "Facebook" &&
                               potential_author.len() > 2 &&
                               !self.is_engagement_data(&potential_author) {
                                return Some(potential_author);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract author from URL patterns in HTML (fallback for share URLs)
    fn extract_author_from_url_patterns(&self, html: &str) -> Option<String> {
        let url_patterns = vec![
            // Look for Facebook profile/page URLs in the HTML
            r#"facebook\.com/([^/"?]+)/?[^"]*"#,
            r#"href="https://www\.facebook\.com/([^/"?]+)"#,
            r#"url":"https://www\.facebook\.com/([^/"?]+)"#,
            // Look for canonical URLs
            r#"<link rel="canonical" href="https://www\.facebook\.com/([^/"?]+)"#,
            // Look for og:url meta tags
            r#"<meta property="og:url" content="https://www\.facebook\.com/([^/"?]+)"#,
        ];

        for pattern in url_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for capture in regex.captures_iter(html) {
                    if let Some(username_match) = capture.get(1) {
                        let username = username_match.as_str();

                        // Filter out common non-username patterns
                        if !self.is_valid_username(username) {
                            continue;
                        }

                        // Convert username to a more readable format
                        let author_name = self.format_username_as_author(username);
                        if !author_name.is_empty() {
                            return Some(author_name);
                        }
                    }
                }
            }
        }

        None
    }

    /// Check if a username is valid (not a system path or common non-username)
    fn is_valid_username(&self, username: &str) -> bool {
        let invalid_usernames = [
            "www", "m", "mobile", "touch", "share", "video", "watch", "reel", "story",
            "login", "signup", "help", "about", "privacy", "terms", "support",
            "api", "graph", "developers", "business", "ads", "pages", "groups",
            "events", "marketplace", "gaming", "jobs", "dating", "weather",
            "plugins", "tr", "ajax", "dialog", "sharer", "v2.0", "v3.0",
        ];

        // Must be at least 3 characters
        if username.len() < 3 {
            return false;
        }

        // Must not be in invalid list
        if invalid_usernames.contains(&username.to_lowercase().as_str()) {
            return false;
        }

        // Must not be all numbers
        if username.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Must not contain special characters that indicate it's not a username
        if username.contains('.') || username.contains('-') && username.len() > 20 {
            return false;
        }

        true
    }

    /// Format username as author name (convert from URL format to display format)
    fn format_username_as_author(&self, username: &str) -> String {
        // If it looks like a page ID (all numbers), skip it
        if username.chars().all(|c| c.is_ascii_digit()) {
            return String::new();
        }

        // Convert common username patterns to readable names
        let formatted = username
            .replace('.', " ")
            .replace('-', " ")
            .replace('_', " ");

        // Capitalize words
        let words: Vec<String> = formatted
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect();

        let result = words.join(" ");

        // Only return if it looks like a reasonable name
        if result.len() > 2 && result.len() < 50 {
            result
        } else {
            String::new()
        }
    }

    /// Extract video description from HTML
    fn extract_description_from_html(&self, html: &str) -> String {
        let patterns = vec![
            r#""description":"([^"]+)""#,
            r#""text":"([^"]+)","ranges":\[\]"#,
            r#"<meta property="og:description" content="([^"]+)""#,
            r#""message":"([^"]+)""#,
            r#""story":"([^"]+)""#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(desc_match) = capture.get(1) {
                        let mut description = desc_match.as_str().replace("\\", "");
                        description = description.replace("\\n", "\n");
                        description = description.replace("\\u0026", "&");
                        description = description.replace("\\u003C", "<");
                        description = description.replace("\\u003E", ">");

                        if !description.is_empty() && description.len() > 10 {
                            return description;
                        }
                    }
                }
            }
        }

        "No description available".to_string()
    }

    /// Extract publish date from HTML
    fn extract_publish_date_from_html(&self, html: &str) -> String {
        let patterns = vec![
            r#""publish_time":(\d+)"#,
            r#""creation_time":(\d+)"#,
            r#""datePublished":"([^"]+)""#,
            r#"data-utime="(\d+)""#,
            r#""timestamp":\{"time":(\d+)"#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(time_match) = capture.get(1) {
                        let time_str = time_match.as_str();

                        // Try to parse as timestamp
                        if let Ok(timestamp) = time_str.parse::<i64>() {
                            // Convert timestamp to readable date
                            if timestamp > 1000000000 {
                                if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp, 0) {
                                    return datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                                }
                            }
                        } else {
                            // Return as-is if it's already a formatted date
                            return time_str.to_string();
                        }
                    }
                }
            }
        }

        "Unknown date".to_string()
    }

    /// Extract likes count from HTML
    fn extract_likes_from_html(&self, html: &str) -> String {
        let patterns = vec![
            // Facebook Reels specific patterns with K notation
            r#""reaction_count":\{"count":(\d+)"#,
            r#""like_count":(\d+)"#,
            r#""reaction_count":(\d+)"#,
            r#"(\d+\.?\d*[KMB]?) reactions"#,
            r#"(\d+\.?\d*[KMB]?) likes"#,
            r#"(\d+) likes"#,
            r#"(\d+) reactions"#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(count_match) = capture.get(1) {
                        let count_str = count_match.as_str();
                        return self.format_engagement_count(count_str);
                    }
                }
            }
        }

        "0".to_string()
    }

    /// Extract comments count from HTML
    fn extract_comments_from_html(&self, html: &str) -> String {
        let patterns = vec![
            // Facebook Reels specific patterns (highest priority)
            r#""comment_count":\{"total_count":(\d+)"#,
            r#""comments_count":(\d+)"#,
            r#""total_comment_count":(\d+)"#,
            r#""comment_count":(\d+)"#,

            // Enhanced text-based patterns for comments
            r#"(\d+\.?\d*[KMB]?) comments"#,
            r#"(\d+) comments"#,

            // Additional Facebook-specific patterns
            r#""total_comments":(\d+)"#,
            r#""comments_total_count":(\d+)"#,
            r#""comment_total":(\d+)"#,

            // Engagement-related patterns (from debug findings)
            r#""feedback_count":(\d+)"#,
            r#""comment_count_reduced":"([^"]+)""#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(count_match) = capture.get(1) {
                        let count_str = count_match.as_str();
                        return self.format_engagement_count(count_str);
                    }
                }
            }
        }

        "0".to_string()
    }

    /// Extract views count from HTML
    fn extract_views_from_html(&self, html: &str) -> String {
        let patterns = vec![
            r#""view_count":(\d+)"#,
            r#""play_count":(\d+)"#,
            r#""video_view_count":(\d+)"#,
            r#"(\d+) views"#,
            r#"(\d+[\.,]\d+[KMB]?) views"#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(count_match) = capture.get(1) {
                        return count_match.as_str().to_string();
                    }
                }
            }
        }

        "0".to_string()
    }

    /// Extract shares count from HTML
    fn extract_shares_from_html(&self, html: &str) -> String {
        let patterns = vec![
            // Facebook Reels specific patterns (HIGHEST PRIORITY - found in debug)
            r#""share_count_reduced":"([^"]+)""#,
            r#""share_count":\{"count":(\d+)"#,
            r#""shares_count":(\d+)"#,
            r#""reshare_count":(\d+)"#,
            r#""total_share_count":(\d+)"#,
            r#""share_count":(\d+)"#,

            // Enhanced text-based patterns for shares
            r#"(\d+\.?\d*[KMB]?) shares"#,
            r#"(\d+\.?\d*[KMB]?) share"#,
            r#"(\d+) shares"#,
            r#"(\d+) share"#,

            // Additional Facebook-specific patterns
            r#""sharing_count":(\d+)"#,
            r#""forward_count":(\d+)"#,
            r#""repost_count":(\d+)"#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(count_match) = capture.get(1) {
                        let count_str = count_match.as_str();
                        return self.format_engagement_count(count_str);
                    }
                }
            }
        }

        "0".to_string()
    }

    /// Format engagement count (handle K, M, B notation)
    fn format_engagement_count(&self, count_str: &str) -> String {
        // If it already has K, M, B notation, return as-is
        if count_str.contains('K') || count_str.contains('M') || count_str.contains('B') {
            return count_str.to_string();
        }

        // If it's a plain number, try to parse and format
        if let Ok(count) = count_str.parse::<u64>() {
            if count >= 1_000_000_000 {
                format!("{:.1}B", count as f64 / 1_000_000_000.0)
            } else if count >= 1_000_000 {
                format!("{:.1}M", count as f64 / 1_000_000.0)
            } else if count >= 1_000 {
                format!("{:.1}K", count as f64 / 1_000.0)
            } else {
                count.to_string()
            }
        } else {
            count_str.to_string()
        }
    }

    /// Parse engagement count string to u64 (handle K, M, B notation)
    fn parse_engagement_count(&self, count_str: &str) -> u64 {
        let count_str = count_str.trim();

        if count_str.is_empty() || count_str == "0" {
            return 0;
        }

        // Handle K, M, B notation
        if count_str.ends_with('K') {
            if let Ok(num) = count_str[..count_str.len()-1].parse::<f64>() {
                return (num * 1_000.0) as u64;
            }
        } else if count_str.ends_with('M') {
            if let Ok(num) = count_str[..count_str.len()-1].parse::<f64>() {
                return (num * 1_000_000.0) as u64;
            }
        } else if count_str.ends_with('B') {
            if let Ok(num) = count_str[..count_str.len()-1].parse::<f64>() {
                return (num * 1_000_000_000.0) as u64;
            }
        }

        // Try to parse as plain number
        count_str.parse::<u64>().unwrap_or(0)
    }

    /// Extract hashtags from HTML
    fn extract_hashtags_from_html(&self, html: &str) -> Vec<String> {
        let mut hashtags = Vec::new();

        let patterns = vec![
            // Facebook Reels specific hashtag patterns
            r#""hashtag":"([^"]+)""#,
            r#""tag":"(\w+)""#,
            // General hashtag patterns - only match actual hashtags
            r#"#([a-zA-Z][a-zA-Z0-9_]*)"#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for capture in regex.captures_iter(html) {
                    if let Some(tag_match) = capture.get(1) {
                        let hashtag = tag_match.as_str().to_string();

                        // Filter out HTML entities, color codes, and invalid hashtags
                        if self.is_valid_hashtag(&hashtag) {
                            let formatted_hashtag = if hashtag.starts_with('#') {
                                hashtag
                            } else {
                                format!("#{}", hashtag)
                            };

                            if !hashtags.contains(&formatted_hashtag) {
                                hashtags.push(formatted_hashtag);
                            }
                        }
                    }
                }
            }
        }

        hashtags
    }

    /// Check if a hashtag is valid (not HTML entity, color code, etc.)
    fn is_valid_hashtag(&self, hashtag: &str) -> bool {
        // Filter out HTML entities like "xf4", "x1ecf", etc.
        if hashtag.starts_with('x') && hashtag.len() <= 6 {
            return false;
        }

        // Filter out color codes
        if hashtag.len() == 6 && hashtag.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }

        // Filter out short codes like "064", "123", "125"
        if hashtag.len() <= 3 && hashtag.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Filter out common non-hashtag patterns
        if hashtag == "Intent" || hashtag == "FFFFFF" || hashtag == "000000" {
            return false;
        }

        // Must be at least 2 characters and start with a letter
        hashtag.len() >= 2 && hashtag.chars().next().unwrap_or('0').is_alphabetic()
    }

    /// Extract duration in seconds from HTML (for VideoMetadata)
    fn extract_duration_seconds_from_html(&self, html: &str) -> Option<u32> {
        let patterns = vec![
            // Facebook Reels specific patterns (highest priority)
            r#""duration_s":(\d+)"#,
            r#""length_seconds":(\d+)"#,
            r#""video_duration":(\d+)"#,
            r#""playable_duration_in_ms":(\d+)"#,

            // General patterns
            r#""duration":(\d+)"#,
            r#""lengthSeconds":"(\d+)""#,
            r#""duration":"(\d+)""#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(duration_match) = capture.get(1) {
                        if let Ok(mut seconds) = duration_match.as_str().parse::<u32>() {
                            // Handle milliseconds conversion
                            if pattern.contains("_in_ms") {
                                seconds = seconds / 1000;
                            }

                            // Only accept plausible video durations (5s to 10min)
                            if (5..=600).contains(&seconds) {
                                return Some(seconds);
                            }
                        }
                    }
                }
            }
        }

        // More specific fallback patterns
        let fallback_patterns = vec![
            r#""duration_s":(\d+)"#,
            r#""t":(\d+),"#,  // More specific than just "t":
        ];

        for pattern in fallback_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(duration_match) = capture.get(1) {
                        if let Ok(seconds) = duration_match.as_str().parse::<u32>() {
                            // Only accept plausible video durations (5s to 10min)
                            if (5..=600).contains(&seconds) {
                                return Some(seconds);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Detect privacy level from HTML content
    pub fn detect_privacy_level_from_html(&self, html: &str) -> Option<PrivacyLevel> {

        // Check for explicit privacy indicators
        let private_indicators = [
            "This content isn't available right now",
            "This video is private",
            "Only friends can see this",
            "This post is no longer available",
            "You must log in to continue",
            "Sign up for Facebook to continue",
            "Create an account or log in to Facebook",
            "private_video",
            "friends_only",
            "restricted_content"
        ];

        let friends_indicators = [
            "Only friends can see this",
            "friends_only",
            "visible_to_friends"
        ];

        // Check for public indicators
        let public_indicators = [
            "Public",
            "Everyone",
            "public_video",
            "publicly_visible",
            "globe icon", // Facebook uses globe icon for public posts
            "data-privacy=\"public\"",
            "privacy_level\":\"public\"",
            "audience\":\"public\"",
            "visibility\":\"public\"",
            // Facebook reels are often public by default
            "facebook.com/reel",
            "facebook.com/watch",
            // CDN URLs suggest public content
            "fbcdn.net"
        ];

        // Check for private content first (highest priority)
        for indicator in private_indicators {
            if html.to_lowercase().contains(&indicator.to_lowercase()) {
                return Some(PrivacyLevel::Private);
            }
        }

        // Check for friends-only content
        for indicator in friends_indicators {
            if html.to_lowercase().contains(&indicator.to_lowercase()) {
                return Some(PrivacyLevel::Friends);
            }
        }

        // Check for public indicators
        for indicator in public_indicators {
            if html.to_lowercase().contains(&indicator.to_lowercase()) {
                return Some(PrivacyLevel::Public);
            }
        }

        // Additional heuristics for public content
        // If we found video URLs from CDN, it's likely public
        if html.contains("fbcdn.net") && html.contains(".mp4") {
            return Some(PrivacyLevel::Public);
        }

        // If we can extract video metadata without auth, it's likely public
        if !html.contains("log in") && !html.contains("sign up") &&
           (html.contains("video") || html.contains("reel")) {
            return Some(PrivacyLevel::Public);
        }

        // Default to unknown if we can't determine
        None
    }

    /// Extract video duration from HTML
    pub fn extract_duration_from_html(&self, html: &str) -> String {
        let patterns = vec![
            r#"duration_s":(\d+)"#,
            r#"duration":(\d+)"#,
            r#"length_seconds":(\d+)"#,
            r#"video_duration":(\d+)"#,
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(capture) = regex.captures(html) {
                    if let Some(duration_match) = capture.get(1) {
                        if let Ok(seconds) = duration_match.as_str().parse::<u32>() {
                            let minutes = seconds / 60;
                            let remaining_seconds = seconds % 60;
                            return format!("{}:{:02}", minutes, remaining_seconds);
                        }
                    }
                }
            }
        }
        // Fallback: Try to extract '"t":<number>' if no other pattern matches
        if let Ok(regex) = Regex::new(r#"t":(\d+)"#) {
            if let Some(capture) = regex.captures(html) {
                if let Some(duration_match) = capture.get(1) {
                    if let Ok(seconds) = duration_match.as_str().parse::<u32>() {
                        if (5..=600).contains(&seconds) {
                            let minutes = seconds / 60;
                            let remaining_seconds = seconds % 60;
                            return format!("{}:{:02}", minutes, remaining_seconds);
                        }
                    }
                }
            }
        }
        "Unknown duration".to_string()
    }

    /// Extract video thumbnail from HTML with enhanced debugging and download to data URL
    pub async fn extract_thumbnail_from_html(&self, html: &str) -> String {
        println!("🖼️ THUMBNAIL EXTRACTION CALLED - Starting enhanced thumbnail extraction from HTML");
        tracing::info!("🖼️ Starting enhanced thumbnail extraction from HTML");

        // First extract the CDN URL using existing logic
        let cdn_url = self.extract_thumbnail_url_from_html(html);

        if cdn_url.is_empty() {
            tracing::warn!("❌ No thumbnail URL found in HTML");
            return self.create_fallback_thumbnail();
        }

        tracing::info!("✅ Found thumbnail CDN URL: {}", &cdn_url[..100.min(cdn_url.len())]);

        // Try to download the thumbnail and convert to data URL
        match self.download_thumbnail_to_data_url(&cdn_url).await {
            Ok(data_url) => {
                tracing::info!("✅ Successfully downloaded thumbnail and converted to data URL");
                data_url
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to download thumbnail: {}, using fallback", e);
                self.create_fallback_thumbnail()
            }
        }
    }

    /// Extract enhanced thumbnail collection with multiple variants and aspect ratios
    pub async fn extract_thumbnail_collection(
        &self,
        html: &str,
        video_aspect_ratio: AspectRatio,
    ) -> ThumbnailCollection {
        use crate::processing::thumbnail::ThumbnailExtractor;

        tracing::info!("🖼️ Starting enhanced thumbnail collection extraction");

        // Create thumbnail extractor
        let thumbnail_extractor = match ThumbnailExtractor::new() {
            Ok(extractor) => extractor,
            Err(e) => {
                tracing::warn!("⚠️ Failed to create thumbnail extractor: {}, using fallback", e);
                return self.create_fallback_thumbnail_collection(video_aspect_ratio);
            }
        };

        // Extract comprehensive thumbnail collection
        let collection = thumbnail_extractor
            .extract_thumbnail_collection(html, video_aspect_ratio)
            .await;

        // If no variants were created, create fallback collection
        if collection.variants.is_empty() {
            tracing::warn!("❌ No thumbnail variants extracted, creating fallback collection");
            self.create_fallback_thumbnail_collection(video_aspect_ratio)
        } else {
            tracing::info!("✅ Successfully extracted {} thumbnail variants", collection.variants.len());
            collection
        }
    }

    /// Create fallback thumbnail collection when extraction fails
    fn create_fallback_thumbnail_collection(&self, video_aspect_ratio: AspectRatio) -> ThumbnailCollection {
        // ThumbnailVariant and DisplayContext already imported at top

        let mut collection = ThumbnailCollection::new(video_aspect_ratio);

        // Create fallback variants for different display contexts
        let contexts = vec![
            DisplayContext::DesktopFullscreen,
            DisplayContext::MobilePortrait,
            DisplayContext::MobileLandscape,
            DisplayContext::WebThumbnail,
            DisplayContext::SocialShare,
            DisplayContext::PlayerPreview,
        ];

        for context in contexts {
            let (width, height) = context.preferred_dimensions();
            let svg_data_url = self.create_fallback_svg_for_context(width, height, context);

            let variant = ThumbnailVariant::new(
                width,
                height,
                svg_data_url,
                "svg".to_string(),
                vec![context],
            );

            collection.add_variant(variant);
        }

        collection
    }

    /// Create fallback SVG for specific context
    fn create_fallback_svg_for_context(&self, width: u32, height: u32, context: DisplayContext) -> String {
        let context_label = match context {
            DisplayContext::DesktopFullscreen => "Desktop",
            DisplayContext::MobilePortrait => "Mobile Portrait",
            DisplayContext::MobileLandscape => "Mobile Landscape",
            DisplayContext::WebThumbnail => "Web Thumbnail",
            DisplayContext::SocialShare => "Social Share",
            DisplayContext::PlayerPreview => "Player Preview",
        };

        let svg_content = format!(
            "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"100%\" height=\"100%\" fill=\"#f3f4f6\"/><circle cx=\"50%\" cy=\"35%\" r=\"15\" fill=\"#9ca3af\"/><polygon points=\"45,30 45,40 55,35\" fill=\"#f3f4f6\"/><text x=\"50%\" y=\"55%\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"10\" fill=\"#6b7280\">No Thumbnail</text><text x=\"50%\" y=\"70%\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"8\" fill=\"#9ca3af\">{}</text><text x=\"50%\" y=\"85%\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"8\" fill=\"#9ca3af\">{}x{}</text></svg>",
            width, height, context_label, width, height
        );

        format!(
            "data:image/svg+xml;base64,{}",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, svg_content.as_bytes())
        )
    }

    /// Extract thumbnail URL from HTML (enhanced with comprehensive patterns)
    fn extract_thumbnail_url_from_html(&self, html: &str) -> String {
        println!("🔍 THUMBNAIL URL EXTRACTION CALLED - Starting comprehensive thumbnail URL extraction");
        println!("📄 HTML content length: {} characters", html.len());
        tracing::info!("🔍 Starting comprehensive thumbnail URL extraction");
        tracing::info!("📄 HTML content length: {} characters", html.len());

        // Debug: Show a sample of the HTML content to understand structure
        self.debug_html_content(html);

        // Enhanced patterns with more comprehensive coverage for current Facebook structure
        let patterns = vec![
            // CRITICAL: Facebook's preferred_thumbnail pattern (most important)
            r#""preferred_thumbnail":\{"image":\{"uri":"([^"]+)""#,

            // Modern Facebook JSON-LD patterns
            r#""thumbnailUrl":"([^"]+)""#,
            r#""thumbnail":"([^"]+)""#,
            r#""image":"([^"]+\.jpg[^"]*)""#,
            r#""image":"([^"]+\.png[^"]*)""#,
            r#""image":"([^"]+\.webp[^"]*)""#,

            // Video poster and preview images
            r#""poster":"([^"]+)""#,
            r#""preview_image":"([^"]+)""#,
            r#""video_preview_image_url":"([^"]+)""#,

            // Facebook CDN direct patterns (with escaped slashes)
            r#""(https:\\?/\\?/[^"]*\.fbcdn\.net[^"]*\.(jpg|jpeg|png|webp)[^"]*)""#,
            r#"'(https:\\?/\\?/[^']*\.fbcdn\.net[^']*\.(jpg|jpeg|png|webp)[^']*)'"#,

            // Open Graph and meta tags
            r#"<meta property="og:image" content="([^"]+)""#,
            r#"<meta name="twitter:image" content="([^"]+)""#,
            r#"<meta property="og:image:url" content="([^"]+)""#,

            // Facebook-specific patterns
            r#""preview_image":"([^"]+)""#,
            r#""thumbnailImage":"([^"]+)""#,
            r#""poster":"([^"]+)""#,
            r#""previewImage":"([^"]+)""#,
            r#""image":\{"uri":"([^"]+)""#,
            r#""thumbnailUrl":"([^"]+)""#,
            r#""image_url":"([^"]+)""#,
            r#""cover_photo":\{"source":"([^"]+)""#,

            // Video-specific thumbnail patterns
            r#""video_thumbnail":"([^"]+)""#,
            r#""video_preview_image":"([^"]+)""#,
            r#""playable_url_quality_hd_thumbnail":"([^"]+)""#,

            // Escaped JSON patterns
            r#"\\"thumbnail\\":\\"([^"]+)\\""#,
            r#"\\"image\\":\\"([^"]+)\\""#,

            // Alternative formats
            r#"thumbnail_url=([^&\s]+)"#,
            r#"preview_url=([^&\s]+)"#,
        ];

        let mut all_candidates = Vec::new();

        for (index, pattern) in patterns.iter().enumerate() {
            tracing::debug!("🔍 Trying thumbnail pattern {}: {}", index + 1, pattern);

            if let Ok(regex) = Regex::new(pattern) {
                let matches: Vec<_> = regex.captures_iter(html).collect();
                tracing::debug!("   📊 Found {} matches for pattern {}", matches.len(), index + 1);

                for (match_index, capture) in matches.iter().enumerate() {
                    if let Some(thumb_match) = capture.get(1) {
                        let mut thumbnail = thumb_match.as_str().to_string();

                        // Clean up escaped characters
                        thumbnail = thumbnail.replace("\\", "");
                        thumbnail = thumbnail.replace("\\/", "/");
                        thumbnail = thumbnail.replace("\\u0026", "&");

                        tracing::info!("✅ Found thumbnail candidate {} from pattern {}: {}",
                            match_index + 1, index + 1, &thumbnail[..100.min(thumbnail.len())]);

                        if thumbnail.starts_with("http") && thumbnail.len() > 20 {
                            tracing::info!("✅ Valid thumbnail URL found: {}", &thumbnail[..100.min(thumbnail.len())]);

                            // Validate URL format more thoroughly
                            if thumbnail.contains("fbcdn.net") || thumbnail.contains("facebook.com") {
                                tracing::info!("🎯 Facebook CDN thumbnail detected - high confidence");
                                all_candidates.push((thumbnail.clone(), 100)); // High priority
                            } else if thumbnail.contains(".jpg") || thumbnail.contains(".png") || thumbnail.contains(".webp") {
                                tracing::info!("🖼️ Image format detected in URL - medium confidence");
                                all_candidates.push((thumbnail.clone(), 50)); // Medium priority
                            } else {
                                tracing::info!("🔗 Generic HTTP URL - low confidence");
                                all_candidates.push((thumbnail.clone(), 10)); // Low priority
                            }
                        } else {
                            tracing::warn!("⚠️ Thumbnail candidate doesn't start with http or is too short: {}",
                                &thumbnail[..50.min(thumbnail.len())]);
                        }
                    }
                }
            } else {
                tracing::warn!("❌ Invalid regex pattern {}: {}", index + 1, pattern);
            }
        }

        // Sort candidates by priority and return the best one
        if !all_candidates.is_empty() {
            all_candidates.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by priority (descending)

            tracing::info!("📊 Thumbnail extraction summary:");
            tracing::info!("   🎯 Total candidates found: {}", all_candidates.len());

            for (i, (url, priority)) in all_candidates.iter().take(3).enumerate() {
                tracing::info!("   {}. Priority {}: {}", i + 1, priority, &url[..80.min(url.len())]);
            }

            let best_thumbnail = &all_candidates[0].0;
            tracing::info!("✅ Selected best thumbnail: {}", &best_thumbnail[..100.min(best_thumbnail.len())]);
            return best_thumbnail.clone();
        }

        // If no candidates found, try a more aggressive search
        tracing::warn!("❌ No thumbnail candidates found with standard patterns");
        tracing::info!("🔍 Attempting aggressive thumbnail search...");

        // Look for any fbcdn.net image URLs
        if let Ok(regex) = Regex::new(r#"https://[^"]*\.fbcdn\.net/[^"]*\.(jpg|jpeg|png|webp)[^"]*"#) {
            let matches: Vec<_> = regex.find_iter(html).collect();
            tracing::info!("   📊 Found {} fbcdn.net image URLs", matches.len());

            for (i, m) in matches.iter().take(5).enumerate() {
                let url = m.as_str();
                tracing::info!("   {}. {}", i + 1, &url[..100.min(url.len())]);

                // Return the first one as a fallback
                if i == 0 {
                    tracing::info!("✅ Using first fbcdn.net image as fallback thumbnail");
                    return url.to_string();
                }
            }
        }

        tracing::warn!("❌ No thumbnail found in HTML after exhaustive search");
        tracing::info!("💡 Consider checking if the video is private or requires authentication");
        String::new()
    }

    /// Download thumbnail from CDN URL and convert to data URL using multiple strategies
    async fn download_thumbnail_to_data_url(&self, cdn_url: &str) -> Result<String, String> {
        tracing::info!("🔄 Downloading thumbnail from CDN: {}", &cdn_url[..100.min(cdn_url.len())]);

        // Create HTTP client with working configuration from our tests
        let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(10))
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // Strategy 1: Browser-like headers with Facebook referer (most likely to work)
        tracing::info!("🔄 Strategy 1: Browser headers with Facebook referer");
        match self.try_download_with_browser_headers(&client, cdn_url).await {
            Ok(data_url) => {
                tracing::info!("✅ Strategy 1 succeeded");
                return Ok(data_url);
            }
            Err(e) => {
                tracing::warn!("⚠️ Strategy 1 failed: {}", e);
            }
        }

        // Strategy 2: Mobile browser headers
        tracing::info!("🔄 Strategy 2: Mobile browser headers");
        match self.try_download_with_mobile_headers(&client, cdn_url).await {
            Ok(data_url) => {
                tracing::info!("✅ Strategy 2 succeeded");
                return Ok(data_url);
            }
            Err(e) => {
                tracing::warn!("⚠️ Strategy 2 failed: {}", e);
            }
        }

        // Strategy 3: Minimal headers (last resort)
        tracing::info!("🔄 Strategy 3: Minimal headers");
        match self.try_download_with_minimal_headers(&client, cdn_url).await {
            Ok(data_url) => {
                tracing::info!("✅ Strategy 3 succeeded");
                return Ok(data_url);
            }
            Err(e) => {
                tracing::warn!("⚠️ Strategy 3 failed: {}", e);
            }
        }

        Err("All thumbnail download strategies failed".to_string())
    }

    /// Try downloading with browser-like headers
    async fn try_download_with_browser_headers(&self, client: &reqwest::Client, cdn_url: &str) -> Result<String, String> {
        let response = client
            .get(cdn_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Referer", "https://www.facebook.com/")
            .header("Origin", "https://www.facebook.com")
            .header("Connection", "keep-alive")
            .header("Sec-Fetch-Dest", "image")
            .header("Sec-Fetch-Mode", "no-cors")
            .header("Sec-Fetch-Site", "cross-site")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        self.process_thumbnail_response(response).await
    }

    /// Try downloading with mobile browser headers
    async fn try_download_with_mobile_headers(&self, client: &reqwest::Client, cdn_url: &str) -> Result<String, String> {
        let response = client
            .get(cdn_url)
            .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")
            .header("Accept", "image/*,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Connection", "keep-alive")
            .header("Referer", "https://m.facebook.com/")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        self.process_thumbnail_response(response).await
    }

    /// Try downloading with minimal headers
    async fn try_download_with_minimal_headers(&self, client: &reqwest::Client, cdn_url: &str) -> Result<String, String> {
        let response = client
            .get(cdn_url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        self.process_thumbnail_response(response).await
    }

    /// Process HTTP response and convert to data URL
    async fn process_thumbnail_response(&self, response: reqwest::Response) -> Result<String, String> {
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let bytes = response.bytes().await
            .map_err(|e| format!("Failed to read response bytes: {}", e))?;

        if bytes.len() == 0 {
            return Err("Empty response".to_string());
        }

        if bytes.len() < 100 {
            tracing::warn!("⚠️ Downloaded thumbnail is very small ({} bytes) - might be an error", bytes.len());
            return Err("Downloaded content too small to be a valid image".to_string());
        }

        // Determine image format and create data URL
        let (mime_type, _extension) = if bytes.starts_with(b"\xFF\xD8\xFF") {
            ("image/jpeg", "jpg")
        } else if bytes.starts_with(b"\x89PNG") {
            ("image/png", "png")
        } else if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WEBP" {
            ("image/webp", "webp")
        } else {
            ("image/jpeg", "jpg") // Default to JPEG
        };

        // Convert to base64 data URL
        let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);

        tracing::info!("✅ Successfully downloaded {} bytes and converted to data URL", bytes.len());
        Ok(data_url)
    }

    /// Create fallback SVG thumbnail when download fails
    fn create_fallback_thumbnail(&self) -> String {
        tracing::info!("🎨 Creating fallback SVG thumbnail");

        // Create SVG content
        let mut svg_content = String::new();
        svg_content.push_str(r#"<svg width="320" height="180" xmlns="http://www.w3.org/2000/svg">"#);
        svg_content.push_str(r#"<defs><linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">"#);
        svg_content.push_str("<stop offset=\"0%\" style=\"stop-color:#1877f2;stop-opacity:1\" />");
        svg_content.push_str("<stop offset=\"100%\" style=\"stop-color:#42a5f5;stop-opacity:1\" />");
        svg_content.push_str(r#"</linearGradient></defs>"#);
        svg_content.push_str(r#"<rect width="320" height="180" fill="url(#bg)"/>"#);
        svg_content.push_str(r#"<circle cx="160" cy="90" r="30" fill="white" opacity="0.9"/>"#);
        svg_content.push_str("<polygon points=\"150,75 150,105 175,90\" fill=\"#1877f2\"/>");
        svg_content.push_str(r#"<text x="160" y="130" font-family="Arial" font-size="12" fill="white" text-anchor="middle">Facebook Video</text>"#);
        svg_content.push_str(r#"<text x="160" y="150" font-family="Arial" font-size="10" fill="white" text-anchor="middle" opacity="0.8">Thumbnail Unavailable</text>"#);
        svg_content.push_str(r#"</svg>"#);

        // Convert to base64 data URL
        let base64_svg = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, svg_content.as_bytes());
        let data_url = format!("data:image/svg+xml;base64,{}", base64_svg);

        tracing::info!("✅ Created fallback SVG thumbnail ({} bytes)", svg_content.len());
        data_url
    }

    /// Debug HTML content to understand structure and find thumbnail patterns
    fn debug_html_content(&self, html: &str) {
        println!("🔍 DEBUG: Analyzing HTML content structure...");
        tracing::info!("🔍 Debugging HTML content structure...");

        // Look for common Facebook thumbnail-related keywords
        let keywords = vec![
            "thumbnail", "preview", "image", "poster", "fbcdn.net",
            "og:image", "twitter:image", "video_thumbnail", "playable_url"
        ];

        for keyword in keywords {
            let count = html.matches(keyword).count();
            if count > 0 {
                println!("   📊 Found '{}': {} occurrences", keyword, count);
                tracing::info!("   📊 Found '{}': {} occurrences", keyword, count);

                // Show context around the first few occurrences
                if let Some(pos) = html.find(keyword) {
                    let start = pos.saturating_sub(100);
                    let end = (pos + keyword.len() + 100).min(html.len());
                    let context = &html[start..end];
                    println!("      Context: ...{}...", context.replace('\n', " "));
                    tracing::info!("      Context: ...{}...", context.replace('\n', " "));
                }
            } else {
                println!("   ❌ No occurrences of '{}'", keyword);
            }
        }

        // Look for any URLs that might be thumbnails
        if let Ok(regex) = Regex::new(r#"https://[^"\s]*\.(jpg|jpeg|png|webp)[^"\s]*"#) {
            let matches: Vec<_> = regex.find_iter(html).collect();
            println!("   🖼️ Found {} potential image URLs", matches.len());
            tracing::info!("   🖼️ Found {} potential image URLs", matches.len());

            for (i, m) in matches.iter().take(5).enumerate() {
                println!("      {}. {}", i + 1, m.as_str());
                tracing::info!("      {}. {}", i + 1, m.as_str());
            }
        }

        // Look for fbcdn.net URLs specifically
        if let Ok(regex) = Regex::new(r#"https://[^"\s]*\.fbcdn\.net[^"\s]*"#) {
            let matches: Vec<_> = regex.find_iter(html).collect();
            println!("   🌐 Found {} fbcdn.net URLs", matches.len());
            tracing::info!("   🌐 Found {} fbcdn.net URLs", matches.len());

            for (i, m) in matches.iter().take(5).enumerate() {
                println!("      {}. {}", i + 1, m.as_str());
                tracing::info!("      {}. {}", i + 1, m.as_str());
            }
        }

        // Look for specific patterns that might contain thumbnails
        let specific_patterns = vec![
            r#""image":\{"uri":"[^"]+""#,
            r#""thumbnail":[^,}]+"#,
            r#""preview_image":[^,}]+"#,
            r#"og:image[^>]*content="[^"]+""#,
        ];

        for (i, pattern) in specific_patterns.iter().enumerate() {
            if let Ok(regex) = Regex::new(pattern) {
                let matches: Vec<_> = regex.find_iter(html).collect();
                if matches.len() > 0 {
                    println!("   🎯 Pattern {} found {} matches: {}", i + 1, matches.len(), pattern);
                    tracing::info!("   🎯 Pattern {} found {} matches: {}", i + 1, matches.len(), pattern);

                    for (j, m) in matches.iter().take(3).enumerate() {
                        println!("      {}. {}", j + 1, m.as_str());
                        tracing::info!("      {}. {}", j + 1, m.as_str());
                    }
                }
            }
        }
    }

    /// Probe the duration of a remote MP4 video by downloading only the header (first 1MB)
    /// and parsing it with mp4parse. This avoids downloading the entire file.
    /// Returns duration in seconds if successful, or None if not available.
    pub async fn probe_duration_from_video_url(&self, _video_url: &str) -> Option<u32> {
        // TODO: Re-implement with alternative MP4 parsing library if needed
        tracing::warn!("probe_duration_from_video_url: currently disabled");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn test_extract_duration_from_html_duration_s() {
        let extractor = MetadataExtractor::new();
        let html = r#"{"duration_s":123}"#;
        let result = extractor.extract_duration_from_html(html);
        assert_eq!(result, "2:03 (123 seconds)");
    }

    #[test]
    fn test_extract_duration_from_html_duration() {
        let extractor = MetadataExtractor::new();
        let html = r#"{"duration":75}"#;
        let result = extractor.extract_duration_from_html(html);
        assert_eq!(result, "1:15 (75 seconds)");
    }

    #[test]
    fn test_extract_duration_from_html_length_seconds() {
        let extractor = MetadataExtractor::new();
        let html = r#"{"length_seconds":360}"#;
        let result = extractor.extract_duration_from_html(html);
        assert_eq!(result, "6:00 (360 seconds)");
    }

    #[test]
    fn test_extract_duration_from_html_video_duration() {
        let extractor = MetadataExtractor::new();
        let html = r#"{"video_duration":9}"#;
        let result = extractor.extract_duration_from_html(html);
        assert_eq!(result, "0:09 (9 seconds)");
    }

    #[test]
    fn test_extract_duration_from_html_no_match() {
        let extractor = MetadataExtractor::new();
        let html = r#"{"no_duration":true}"#;
        let result = extractor.extract_duration_from_html(html);
        assert_eq!(result, "Unknown duration");
    }

    #[test]
    fn test_extract_duration_from_html_multiple_patterns() {
        let extractor = MetadataExtractor::new();
        let html = r#"{"duration":45, "length_seconds":99}"#;
        // Should match the first pattern found ("duration")
        let result = extractor.extract_duration_from_html(html);
        assert_eq!(result, "0:45 (45 seconds)");
    }

    #[test]
    fn test_probe_duration_from_video_url_placeholder() {
        // Placeholder test: just checks that the function returns None for now
        let extractor = MetadataExtractor::new();
        let url = "https://example.com/video.mp4";
        let result = block_on(extractor.probe_duration_from_video_url(url));
        assert!(result.is_none());
    }
}
