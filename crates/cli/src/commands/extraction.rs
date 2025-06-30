//! Extraction command implementations
//!
//! This module contains CLI command implementations for testing and debugging
//! the video extraction functionality.

use facebook_video_downloader_core::FacebookExtractor;
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
}

pub async fn handle_extraction_command(cmd: ExtractionCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.command {
        ExtractionSubcommands::Extract { url, verbose, format } => {
            extract_single(url, verbose, format).await
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
    
    let extractor = FacebookExtractor::new()?;

    match extractor.extract_video_info(&url).await {
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
                    println!("  Author: {}", video_info.metadata.author);
                    println!("  Duration: {}", video_info.duration);
                    println!("  Video ID: {}", video_info.video_id);
                    println!("  Content Type: {:?}", video_info.content_type);
                    println!("  Privacy Level: {:?}", video_info.privacy_level);
                    println!("  Access Method: {:?}", video_info.access_method);
                    
                    if !video_info.qualities.is_empty() {
                        println!();
                        println!("🎥 Available Qualities:");
                        for (i, quality) in video_info.qualities.iter().enumerate() {
                            println!("  {}. {} - {}x{} (~{} MB)",
                                i + 1,
                                quality.quality,
                                quality.width,
                                quality.height,
                                quality.estimated_size_mb
                            );
                        }
                    }
                    
                    if !video_info.thumbnail_variants.variants.is_empty() {
                        println!();
                        println!("🖼️ Thumbnails: {} variants available", video_info.thumbnail_variants.variants.len());
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
