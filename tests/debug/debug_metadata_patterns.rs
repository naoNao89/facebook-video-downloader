use facebook_extractor_core::extractor::FacebookExtractor;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🔍 Debug Metadata Patterns for Facebook Reels");
    println!("============================================");
    
    let test_url = "https://www.facebook.com/share/v/16b8W3F2tG/";
    println!("🎯 Testing URL: {}", test_url);
    
    let extractor = FacebookExtractor::new()?;
    
    // Get the HTML content using the same headers as FacebookExtractor
    println!("\n📡 Fetching HTML content with proper browser headers...");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let response = client
        .get(test_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-User", "?1")
        .header("Cache-Control", "max-age=0")
        .header("sec-ch-ua", "\"Google Chrome\";v=\"120\", \"Chromium\";v=\"120\", \"Not_A Brand\";v=\"99\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"macOS\"")
        .send()
        .await?;

    let html = response.text().await?;
    
    println!("📄 HTML content length: {} characters", html.len());
    
    // Debug author patterns
    println!("\n👤 AUTHOR PATTERNS DEBUG");
    println!("========================");
    
    // Look for title tag content
    if let Ok(regex) = Regex::new(r#"<title>([^<]+)</title>"#) {
        if let Some(capture) = regex.captures(&html) {
            if let Some(title_match) = capture.get(1) {
                println!("📄 Title tag content: {}", title_match.as_str());
            }
        }
    } else {
        println!("❌ No title tag found");
    }

    // Look for og:title content
    if let Ok(regex) = Regex::new(r#"<meta property="og:title" content="([^"]+)""#) {
        if let Some(capture) = regex.captures(&html) {
            if let Some(title_match) = capture.get(1) {
                println!("🏷️ og:title content: {}", title_match.as_str());
            }
        }
    } else {
        println!("❌ No og:title meta tag found");
    }

    // Look for any meta tags
    if let Ok(regex) = Regex::new(r#"<meta[^>]+>"#) {
        let matches: Vec<_> = regex.find_iter(&html).take(10).collect();
        println!("🏷️ Found {} meta tags (showing first 10):", matches.len());
        for (i, m) in matches.iter().enumerate() {
            println!("   {}: {}", i + 1, m.as_str());
        }
    }
    
    // Look for potential author patterns
    let author_search_patterns = vec![
        "Haizaizhu",
        "Gaming",
        "haizaizhu",
        "gaming",
    ];
    
    for pattern in author_search_patterns {
        let count = html.matches(pattern).count();
        println!("🔍 Found '{}': {} occurrences", pattern, count);
        
        if count > 0 {
            // Show context around matches
            if let Ok(regex) = Regex::new(&format!(r".{{0,50}}{}.{{0,50}}", regex::escape(pattern))) {
                let matches: Vec<_> = regex.find_iter(&html).take(3).collect();
                for (i, m) in matches.iter().enumerate() {
                    println!("   Context {}: {}", i + 1, m.as_str());
                }
            }
        }
    }
    
    // Debug comments patterns
    println!("\n💬 COMMENTS PATTERNS DEBUG");
    println!("==========================");
    
    let comment_search_patterns = vec![
        "comment",
        "Comment",
        "424",
        "400",
        "500",
    ];
    
    for pattern in comment_search_patterns {
        let count = html.matches(pattern).count();
        println!("🔍 Found '{}': {} occurrences", pattern, count);
        
        if count > 0 && count < 20 {
            // Show context around matches for reasonable counts
            if let Ok(regex) = Regex::new(&format!(r".{{0,30}}{}.{{0,30}}", regex::escape(pattern))) {
                let matches: Vec<_> = regex.find_iter(&html).take(5).collect();
                for (i, m) in matches.iter().enumerate() {
                    println!("   Context {}: {}", i + 1, m.as_str());
                }
            }
        }
    }
    
    // Debug shares patterns
    println!("\n🔄 SHARES PATTERNS DEBUG");
    println!("========================");
    
    let share_search_patterns = vec![
        "share",
        "Share",
        "1400",
        "1.4K",
        "1.4k",
        "1,400",
    ];
    
    for pattern in share_search_patterns {
        let count = html.matches(pattern).count();
        println!("🔍 Found '{}': {} occurrences", pattern, count);
        
        if count > 0 && count < 20 {
            // Show context around matches for reasonable counts
            if let Ok(regex) = Regex::new(&format!(r".{{0,30}}{}.{{0,30}}", regex::escape(pattern))) {
                let matches: Vec<_> = regex.find_iter(&html).take(5).collect();
                for (i, m) in matches.iter().enumerate() {
                    println!("   Context {}: {}", i + 1, m.as_str());
                }
            }
        }
    }
    
    // Look for JSON structures that might contain metadata
    println!("\n📋 JSON STRUCTURES DEBUG");
    println!("=========================");
    
    // Look for JSON with engagement data
    if let Ok(regex) = Regex::new(r#"\{"[^}]*"count":\d+[^}]*\}"#) {
        let matches: Vec<_> = regex.find_iter(&html).take(10).collect();
        println!("🔍 Found {} JSON objects with 'count' field:", matches.len());
        for (i, m) in matches.iter().enumerate() {
            println!("   {}: {}", i + 1, m.as_str());
        }
    }
    
    // Look for JSON with name fields
    if let Ok(regex) = Regex::new(r#"\{"[^}]*"name":"[^"]+[^}]*\}"#) {
        let matches: Vec<_> = regex.find_iter(&html).take(10).collect();
        println!("\n🔍 Found {} JSON objects with 'name' field:", matches.len());
        for (i, m) in matches.iter().enumerate() {
            println!("   {}: {}", i + 1, m.as_str());
        }
    }
    
    // Save a sample of the HTML for manual inspection
    println!("\n💾 Saving HTML sample for manual inspection...");
    std::fs::write("facebook_metadata_debug.html", &html)?;
    println!("   Saved complete HTML to facebook_metadata_debug.html");
    
    println!("\n✅ Debug analysis complete!");
    
    Ok(())
}
