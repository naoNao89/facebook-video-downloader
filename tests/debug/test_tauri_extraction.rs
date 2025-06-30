use facebook_extractor_core::FacebookExtractor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🔍 Testing Tauri Extraction Path");
    println!("================================");
    
    // Test both problematic URLs
    let test_urls = vec![
        (
            "https://www.facebook.com/reel/4037284366546945",
            "Hôm ai đòi rủ thầy solo?",
            "Showbiz Có Gì Vui?"
        ),
        (
            "https://www.facebook.com/reel/2133670053772008",
            "Startup Mỹ ra mắt công nghệ biến không khí thành xăng",
            "Vietstock"
        ),
    ];
    
    for (test_url, expected_title_part, expected_author_part) in test_urls {
        println!("\n🎯 Testing URL: {}", test_url);
        println!("📝 Expected title to contain: {}", expected_title_part);
        println!("📝 Expected author to contain: {}", expected_author_part);
        
        // Use the same extractor that Tauri uses
        let extractor = FacebookExtractor::new()?;
        
        match extractor.extract_video_info(test_url).await {
            Ok(video_info) => {
                println!("✅ Extraction successful!");
                println!("📊 Results:");
                println!("   Title: {}", video_info.title);
                println!("   Author: {}", video_info.metadata.author);
                println!("   Views: {}", video_info.metadata.views);
                println!("   Likes: {}", video_info.metadata.likes);
                println!("   Comments: {}", video_info.metadata.comments);
                println!("   Shares: {}", video_info.metadata.shares);
                println!("   Duration: {} seconds", video_info.duration);
                println!("   Content Type: {:?}", video_info.content_type);
                println!("   Access Method: {:?}", video_info.access_method);
                
                // Validation
                println!("\n🔍 Validation:");
                
                if video_info.title.contains(expected_title_part) {
                    println!("   ✅ Title correctly contains: {}", expected_title_part);
                } else {
                    println!("   ❌ Title does not contain expected text");
                    println!("       Expected: {}", expected_title_part);
                    println!("       Got: {}", video_info.title);
                }
                
                if video_info.metadata.author.contains(expected_author_part) {
                    println!("   ✅ Author correctly contains: {}", expected_author_part);
                } else {
                    println!("   ❌ Author does not contain expected text");
                    println!("       Expected: {}", expected_author_part);
                    println!("       Got: {}", video_info.metadata.author);
                }
                
                // Check if it's using the correct extraction method
                match video_info.access_method {
                    facebook_extractor_core::types::AccessMethod::Mobile => {
                        println!("   ✅ Using mobile extraction (correct for reels)");
                    }
                    other => {
                        println!("   ⚠️ Using {} extraction (may not be optimal for reels)", 
                                format!("{:?}", other));
                    }
                }
            }
            Err(e) => {
                println!("❌ Extraction failed: {}", e);
            }
        }
    }
    
    Ok(())
}
