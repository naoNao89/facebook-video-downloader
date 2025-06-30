use facebook_extractor_core::FacebookExtractor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing URL Validation and Content Type Detection");
    println!("===================================================");
    
    let test_urls = vec![
        "https://www.facebook.com/reel/4037284366546945",
        "https://www.facebook.com/reel/2133670053772008",
    ];
    
    let extractor = FacebookExtractor::new()?;
    
    for test_url in test_urls {
        println!("\n🎯 Testing URL: {}", test_url);
        
        // Test URL validation
        let validation = extractor.validate_url(test_url);
        println!("📋 Validation Results:");
        println!("   Is Valid: {}", validation.is_valid);
        println!("   Content Type: {:?}", validation.content_type);
        println!("   Video ID: {:?}", validation.video_id);
        println!("   Error: {:?}", validation.error_message);
        
        // Check if it contains "/reel/"
        if test_url.contains("/reel/") {
            println!("   ✅ URL contains '/reel/' - should be detected as reel");
        } else {
            println!("   ❌ URL does not contain '/reel/'");
        }
        
        // Test video ID extraction
        match extractor.extract_video_id(test_url) {
            Ok(video_id) => {
                println!("   ✅ Video ID extracted: {}", video_id);
            }
            Err(e) => {
                println!("   ❌ Video ID extraction failed: {}", e);
            }
        }
    }
    
    Ok(())
}
