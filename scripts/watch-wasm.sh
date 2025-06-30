#!/bin/bash

# WASM Build Watcher for Facebook Video Downloader
# Optimized for fast WASM compilation and hot reload

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-8}
WATCH_DIRS=("src/" "Cargo.toml")
BUILD_PROFILE="dev-fast"
TARGET="wasm32-unknown-unknown"
OUTPUT_DIR="dist"

log_info() {
    echo -e "${BLUE}[WASM Watcher] $1${NC}"
}

log_build() {
    echo -e "${PURPLE}[WASM Watcher] 🔨 $1${NC}"
}

log_success() {
    echo -e "${GREEN}[WASM Watcher] ✅ $1${NC}"
}

log_error() {
    echo -e "${RED}[WASM Watcher] ❌ $1${NC}"
}

# Optimized WASM build function
build_wasm_optimized() {
    local start_time=$(date +%s)
    
    log_build "Starting optimized WASM build..."
    
    # Set environment variables for maximum speed
    export CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS
    export CARGO_INCREMENTAL=1
    export CARGO_PROFILE_DEV_BUILD_OVERRIDE_OPT_LEVEL=0
    export CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=1
    export CARGO_BUILD_PIPELINING=true
    
    # Use faster linker if available
    if command -v lld >/dev/null 2>&1; then
        export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=lld
    fi
    
    # Build WASM
    if cargo build --target $TARGET --profile $BUILD_PROFILE --jobs $CARGO_BUILD_JOBS; then
        log_success "Cargo build completed"
        
        # Generate bindings
        local wasm_file="target/$TARGET/$BUILD_PROFILE/facebook_video_downloader_frontend.wasm"
        
        if [ -f "$wasm_file" ]; then
            if wasm-bindgen --out-dir $OUTPUT_DIR --target web --no-typescript "$wasm_file"; then
                local end_time=$(date +%s)
                local duration=$((end_time - start_time))
                log_success "WASM build completed in ${duration}s"
                
                # Notify Tauri of changes
                touch src-tauri/src/main.rs 2>/dev/null || true
                
                return 0
            else
                log_error "wasm-bindgen failed"
                return 1
            fi
        else
            log_error "WASM file not found: $wasm_file"
            return 1
        fi
    else
        log_error "Cargo build failed"
        return 1
    fi
}

# Incremental build check
should_rebuild() {
    local wasm_output="$OUTPUT_DIR/facebook_video_downloader_frontend.js"
    
    # If output doesn't exist, always rebuild
    if [ ! -f "$wasm_output" ]; then
        return 0
    fi
    
    # Check if any source files are newer than output
    local newer_files=$(find src/ -name "*.rs" -newer "$wasm_output" 2>/dev/null)
    
    if [ -n "$newer_files" ]; then
        log_info "Source files changed: $(echo $newer_files | tr '\n' ' ')"
        return 0
    fi
    
    # Check Cargo.toml
    if [ "Cargo.toml" -nt "$wasm_output" ]; then
        log_info "Cargo.toml changed"
        return 0
    fi
    
    return 1
}

# File watching with smart rebuilding
watch_and_build() {
    log_info "Starting intelligent WASM file watcher..."
    
    # Perform initial build if needed
    if should_rebuild; then
        build_wasm_optimized
    else
        log_info "WASM output is up to date"
    fi
    
    # Choose watching strategy
    if command -v fswatch >/dev/null 2>&1; then
        log_info "Using fswatch for file monitoring"
        
        # Watch with debouncing
        fswatch -o -r -l 0.5 "${WATCH_DIRS[@]}" | while read f; do
            if should_rebuild; then
                build_wasm_optimized
            else
                log_info "No rebuild needed"
            fi
        done
        
    elif command -v inotifywait >/dev/null 2>&1; then
        log_info "Using inotifywait for file monitoring"
        
        while true; do
            # Wait for changes
            inotifywait -r -e modify,create,delete,move "${WATCH_DIRS[@]}" >/dev/null 2>&1
            
            # Debounce rapid changes
            sleep 0.5
            
            if should_rebuild; then
                build_wasm_optimized
            fi
        done
        
    else
        log_info "Using polling mode for file monitoring"
        
        while true; do
            if should_rebuild; then
                build_wasm_optimized
            fi
            sleep 2
        done
    fi
}

# Cleanup function
cleanup() {
    log_info "Shutting down WASM watcher..."
    exit 0
}

# Setup
setup_environment() {
    log_info "Setting up WASM build environment..."
    
    # Ensure output directory exists
    mkdir -p "$OUTPUT_DIR"
    
    # Check for required tools
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "cargo not found"
        exit 1
    fi
    
    if ! command -v wasm-bindgen >/dev/null 2>&1; then
        log_error "wasm-bindgen not found. Install with: cargo install wasm-bindgen-cli"
        exit 1
    fi
    
    # Check for WASM target
    if ! rustup target list --installed | grep -q "$TARGET"; then
        log_info "Installing WASM target..."
        rustup target add "$TARGET"
    fi
    
    log_success "Environment setup completed"
}

# Main function
main() {
    log_info "Starting WASM build watcher..."
    
    setup_environment
    
    # Set up signal handlers
    trap cleanup SIGINT SIGTERM
    
    # Start watching and building
    watch_and_build
}

# Run main function
main "$@"
