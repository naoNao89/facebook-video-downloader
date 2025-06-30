use facebook_extractor_core::{FacebookExtractor, DisplayContext, AspectRatio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🧪 Testing Enhanced Thumbnail Extraction");
    println!("========================================");
    
    let test_url = "https://www.facebook.com/share/v/16b8W3F2tG/";
    println!("🎯 Testing URL: {}", test_url);
    
    let extractor = FacebookExtractor::new()?;
    
    match extractor.extract_video_info(test_url).await {
        Ok(video_info) => {
            println!("\n✅ Video extraction successful!");
            println!("📝 Title: {}", video_info.title);
            println!("🆔 Video ID: {}", video_info.video_id);
            
            // Test enhanced thumbnail functionality
            println!("\n🖼️ Enhanced Thumbnail Analysis");
            println!("==============================");
            
            // Check legacy thumbnail
            println!("📄 Legacy thumbnail (backward compatibility):");
            if video_info.thumbnail.starts_with("data:") {
                println!("   ✅ Legacy thumbnail is a data URL ({} chars)", video_info.thumbnail.len());
            } else {
                println!("   ❌ Legacy thumbnail is not a data URL: {}", &video_info.thumbnail[..100.min(video_info.thumbnail.len())]);
            }
            
            // Check thumbnail collection
            println!("\n📊 Thumbnail Collection Analysis:");
            println!("   Original video aspect ratio: {:?}", video_info.thumbnail_variants.original_aspect_ratio);
            println!("   Number of variants: {}", video_info.thumbnail_variants.variants.len());
            
            if video_info.thumbnail_variants.variants.is_empty() {
                println!("   ❌ No thumbnail variants found");
                return Ok(());
            }
            
            // Display variant summary
            let summary = video_info.thumbnail_variants.get_summary();
            println!("   Variant sizes available:");
            for (size, count) in summary {
                println!("     - {}: {} variant(s)", size, count);
            }
            
            // Test different display contexts
            println!("\n🎯 Display Context Testing:");
            let contexts = vec![
                DisplayContext::DesktopFullscreen,
                DisplayContext::MobilePortrait,
                DisplayContext::MobileLandscape,
                DisplayContext::WebThumbnail,
                DisplayContext::SocialShare,
                DisplayContext::PlayerPreview,
            ];
            
            for context in contexts {
                match video_info.get_thumbnail_for_context(context) {
                    Some(variant) => {
                        println!("   ✅ {:?}: {}x{} ({} format, {} bytes, quality: {})",
                            context,
                            variant.width,
                            variant.height,
                            variant.format,
                            variant.file_size,
                            variant.quality_score
                        );
                    }
                    None => {
                        println!("   ❌ {:?}: No suitable variant found", context);
                    }
                }
            }
            
            // Test aspect ratio detection
            println!("\n📐 Aspect Ratio Analysis:");
            let detected_ratio = video_info.detect_video_aspect_ratio();
            println!("   Detected video aspect ratio: {:?} (ratio: {:.2})", detected_ratio, detected_ratio.ratio());
            
            if detected_ratio.is_landscape() {
                println!("   📺 Video orientation: Landscape");
            } else if detected_ratio.is_portrait() {
                println!("   📱 Video orientation: Portrait");
            } else {
                println!("   ⬜ Video orientation: Square");
            }
            
            // Test available aspect ratios in thumbnails
            let available_ratios = video_info.thumbnail_variants.available_aspect_ratios();
            println!("   Available thumbnail aspect ratios:");
            for ratio in available_ratios {
                let variants = video_info.thumbnail_variants.get_variants_by_aspect_ratio(ratio);
                println!("     - {:?}: {} variant(s)", ratio, variants.len());
            }
            
            // Test responsive thumbnail selection
            println!("\n📱 Responsive Thumbnail Selection:");
            
            // Desktop scenario
            if let Some(desktop_variant) = video_info.get_thumbnail_for_context(DisplayContext::DesktopFullscreen) {
                println!("   🖥️ Desktop fullscreen: {}x{} ({})", 
                    desktop_variant.width, desktop_variant.height, desktop_variant.format);
            }
            
            // Mobile scenarios
            if let Some(mobile_portrait) = video_info.get_thumbnail_for_context(DisplayContext::MobilePortrait) {
                println!("   📱 Mobile portrait: {}x{} ({})", 
                    mobile_portrait.width, mobile_portrait.height, mobile_portrait.format);
            }
            
            if let Some(mobile_landscape) = video_info.get_thumbnail_for_context(DisplayContext::MobileLandscape) {
                println!("   📱 Mobile landscape: {}x{} ({})", 
                    mobile_landscape.width, mobile_landscape.height, mobile_landscape.format);
            }
            
            // Web thumbnail
            if let Some(web_thumb) = video_info.get_thumbnail_for_context(DisplayContext::WebThumbnail) {
                println!("   🌐 Web thumbnail: {}x{} ({})", 
                    web_thumb.width, web_thumb.height, web_thumb.format);
            }
            
            // Test best thumbnail selection
            println!("\n🏆 Best Thumbnail Selection:");
            if let Some(best_variant) = video_info.get_best_thumbnail() {
                println!("   Best overall: {}x{} ({} format, quality score: {})",
                    best_variant.width,
                    best_variant.height,
                    best_variant.format,
                    best_variant.quality_score
                );
                
                println!("   Optimized for contexts: {:?}", best_variant.optimized_for);
            }
            
            // Test URL retrieval for different contexts
            println!("\n🔗 URL Retrieval Testing:");
            for context in [DisplayContext::DesktopFullscreen, DisplayContext::MobilePortrait, DisplayContext::WebThumbnail] {
                let url = video_info.get_thumbnail_url_for_context(context);
                if url.starts_with("data:") {
                    println!("   {:?}: Data URL ({} chars)", context, url.len());
                } else {
                    println!("   {:?}: {}", context, &url[..100.min(url.len())]);
                }
            }
            
            println!("\n✅ Enhanced thumbnail extraction test completed successfully!");
            
        }
        Err(e) => {
            println!("❌ Video extraction failed: {}", e);
        }
    }
    
    Ok(())
}
