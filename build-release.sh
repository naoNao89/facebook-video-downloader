#!/bin/bash
echo "Building Facebook Video Downloader for release..."
trunk build --release
cargo tauri build
