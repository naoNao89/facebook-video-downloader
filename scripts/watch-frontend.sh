#!/bin/bash

# Frontend File Watcher for Facebook Video Downloader
# Watches Rust/Yew frontend files and triggers WASM rebuilds

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Configuration
CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-8}
WATCH_DIR="src/"
DEBOUNCE_TIME=1  # seconds

log_info() {
    echo -e "${BLUE}[Frontend Watcher] $1${NC}"
}

log_build() {
    echo -e "${PURPLE}[Frontend Watcher] 🔨 $1${NC}"
}

log_success() {
    echo -e "${GREEN}[Frontend Watcher] ✅ $1${NC}"
}

# Build WASM function
build_wasm() {
    log_build "Building WASM frontend..."
    
    # Set environment for fast builds
    export CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS
    export CARGO_INCREMENTAL=1
    
    if CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS cargo build --target wasm32-unknown-unknown --profile dev-fast; then
        if wasm-bindgen --out-dir dist --target web --no-typescript \
            target/wasm32-unknown-unknown/dev-fast/facebook_video_downloader_frontend.wasm; then
            log_success "WASM build completed successfully"
            return 0
        else
            echo -e "${YELLOW}[Frontend Watcher] ⚠️ wasm-bindgen failed${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}[Frontend Watcher] ⚠️ Cargo build failed${NC}"
        return 1
    fi
}

# File watching with different strategies
watch_with_fswatch() {
    log_info "Using fswatch for monitoring $WATCH_DIR"
    fswatch -o -r --event Created --event Updated --event Removed "$WATCH_DIR" | while read f; do
        log_build "Changes detected in frontend files"
        build_wasm
        # Trigger Tauri reload by touching main.rs
        touch src-tauri/src/main.rs 2>/dev/null || true
    done
}

watch_with_inotify() {
    log_info "Using inotifywait for monitoring $WATCH_DIR"
    inotifywait -m -r -e modify,create,delete,move "$WATCH_DIR" --format '%w%f %e' | while read file event; do
        # Only process .rs files
        if [[ "$file" == *.rs ]]; then
            log_build "Change detected: $file ($event)"
            sleep $DEBOUNCE_TIME  # Debounce rapid changes
            build_wasm
            # Trigger Tauri reload
            touch src-tauri/src/main.rs 2>/dev/null || true
        fi
    done
}

watch_with_polling() {
    log_info "Using polling mode for monitoring $WATCH_DIR"
    
    # Create initial checksum
    local last_checksum=""
    
    while true; do
        # Calculate checksum of all .rs files
        local current_checksum=$(find "$WATCH_DIR" -name "*.rs" -type f -exec md5sum {} \; 2>/dev/null | md5sum | cut -d' ' -f1)
        
        if [ "$current_checksum" != "$last_checksum" ] && [ -n "$last_checksum" ]; then
            log_build "Changes detected in frontend files (polling)"
            build_wasm
            # Trigger Tauri reload
            touch src-tauri/src/main.rs 2>/dev/null || true
        fi
        
        last_checksum="$current_checksum"
        sleep 2
    done
}

# Main function
main() {
    log_info "Starting frontend file watcher..."
    
    # Check if watch directory exists
    if [ ! -d "$WATCH_DIR" ]; then
        echo -e "${YELLOW}[Frontend Watcher] ⚠️ Watch directory $WATCH_DIR does not exist${NC}"
        exit 1
    fi
    
    # Perform initial build
    log_info "Performing initial WASM build..."
    build_wasm
    
    # Choose watching strategy based on available tools
    if command -v fswatch >/dev/null 2>&1; then
        watch_with_fswatch
    elif command -v inotifywait >/dev/null 2>&1; then
        watch_with_inotify
    else
        log_info "No native file watcher found, using polling mode"
        watch_with_polling
    fi
}

# Handle cleanup
cleanup() {
    log_info "Shutting down frontend watcher..."
    exit 0
}

trap cleanup SIGINT SIGTERM

# Run main function
main "$@"
