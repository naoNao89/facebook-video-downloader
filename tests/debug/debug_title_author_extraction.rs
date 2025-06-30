use facebook_extractor_core::extractor::FacebookExtractor;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🔍 Debug Title and Author Extraction for Facebook Reels");
    println!("======================================================");

    let test_url = "https://www.facebook.com/share/v/16fmUF2rfh/";
    println!("🎯 Testing URL: {}", test_url);

    // Use the FacebookExtractor to get the HTML content (it has better headers and handling)
    println!("\n📡 Fetching HTML content using FacebookExtractor...");
    let extractor = FacebookExtractor::new()?;

    // Try to extract video info to get the HTML content
    match extractor.extract_video_info(test_url).await {
        Ok(video_info) => {
            println!("✅ Video extraction successful");
            println!("Current Title: {}", video_info.title);
            println!("Current Author: {}", video_info.metadata.author);
        }
        Err(e) => {
            println!("❌ Video extraction failed: {}", e);
        }
    }

    // Since we can't easily get the raw HTML from the extractor, let's try a different approach
    // Let's use the mobile version which sometimes works better
    println!("\n📡 Trying mobile version...");
    let mobile_url = test_url.replace("www.facebook.com", "m.facebook.com");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let response = client
        .get(&mobile_url)
        .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .send()
        .await?;

    if !response.status().is_success() {
        println!("❌ Failed to fetch mobile HTML: {}", response.status());
        return Ok(());
    }

    let html = response.text().await?;
    println!("✅ HTML fetched successfully ({} characters)", html.len());

    // Debug title extraction
    println!("\n🔍 TITLE EXTRACTION DEBUG");
    println!("========================");
    
    debug_title_patterns(&html);
    
    // Debug author extraction  
    println!("\n👤 AUTHOR EXTRACTION DEBUG");
    println!("==========================");
    
    debug_author_patterns(&html);
    
    // Debug complete - no need to search for specific expected content

    Ok(())
}

fn debug_title_patterns(html: &str) {
    let title_patterns = vec![
        ("HTML title tag", r#"<title>([^<]+)</title>"#),
        ("OG title", r#"<meta property="og:title" content="([^"]+)""#),
        ("Twitter title", r#"<meta name="twitter:title" content="([^"]+)""#),
        ("JSON title", r#""title":"([^"]+)""#),
        ("JSON text", r#""text":"([^"]+)","ranges":\[\]"#),
        ("JSON message", r#""message":"([^"]+)""#),
        ("JSON story", r#""story":"([^"]+)""#),
        ("Video title", r#""video_title":"([^"]+)""#),
        ("Content title", r#""content_title":"([^"]+)""#),
        ("Post title", r#""post_title":"([^"]+)""#),
    ];

    for (name, pattern) in title_patterns {
        println!("\n🔍 Pattern: {}", name);
        println!("   Regex: {}", pattern);
        
        if let Ok(regex) = Regex::new(pattern) {
            let matches: Vec<_> = regex.captures_iter(html).collect();
            println!("   Matches: {}", matches.len());
            
            for (i, capture) in matches.iter().take(5).enumerate() {
                if let Some(title_match) = capture.get(1) {
                    let title = title_match.as_str();
                    println!("   {}. {}", i + 1, &title[..200.min(title.len())]);
                }
            }
        } else {
            println!("   ❌ Invalid regex");
        }
    }
}

fn debug_author_patterns(html: &str) {
    let author_patterns = vec![
        ("OG title with By", r#"<meta property="og:title" content="[^"]*\| By ([^"]+)""#),
        ("Twitter title with By", r#"<meta name="twitter:title" content="[^"]*\| By ([^"]+)""#),
        ("JSON name with profile", r#""name":"([^"]+)","short_name":"[^"]*","id":"[0-9]+","profile_picture"#),
        ("Page name", r#""page_name":"([^"]+)""#),
        ("Actor name", r#""actor":\{"name":"([^"]+)""#),
        ("Owner name", r#""owner":\{"name":"([^"]+)""#),
        ("Profile name", r#""profile_name":"([^"]+)""#),
        ("Display name", r#""display_name":"([^"]+)""#),
        ("Creator name", r#""creator":\{"name":"([^"]+)""#),
        ("Author Person", r#""@type":"Person","name":"([^"]+)""#),
        ("Author Organization", r#""@type":"Organization","name":"([^"]+)""#),
        ("Standard name", r#""name":"([^"]+)""#),
    ];

    for (name, pattern) in author_patterns {
        println!("\n👤 Pattern: {}", name);
        println!("   Regex: {}", pattern);
        
        if let Ok(regex) = Regex::new(pattern) {
            let matches: Vec<_> = regex.captures_iter(html).collect();
            println!("   Matches: {}", matches.len());
            
            for (i, capture) in matches.iter().take(5).enumerate() {
                if let Some(author_match) = capture.get(1) {
                    let author = author_match.as_str();
                    println!("   {}. {}", i + 1, &author[..100.min(author.len())]);
                }
            }
        } else {
            println!("   ❌ Invalid regex");
        }
    }
}


