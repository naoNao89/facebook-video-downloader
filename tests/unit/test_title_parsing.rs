//! # Title Parsing Test
//!
//! ## Purpose
//! Tests video title extraction and parsing functionality
//!
//! ## Category
//! Content Parsing
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_title_parsing
//! ```
//!
//! ## Dependencies
//! - facebook-extractor-core: Core extraction functionality
//! - reqwest: HTTP client for network requests
//! - tokio: Async runtime
//!
//! ## Setup Requirements
//! - Internet connection for Facebook access
//! - Valid Facebook video URLs for testing

use facebook_extractor_core::metadata::MetadataExtractor;

#[tokio::main]
async fn main() {
    // Test the title parsing with the problematic examples
    let extractor = MetadataExtractor::new();
    
    println!("🧪 Testing Title Parsing Improvements");
    println!("=====================================");
    
    // Test cases based on the actual output we saw
    let test_cases = vec![
        (
            "Raw title from HTML",
            "Du1eaft chu00f3 u0111i du1ea1o nhu01b0ng ko xu00edch, ko ru1ecd mu00f5m | Du1eaft chu00f3 u0111i du1ea1o nhu01b0ng ko xu00edch, ko ru1ecd mu00f5m | By Al Khaleel Studios"
        ),
        (
            "Unicode escaped title",
            "Cu00e1ch siu00eau bu00e3o Yagi hu00ecnh thu00e0nh"
        ),
        (
            "Complex title with hashtags",
            "Cu1ef1c su1ed1t vu1edbi trend u0111u1ed5i chanh du00e2y lu1ea5y tiramisu tu1ea1i tiu1ec7m bu00e1nh 40 nu0103m tuu1ed5i | Cu1ef1c su1ed1t vu1edbi trend u0111u1ed5i chanh du00e2y lu1ea5y tiramisu tu1ea1i tiu1ec7m bu00e1nh 40 nu0103m tuu1ed5i ud83dude0dnn#Y1TD25 #YeaH1 #video #xuhuong"
        ),
        (
            "Simple title",
            "connection_quality"
        ),
        (
            "HTML entities - Vietnamese title",
            "M&#xf4; ph&#x1ecf;ng si&#xea;u ch&#xe2;n th&#x1ef1;c!.."
        ),
        (
            "HTML entities with view counts",
            "2.6M views &#xb7; 9.9K reactions | M&#xf4; ph&#x1ecf;ng si&#xea;u ch&#xe2;n th&#x1ef1;c!.."
        ),
        (
            "HTML entities with author",
            "M&#xf4; ph&#x1ecf;ng si&#xea;u ch&#xe2;n th&#x1ef1;c!.. | By Haizaizhu Gaming"
        ),
        (
            "Complex HTML entities",
            "2.6M views &#xb7; 9.9K reactions | M&#xf4; ph&#x1ecf;ng si&#xea;u ch&#xe2;n th&#x1ef1;c!.. | By Haizaizhu Gaming"
        ),
        (
            "Decimal HTML entities",
            "Test &#183; with &#244; decimal &#7887; entities"
        ),
    ];
    
    for (description, raw_title) in test_cases {
        println!("\n📝 Test: {}", description);
        println!("   Raw: {}", raw_title);
        
        // Test the parse_facebook_title method
        let cleaned_title = test_parse_title(&extractor, raw_title);
        println!("   ✅ Cleaned: {}", cleaned_title);
        
        // Test Unicode decoding specifically
        let decoded = test_unicode_decode(&extractor, raw_title);
        println!("   🔤 Unicode decoded: {}", decoded);

        // Test HTML entity decoding specifically
        let html_decoded = test_html_entity_decode(raw_title);
        println!("   🌐 HTML entities decoded: {}", html_decoded);
    }
    
    println!("\n🎯 Testing HTML title extraction");
    println!("=================================");
    
    // Test with mock HTML that contains the title tag
    let mock_html = r#"
    <html>
    <head>
        <title>Dắt chó đi dạo nhưng ko xích, ko rọ mõm | Dắt chó đi dạo nhưng ko xích, ko rọ mõm | By Al Khaleel Studios</title>
    </head>
    </html>
    "#;
    
    let extracted_title = extractor.extract_title_from_html(mock_html, "test123");
    println!("📄 HTML title extraction result: {}", extracted_title);
}

// Helper function to test the private parse_facebook_title method
fn test_parse_title(extractor: &MetadataExtractor, raw_title: &str) -> String {
    // We need to access the private method, so we'll simulate it here
    // This is a simplified version of the logic

    // First decode HTML entities and Unicode escapes
    let decoded = decode_html_entities(raw_title);
    let decoded = decode_unicode_escapes(&decoded);
    
    // Remove common Facebook suffixes
    let title = decoded
        .replace(" - Facebook", "")
        .replace(" | Facebook", "")
        .trim()
        .to_string();

    // Handle Facebook's common title format: "Title | Title | By Author"
    if title.contains(" | By ") {
        if let Some(clean_title) = title.split(" | By ").next() {
            let clean_title = clean_title.trim();
            
            // If the title is duplicated, extract just the first occurrence
            if clean_title.contains(" | ") {
                let parts: Vec<&str> = clean_title.split(" | ").collect();
                if parts.len() >= 2 && parts[0] == parts[1] {
                    return clean_title_text(parts[0]);
                }
            }
            
            return clean_title_text(clean_title);
        }
    }

    // Handle other common patterns
    if title.contains(" | ") {
        let parts: Vec<&str> = title.split(" | ").collect();
        if parts.len() >= 2 {
            // If first two parts are identical, it's likely a duplicated title
            if parts[0] == parts[1] {
                return clean_title_text(parts[0]);
            }
            // Otherwise, take the first part as the title
            return clean_title_text(parts[0]);
        }
    }

    clean_title_text(&title)
}

fn test_unicode_decode(extractor: &MetadataExtractor, text: &str) -> String {
    decode_unicode_escapes(text)
}

fn test_html_entity_decode(text: &str) -> String {
    decode_html_entities(text)
}

fn decode_html_entities(text: &str) -> String {
    use regex::Regex;

    let mut result = text.to_string();

    println!("🔍 Decoding HTML entities in: {}", text);

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
                    println!("   Replacing {} with {}", full_match, unicode_char);
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
                    println!("   Replacing {} with {}", full_match, unicode_char);
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

    println!("   Final result: {}", result);
    result
}

fn decode_unicode_escapes(text: &str) -> String {
    use regex::Regex;

    let mut result = text.to_string();

    println!("🔍 Decoding Unicode escapes in: {}", text);

    // Handle Unicode escape sequences like \u00e1 -> á (with backslash)
    if let Ok(regex) = Regex::new(r"\\u([0-9a-fA-F]{4})") {
        while let Some(capture) = regex.captures(&result) {
            if let Some(hex_match) = capture.get(1) {
                if let Ok(code_point) = u32::from_str_radix(hex_match.as_str(), 16) {
                    if let Some(unicode_char) = char::from_u32(code_point) {
                        let full_match = capture.get(0).unwrap().as_str();
                        println!("   Replacing {} with {}", full_match, unicode_char);
                        result = result.replace(full_match, &unicode_char.to_string());
                    }
                }
            }
        }
    }

    // Handle Unicode escape sequences like u00e1 -> á (without backslash, common in Facebook JSON)
    // Be more specific to avoid false matches
    if let Ok(regex) = Regex::new(r"\bu([0-9a-fA-F]{4})\b") {
        let matches: Vec<_> = regex.captures_iter(&result).map(|cap| {
            (cap.get(0).unwrap().as_str().to_string(), cap.get(1).unwrap().as_str().to_string())
        }).collect();
        println!("   Found {} potential Unicode escapes without backslash", matches.len());

        for (full_match, hex_str) in matches {
            if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                if let Some(unicode_char) = char::from_u32(code_point) {
                    println!("   Replacing {} with {}", full_match, unicode_char);
                    result = result.replace(&full_match, &unicode_char.to_string());
                }
            }
        }
    }

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
        println!("   Found {} letter-u-hex-letter patterns", matches.len());

        for (full_match, before, hex_str, after) in matches {
            if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                if let Some(unicode_char) = char::from_u32(code_point) {
                    let replacement = format!("{}{}{}", before, unicode_char, after);
                    println!("   Replacing {} with {}", full_match, replacement);
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
        println!("   Found {} letter-u-hex-space patterns", matches.len());

        for (full_match, before, hex_str, after) in matches {
            if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                if let Some(unicode_char) = char::from_u32(code_point) {
                    let replacement = format!("{}{}{}", before, unicode_char, after);
                    println!("   Replacing {} with {}", full_match, replacement);
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
        println!("   Found {} space-u-hex-letter patterns", matches.len());

        for (full_match, before, hex_str, after) in matches {
            if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                if let Some(unicode_char) = char::from_u32(code_point) {
                    let replacement = format!("{}{}{}", before, unicode_char, after);
                    println!("   Replacing {} with {}", full_match, replacement);
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

    println!("   Final result: {}", result);
    result
}

fn clean_title_text(title: &str) -> String {
    use regex::Regex;

    let mut cleaned = title.trim().to_string();

    // Remove view and reaction counts from the beginning with enhanced patterns
    // Handles formats like: "2.6M views · 9.9K reactions | "
    let view_reaction_patterns = vec![
        // Pattern with decoded entities (after HTML entity decoding)
        r"^[\d.]+[KMB]?\s+views\s*[·•]\s*[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
        // More flexible pattern
        r"^[\d.]+[KMB]?\s+views\s*[^\w]*\s*[\d.]+[KMB]?\s+reactions?\s*[|·•]\s*",
    ];

    for pattern in view_reaction_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&cleaned) {
                cleaned = regex.replace(&cleaned, "").to_string();
                break;
            }
        }
    }

    // Remove hashtags and everything after them
    if let Some(hashtag_pos) = cleaned.find(" #") {
        cleaned = cleaned[..hashtag_pos].trim().to_string();
    }

    // Remove emoji sequences at the end
    if let Ok(regex) = Regex::new(r"\s*[\u{1F600}-\u{1F64F}\u{1F300}-\u{1F5FF}\u{1F680}-\u{1F6FF}\u{1F1E0}-\u{1F1FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]+\s*$") {
        cleaned = regex.replace_all(&cleaned, "").to_string();
    }

    cleaned.trim().to_string()
}
