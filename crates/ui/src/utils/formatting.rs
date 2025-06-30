pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if size == 0 {
        return "0 B".to_string();
    }

    let size_f = size as f64;
    let unit_index = (size_f.log10() / THRESHOLD.log10()).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);

    let size_in_unit = size_f / THRESHOLD.powi(unit_index as i32);

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_in_unit, UNITS[unit_index])
    }
}

pub fn format_download_speed(speed: f32) -> String {
    format_file_size(speed as u64) + "/s"
}

pub fn format_duration(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Format duration in seconds to MM:SS or H:MM:SS format (like YouTube)
pub fn format_duration_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

/// Format duration to a descriptive format (e.g., "2 minutes 8 seconds")
pub fn format_duration_descriptive(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();

    if hours > 0 {
        if hours == 1 {
            parts.push("1 hour".to_string());
        } else {
            parts.push(format!("{} hours", hours));
        }
    }

    if minutes > 0 {
        if minutes == 1 {
            parts.push("1 minute".to_string());
        } else {
            parts.push(format!("{} minutes", minutes));
        }
    }

    if secs > 0 || parts.is_empty() {
        if secs == 1 {
            parts.push("1 second".to_string());
        } else {
            parts.push(format!("{} seconds", secs));
        }
    }

    if parts.len() == 1 {
        parts[0].clone()
    } else if parts.len() == 2 {
        format!("{} and {}", parts[0], parts[1])
    } else {
        let last = parts.pop().unwrap();
        format!("{}, and {}", parts.join(", "), last)
    }
}

/// Parse duration string to extract seconds
pub fn parse_duration_to_seconds(duration_str: &str) -> Option<u32> {
    // Handle format like "3:04 (184 seconds)"
    if let Some(start) = duration_str.find('(') {
        if let Some(end) = duration_str.find(" seconds)") {
            let seconds_str = &duration_str[start + 1..end];
            return seconds_str.parse().ok();
        }
    }

    // Handle format like "3:04"
    if duration_str.contains(':') {
        let parts: Vec<&str> = duration_str.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(minutes), Ok(seconds)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                return Some(minutes * 60 + seconds);
            }
        } else if parts.len() == 3 {
            // Handle "h:mm:ss" format
            if let (Ok(hours), Ok(minutes), Ok(seconds)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                return Some(hours * 3600 + minutes * 60 + seconds);
            }
        }
    }

    // Handle pure seconds
    if let Ok(seconds) = duration_str.parse::<u32>() {
        return Some(seconds);
    }

    None
}

/// Extract the MM:SS or H:MM:SS part from a duration string
pub fn extract_time_format(duration_str: &str) -> String {
    // If it's already in MM:SS format, return as-is
    if duration_str.matches(':').count() >= 1 && !duration_str.contains('(') {
        return duration_str.to_string();
    }

    // Try to parse and reformat
    if let Some(seconds) = parse_duration_to_seconds(duration_str) {
        format_duration_time(seconds)
    } else {
        "Unknown".to_string()
    }
}

pub fn format_percentage(value: f32) -> String {
    format!("{:.1}%", value * 100.0)
}

/// Format number with thousands separators
pub fn format_number(num: u64) -> String {
    if num < 1000 {
        return num.to_string();
    }

    let mut result = String::new();
    let num_str = num.to_string();
    let chars: Vec<char> = num_str.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    result
}

pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() <= max_length {
        s.to_string()
    } else {
        format!("{}...", &s[..max_length.saturating_sub(3)])
    }
}

pub fn format_timestamp(timestamp: &str) -> String {
    // TODO: Implement proper timestamp formatting
    // For now, just return the timestamp as-is
    timestamp.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
    }

    #[test]
    fn test_format_duration_time() {
        assert_eq!(format_duration_time(30), "0:30");
        assert_eq!(format_duration_time(90), "1:30");
        assert_eq!(format_duration_time(184), "3:04");
        assert_eq!(format_duration_time(204), "3:24");
        assert_eq!(format_duration_time(128), "2:08");
        assert_eq!(format_duration_time(3661), "1:01:01");
    }

    #[test]
    fn test_format_duration_descriptive() {
        assert_eq!(format_duration_descriptive(30), "30 seconds");
        assert_eq!(format_duration_descriptive(60), "1 minute");
        assert_eq!(format_duration_descriptive(90), "1 minute and 30 seconds");
        assert_eq!(format_duration_descriptive(184), "3 minutes and 4 seconds");
        assert_eq!(format_duration_descriptive(3600), "1 hour");
        assert_eq!(format_duration_descriptive(3661), "1 hour, 1 minute, and 1 second");
    }

    #[test]
    fn test_parse_duration_to_seconds() {
        assert_eq!(parse_duration_to_seconds("3:04 (184 seconds)"), Some(184));
        assert_eq!(parse_duration_to_seconds("3:04"), Some(184));
        assert_eq!(parse_duration_to_seconds("3:24"), Some(204));
        assert_eq!(parse_duration_to_seconds("2:08"), Some(128));
        assert_eq!(parse_duration_to_seconds("1:30:45"), Some(5445));
        assert_eq!(parse_duration_to_seconds("184"), Some(184));
        assert_eq!(parse_duration_to_seconds("Unknown duration"), None);
    }

    #[test]
    fn test_extract_time_format() {
        assert_eq!(extract_time_format("3:04 (184 seconds)"), "3:04");
        assert_eq!(extract_time_format("3:04"), "3:04");
        assert_eq!(extract_time_format("Unknown duration"), "Unknown");
        assert_eq!(extract_time_format("184"), "3:04");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(12345), "12,345");
        assert_eq!(format_number(123456), "123,456");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(12345678), "12,345,678");
        assert_eq!(format_number(123456789), "123,456,789");
        assert_eq!(format_number(1000000), "1,000,000");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(0.0), "0.0%");
        assert_eq!(format_percentage(0.5), "50.0%");
        assert_eq!(format_percentage(1.0), "100.0%");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 8), "hi");
    }
}
