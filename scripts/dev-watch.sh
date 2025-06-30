#!/bin/bash

# Enhanced Development Watch Script for Facebook Video Downloader
# Provides automatic compilation and hot reload for all components

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-8}
WATCH_DEBOUNCE=${WATCH_DEBOUNCE:-500}  # milliseconds
FRONTEND_WATCH_DIRS="src/"
CSS_WATCH_DIRS="src/styles/"
BACKEND_WATCH_DIRS="src-tauri/ crates/"

# PID tracking for cleanup
PIDS=()
TEMP_FILES=()

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up processes...${NC}"
    
    # Kill all background processes
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            echo -e "${BLUE}   Stopping process $pid${NC}"
            kill -TERM "$pid" 2>/dev/null || true
            sleep 1
            kill -KILL "$pid" 2>/dev/null || true
        fi
    done
    
    # Clean up temp files
    for file in "${TEMP_FILES[@]}"; do
        [ -f "$file" ] && rm -f "$file"
    done
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM EXIT

# Utility functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_build() {
    echo -e "${PURPLE}🔨 $1${NC}"
}

log_watch() {
    echo -e "${CYAN}👀 $1${NC}"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for required tools
    command -v cargo >/dev/null 2>&1 || missing_deps+=("cargo")
    command -v wasm-bindgen >/dev/null 2>&1 || missing_deps+=("wasm-bindgen-cli")
    command -v pnpm >/dev/null 2>&1 || missing_deps+=("pnpm")
    command -v postcss >/dev/null 2>&1 || missing_deps+=("postcss-cli")
    
    # Check for file watching tools (we'll use different strategies)
    if ! command -v fswatch >/dev/null 2>&1 && ! command -v inotifywait >/dev/null 2>&1; then
        log_warning "No native file watcher found. Will use polling mode."
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Run 'pnpm run setup' to install missing dependencies"
        exit 1
    fi
    
    log_success "All dependencies available"
}

# Build functions
build_wasm() {
    log_build "Building WASM frontend..."
    CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS cargo build --target wasm32-unknown-unknown --profile dev-fast
    wasm-bindgen --out-dir dist --target web --no-typescript \
        target/wasm32-unknown-unknown/dev-fast/facebook_video_downloader_frontend.wasm
    log_success "WASM build completed"
}

build_css() {
    log_build "Building CSS..."
    postcss src/styles/main.css -o dist/main.css
    log_success "CSS build completed"
}

# Initial build
initial_build() {
    log_info "Performing initial build..."
    
    # Set environment variables for faster compilation
    export CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS
    export CARGO_INCREMENTAL=1
    export CARGO_PROFILE_DEV_BUILD_OVERRIDE_OPT_LEVEL=0
    export CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=1
    
    # Build frontend components
    build_wasm
    build_css
    
    log_success "Initial build completed"
}

# File watching functions
watch_frontend() {
    log_watch "Setting up frontend file watcher..."
    
    # Create a simple file watcher using find and stat
    local last_check=$(date +%s)
    
    while true; do
        # Check for changes in frontend source files
        local current_time=$(date +%s)
        local changed_files=$(find src/ -name "*.rs" -newer /tmp/frontend_last_check 2>/dev/null || true)
        
        if [ -n "$changed_files" ]; then
            log_build "Frontend changes detected, rebuilding WASM..."
            if build_wasm; then
                log_success "Frontend rebuild completed"
                # Touch the Tauri source to trigger reload
                touch src-tauri/src/main.rs
            else
                log_error "Frontend rebuild failed"
            fi
        fi
        
        # Update timestamp
        touch /tmp/frontend_last_check
        sleep 2
    done
}

watch_css() {
    log_watch "Setting up CSS file watcher..."
    
    while true; do
        # Check for changes in CSS files
        local changed_files=$(find src/styles/ -name "*.css" -newer /tmp/css_last_check 2>/dev/null || true)
        
        if [ -n "$changed_files" ]; then
            log_build "CSS changes detected, rebuilding..."
            if build_css; then
                log_success "CSS rebuild completed"
                # Touch the Tauri source to trigger reload
                touch src-tauri/src/main.rs
            else
                log_error "CSS rebuild failed"
            fi
        fi
        
        # Update timestamp
        touch /tmp/css_last_check
        sleep 1
    done
}

# Enhanced file watcher using native tools if available
watch_with_native_tools() {
    if command -v fswatch >/dev/null 2>&1; then
        log_watch "Using fswatch for file monitoring..."
        
        # Watch frontend files
        fswatch -o src/ | while read f; do
            log_build "Frontend changes detected, rebuilding WASM..."
            if build_wasm; then
                log_success "Frontend rebuild completed"
                touch src-tauri/src/main.rs
            fi
        done &
        PIDS+=($!)
        
        # Watch CSS files
        fswatch -o src/styles/ | while read f; do
            log_build "CSS changes detected, rebuilding..."
            if build_css; then
                log_success "CSS rebuild completed"
                touch src-tauri/src/main.rs
            fi
        done &
        PIDS+=($!)
        
    elif command -v inotifywait >/dev/null 2>&1; then
        log_watch "Using inotifywait for file monitoring..."
        
        # Watch frontend files
        inotifywait -m -r -e modify,create,delete src/ --format '%w%f' | while read file; do
            if [[ "$file" == *.rs ]]; then
                log_build "Frontend changes detected: $file"
                if build_wasm; then
                    log_success "Frontend rebuild completed"
                    touch src-tauri/src/main.rs
                fi
            fi
        done &
        PIDS+=($!)
        
        # Watch CSS files
        inotifywait -m -r -e modify,create,delete src/styles/ --format '%w%f' | while read file; do
            if [[ "$file" == *.css ]]; then
                log_build "CSS changes detected: $file"
                if build_css; then
                    log_success "CSS rebuild completed"
                    touch src-tauri/src/main.rs
                fi
            fi
        done &
        PIDS+=($!)
        
    else
        log_warning "No native file watcher available, using polling mode..."
        watch_frontend &
        PIDS+=($!)
        
        watch_css &
        PIDS+=($!)
    fi
}

# Main execution
main() {
    echo -e "${GREEN}🚀 Starting Enhanced Development Environment${NC}"
    echo -e "${BLUE}   Facebook Video Downloader - Hot Reload Enabled${NC}"
    echo -e "${YELLOW}   Press Ctrl+C to stop${NC}\n"
    
    # Initialize timestamp files
    touch /tmp/frontend_last_check
    touch /tmp/css_last_check
    TEMP_FILES+=("/tmp/frontend_last_check" "/tmp/css_last_check")
    
    # Check dependencies
    check_dependencies
    
    # Perform initial build
    initial_build
    
    # Set up file watchers
    watch_with_native_tools
    
    # Start Tauri development server
    log_info "Starting Tauri development server..."
    CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS pnpm tauri:dev &
    TAURI_PID=$!
    PIDS+=($TAURI_PID)
    
    log_success "Development environment started!"
    log_info "Watching for changes in:"
    log_info "  - Frontend (src/): WASM rebuild + hot reload"
    log_info "  - CSS (src/styles/): CSS rebuild + hot reload"
    log_info "  - Backend (src-tauri/, crates/): Automatic Tauri reload"
    
    # Wait for Tauri process
    wait $TAURI_PID
}

# Run main function
main "$@"
