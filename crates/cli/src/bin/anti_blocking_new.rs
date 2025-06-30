//! New CLI tool for anti-blocking functionality including real IPv6 implementation
//!
//! This tool provides access to both simulation and real IPv6 modes with proper
//! user consent and safety measures.

use facebook_video_downloader_cli::commands::anti_blocking::{AntiBlockingCommand, handle_anti_blocking_command};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let cmd = AntiBlockingCommand::parse();
    handle_anti_blocking_command(cmd).await?;
    
    Ok(())
}
