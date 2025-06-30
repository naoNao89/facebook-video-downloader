//! Extraction command implementations
//!
//! This module contains CLI command implementations for testing and debugging
//! the video extraction functionality.

use facebook_video_downloader_core::{FacebookExtractor, ExtractorConfig};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "extraction")]
#[command(about = "Video extraction commands")]
pub struct ExtractionCommand {
    #[command(subcommand)]
    pub command: ExtractionSubcommands,
}

#[derive(Subcommand)]
pub enum ExtractionSubcommands {
    /// Extract video information from a URL
    Extract {
        /// Facebook video URL to extract
        #[arg(short, long)]
        url: String,
        
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
        
        /// Output format (json, pretty)
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
    
    /// Test extraction with multiple URLs
    Batch {
        /// File containing URLs (one per line)
        #[arg(short, long)]
        file: String,
        
        /// Maximum concurrent extractions
        #[arg(short, long, default_value = "3")]
        concurrent: usize,
    },
}

pub async fn handle_extraction_command(cmd: ExtractionCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.command {
        ExtractionSubcommands::Extract { url, verbose, format } => {
            extract_single(url, verbose, format).await
        }
        ExtractionSubcommands::Batch { file, concurrent } => {
            extract_batch(file, concurrent).await
        }
    }
}

async fn extract_single(url: String, verbose: bool, format: String) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        tracing_subscriber::fmt::init();
    }
    
    println!("🎬 Extracting video information");
    println!("===============================");
    println!("URL: {}", url);
    println!("Format: {}", format);
    println!();
    
    let config = ExtractorConfig::default();
    let extractor = FacebookExtractor::new(config);
    
    match extractor.extract(&url).await {
        Ok(video_info) => {
            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&video_info)?;
                    println!("{}", json);
                }
                "pretty" | _ => {
                    println!("✅ Extraction successful!");
                    println!();
                    println!("📹 Video Information:");
                    println!("  Title: {}", video_info.title);
                    println!("  Author: {}", video_info.author);
                    println!("  Duration: {}", video_info.duration);
                    println!("  Video ID: {}", video_info.video_id);
                    println!("  Content Type: {:?}", video_info.content_type);
                    println!("  Privacy Level: {:?}", video_info.privacy_level);
                    println!("  Access Method: {:?}", video_info.access_method);
                    
                    if !video_info.qualities.is_empty() {
                        println!();
                        println!("🎥 Available Qualities:");
                        for (i, quality) in video_info.qualities.iter().enumerate() {
                            println!("  {}. {} - {} ({})", 
                                i + 1, 
                                quality.label, 
                                quality.resolution, 
                                quality.file_size_mb.map_or("Unknown size".to_string(), |s| format!("{:.1} MB", s))
                            );
                        }
                    }
                    
                    if !video_info.thumbnails.variants.is_empty() {
                        println!();
                        println!("🖼️ Thumbnails: {} variants available", video_info.thumbnails.variants.len());
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Extraction failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn extract_batch(file: String, concurrent: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Batch extraction");
    println!("===================");
    println!("File: {}", file);
    println!("Concurrent: {}", concurrent);
    println!();
    
    // Read URLs from file
    let content = std::fs::read_to_string(&file)?;
    let urls: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    
    if urls.is_empty() {
        println!("❌ No valid URLs found in file");
        return Ok(());
    }
    
    println!("📋 Found {} URLs to process", urls.len());
    
    let config = ExtractorConfig::default();
    let extractor = FacebookExtractor::new(config);
    
    let mut successful = 0;
    let mut failed = 0;
    
    // Process URLs in batches
    for chunk in urls.chunks(concurrent) {
        let mut handles = Vec::new();
        
        for url in chunk {
            let extractor_clone = extractor.clone();
            let url_clone = url.clone();
            
            let handle = tokio::spawn(async move {
                match extractor_clone.extract(&url_clone).await {
                    Ok(video_info) => {
                        println!("✅ {}: {}", url_clone, video_info.title);
                        true
                    }
                    Err(e) => {
                        println!("❌ {}: {}", url_clone, e);
                        false
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all handles in this batch
        for handle in handles {
            if let Ok(success) = handle.await {
                if success {
                    successful += 1;
                } else {
                    failed += 1;
                }
            } else {
                failed += 1;
            }
        }
    }
    
    println!();
    println!("📊 Batch Results:");
    println!("  Total: {}", urls.len());
    println!("  Successful: {} ({:.1}%)", successful, (successful as f64 / urls.len() as f64) * 100.0);
    println!("  Failed: {} ({:.1}%)", failed, (failed as f64 / urls.len() as f64) * 100.0);
    
    Ok(())
}
