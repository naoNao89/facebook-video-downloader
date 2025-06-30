//! # Thumbnail Fallback Fix Test
//!
//! ## Purpose
//! Tests fallback mechanisms when primary thumbnail extraction fails
//!
//! ## Category
//! Thumbnail Testing
//!
//! ## Usage
//! ```bash
//! cargo run --bin test_thumbnail_fallback_fix
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

use facebook_extractor_core::FacebookExtractor;
use std::fs;
use base64::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Facebook Thumbnail Fallback Fix Test");
    println!("========================================");
    
    let test_url = "https://www.facebook.com/watch/?v=900186485344685";
    println!("🔍 Test URL: {}", test_url);
    
    println!("\n📋 Step 1: Current behavior (returns CDN URLs)");
    println!("----------------------------------------------");
    
    let extractor = FacebookExtractor::new()?;
    let video_info = extractor.extract_video_info(test_url).await?;
    
    println!("✅ Video extracted: {}", video_info.title);
    println!("🖼️ Current thumbnail: {}", &video_info.thumbnail[..100.min(video_info.thumbnail.len())]);
    
    if video_info.thumbnail.starts_with("https://scontent") {
        println!("❌ ISSUE: Extractor returns CDN URL that will get 403 Forbidden");
        
        println!("\n📋 Step 2: Create placeholder fallback");
        println!("--------------------------------------");
        
        let placeholder_svg = create_placeholder_thumbnail(&video_info.title, &video_info.video_id);
        println!("✅ Created placeholder SVG ({} chars)", placeholder_svg.len());
        
        // Save the placeholder for review
        if let Some(base64_part) = placeholder_svg.strip_prefix("data:image/svg+xml;base64,") {
            if let Ok(svg_bytes) = base64::engine::general_purpose::STANDARD.decode(base64_part) {
                fs::write("placeholder_thumbnail.svg", &svg_bytes)?;
                println!("💾 Saved placeholder to: placeholder_thumbnail.svg");
            }
        }
        
        // Create test HTML to show the fix
        create_fix_demo_html(&video_info.thumbnail, &placeholder_svg).await?;
        println!("💾 Created fix demo: thumbnail_fix_demo.html");
        
        println!("\n📋 Step 3: Proposed solution");
        println!("----------------------------");
        println!("🔧 SOLUTION: Modify the core extractor to:");
        println!("   1. Detect Facebook CDN URLs (scontent.*.fbcdn.net)");
        println!("   2. Attempt to fetch with proper headers");
        println!("   3. If 403 Forbidden, return placeholder data URL instead");
        println!("   4. This will make thumbnails work in the Tauri app");
        
        println!("\n📊 Implementation needed:");
        println!("   - Add thumbnail validation to FacebookExtractor");
        println!("   - Add fallback logic for failed CDN requests");
        println!("   - Return data URLs instead of raw CDN URLs");
        
    } else if video_info.thumbnail.starts_with("data:") {
        println!("✅ GOOD: Extractor already returns data URL");
        println!("   This should work in the Tauri app");
    } else {
        println!("❓ UNKNOWN: Unexpected thumbnail format");
    }
    
    Ok(())
}

fn create_placeholder_thumbnail(title: &str, video_id: &str) -> String {
    let truncated_title = if title.len() > 50 {
        format!("{}...", &title[..47])
    } else {
        title.to_string()
    };
    
    let svg_content = format!(
        "<svg width=\"320\" height=\"180\" xmlns=\"http://www.w3.org/2000/svg\">\
        <rect width=\"320\" height=\"180\" fill=\"#4267B2\" />\
        <circle cx=\"160\" cy=\"90\" r=\"30\" fill=\"white\" opacity=\"0.9\"/>\
        <polygon points=\"150,75 150,105 175,90\" fill=\"#4267B2\"/>\
        <text x=\"160\" y=\"130\" text-anchor=\"middle\" font-family=\"Arial\" font-size=\"12\" fill=\"white\" opacity=\"0.9\">{}</text>\
        <text x=\"160\" y=\"150\" text-anchor=\"middle\" font-family=\"Arial\" font-size=\"10\" fill=\"white\" opacity=\"0.7\">Video ID: {}</text>\
        <text x=\"160\" y=\"170\" text-anchor=\"middle\" font-family=\"Arial\" font-size=\"8\" fill=\"white\" opacity=\"0.5\">Facebook Video</text>\
        </svg>",
        html_escape(&truncated_title),
        video_id
    );
    
    let base64_svg = base64::engine::general_purpose::STANDARD.encode(svg_content.as_bytes());
    format!("data:image/svg+xml;base64,{}", base64_svg)
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

async fn create_fix_demo_html(original_url: &str, placeholder_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let html_content = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Facebook Thumbnail Fix Demo</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .comparison {{ display: flex; gap: 20px; margin: 20px 0; }}
        .thumbnail-demo {{ flex: 1; padding: 20px; border: 1px solid #ddd; border-radius: 8px; }}
        .thumbnail-demo img {{ max-width: 100%; border: 1px solid #ccc; }}
        .broken {{ border-left: 4px solid #f44336; }}
        .fixed {{ border-left: 4px solid #4CAF50; }}
        .url {{ word-break: break-all; font-family: monospace; background: #f5f5f5; padding: 10px; font-size: 12px; }}
        .status {{ font-weight: bold; margin: 10px 0; }}
        .status.error {{ color: #f44336; }}
        .status.success {{ color: #4CAF50; }}
    </style>
</head>
<body>
    <h1>🔧 Facebook Thumbnail Fix Demo</h1>
    <p>This demonstrates the issue with Facebook CDN URLs and the proposed fix.</p>
    
    <div class="comparison">
        <div class="thumbnail-demo broken">
            <h3>❌ Current Issue (CDN URL)</h3>
            <div class="status error">403 Forbidden - Cannot load</div>
            <img src="{}" alt="Facebook CDN thumbnail" onerror="this.style.display='none'; this.nextElementSibling.style.display='block';">
            <p style="display:none; color: red; text-align: center; padding: 20px; border: 2px dashed #f44336;">
                ❌ Failed to load Facebook CDN image<br>
                (This is the current issue in Tauri app)
            </p>
            <h4>URL:</h4>
            <div class="url">{}</div>
        </div>
        
        <div class="thumbnail-demo fixed">
            <h3>✅ Proposed Fix (Placeholder)</h3>
            <div class="status success">Data URL - Always works</div>
            <img src="{}" alt="Placeholder thumbnail">
            <h4>Data URL (first 200 chars):</h4>
            <div class="url">{}</div>
        </div>
    </div>
    
    <h2>📋 Implementation Plan</h2>
    <ol>
        <li><strong>Detect CDN URLs:</strong> Check if thumbnail starts with "https://scontent"</li>
        <li><strong>Attempt download:</strong> Try to fetch with Facebook headers</li>
        <li><strong>Handle 403:</strong> If forbidden, generate placeholder data URL</li>
        <li><strong>Return data URL:</strong> Always return data URLs instead of CDN URLs</li>
    </ol>
    
    <h2>🎯 Benefits</h2>
    <ul>
        <li>✅ Thumbnails always display in Tauri app</li>
        <li>✅ No CORS issues with data URLs</li>
        <li>✅ Graceful fallback for blocked content</li>
        <li>✅ Better user experience</li>
    </ul>
    
    <h2>📁 Files to Modify</h2>
    <ul>
        <li><code>crates/facebook-extractor-core/src/metadata.rs</code> - Add thumbnail validation</li>
        <li><code>crates/facebook-extractor-core/src/extractor.rs</code> - Add fallback logic</li>
    </ul>
</body>
</html>"#, 
        original_url,
        &original_url[..100.min(original_url.len())],
        placeholder_url,
        &placeholder_url[..200.min(placeholder_url.len())]
    );

    fs::write("thumbnail_fix_demo.html", html_content)?;
    Ok(())
}
