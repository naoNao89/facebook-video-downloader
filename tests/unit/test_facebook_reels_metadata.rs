use facebook_extractor_core::extractor::FacebookExtractor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🧪 Testing Facebook Reels Metadata Extraction");
    println!("==============================================");

    // Test both problematic URLs mentioned by the user
    let test_urls = vec![
        (
            "https://www.facebook.com/reel/4037284366546945",
            "Hôm ai đòi rủ thầy solo? #thichkhaiquang #thichkhaiquang89",
            "Showbiz Có Gì Vui?",
            1800000,
            24000
        ),
        (
            "https://www.facebook.com/reel/2133670053772008",
            "Startup Mỹ ra mắt công nghệ biến không khí thành xăng ngay tại chỗ",
            "Vietstock",
            0, // Views not specified
            11200 // 11.2k likes
        ),
    ];

    for (test_url, expected_title, expected_author, expected_views, expected_likes) in test_urls {
        println!("\n🎯 Testing URL: {}", test_url);
        println!("📝 Expected results:");
        println!("   Title: {}", expected_title);
        println!("   Author: {}", expected_author);
        if expected_views > 0 {
            println!("   Views: {}", expected_views);
        }
        if expected_likes > 0 {
            println!("   Likes: {}", expected_likes);
        }

        let extractor = FacebookExtractor::new()?;

        match extractor.extract_video_info(test_url).await {
        Ok(video_info) => {
            println!("\n✅ Extraction successful!");
            println!("📊 Video Information:");
            println!("   Title: {}", video_info.title);
            println!("   Author: {}", video_info.metadata.author);
            println!("   Views: {}", video_info.metadata.views);
            println!("   Likes: {}", video_info.metadata.likes);
            println!("   Comments: {}", video_info.metadata.comments);
            println!("   Shares: {}", video_info.metadata.shares);
            println!("   Duration: {} seconds", video_info.metadata.duration_seconds.unwrap_or(0));
            println!("   Hashtags: {:?}", video_info.metadata.hashtags);
            
            // Test specific expectations
            println!("\n🔍 Validation:");
            
            // Title validation
            if video_info.title.contains(expected_title.split_whitespace().next().unwrap_or("")) {
                println!("   ✅ Title correctly extracted: {}", video_info.title);
            } else {
                println!("   ❌ Title not properly extracted: {}", video_info.title);
                println!("       Expected: {}", expected_title);
            }

            // Author validation
            if video_info.metadata.author.contains(expected_author) {
                println!("   ✅ Author correctly extracted: {}", video_info.metadata.author);
            } else {
                println!("   ❌ Author not correctly extracted: {}", video_info.metadata.author);
                println!("       Expected: {}", expected_author);
                println!("       Got: {}", video_info.metadata.author);
            }
            
            // Views validation (if expected)
            if expected_views > 0 {
                if video_info.metadata.views >= (expected_views as f64 * 0.9) as u64 &&
                   video_info.metadata.views <= (expected_views as f64 * 1.1) as u64 {
                    println!("   ✅ Views correctly extracted: {}", video_info.metadata.views);
                } else {
                    println!("   ❌ Views not in expected range: {}", video_info.metadata.views);
                    println!("       Expected: ~{}", expected_views);
                }
            }

            // Likes validation (if expected)
            if expected_likes > 0 {
                if video_info.metadata.likes >= (expected_likes as f64 * 0.8) as u64 {
                    println!("   ✅ Likes extracted: {}", video_info.metadata.likes);
                } else {
                    println!("   ❌ Likes not properly extracted: {}", video_info.metadata.likes);
                    println!("       Expected: ~{}", expected_likes);
                }
            } else if video_info.metadata.likes > 0 {
                println!("   ✅ Likes extracted: {}", video_info.metadata.likes);
            }
            
            // Comments should be reasonable
            if video_info.metadata.comments > 100 {
                println!("   ✅ Comments extracted: {}", video_info.metadata.comments);
            } else {
                println!("   ❌ Comments not properly extracted: {}", video_info.metadata.comments);
            }
            
            // Shares should be extracted
            if video_info.metadata.shares > 0 {
                println!("   ✅ Shares extracted: {}", video_info.metadata.shares);
            } else {
                println!("   ❌ Shares not extracted: {}", video_info.metadata.shares);
            }
            
            // Duration should be around 61 seconds
            if let Some(duration) = video_info.metadata.duration_seconds {
                if duration >= 55 && duration <= 70 {
                    println!("   ✅ Duration extracted: {} seconds", duration);
                } else {
                    println!("   ❌ Duration not in expected range: {} seconds", duration);
                }
            } else {
                println!("   ❌ Duration not extracted");
            }
            
            // Hashtags should include gaming-related tags
            if !video_info.metadata.hashtags.is_empty() {
                println!("   ✅ Hashtags extracted: {:?}", video_info.metadata.hashtags);
            } else {
                println!("   ❌ No hashtags extracted");
            }
        }
        Err(e) => {
            println!("❌ Extraction failed: {}", e);
        }
    }
    }

    Ok(())
}
