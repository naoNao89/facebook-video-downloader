#!/bin/bash

# Facebook Video Downloader Build Script
# Supports both development and production builds

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check dependencies
check_dependencies() {
    print_status "Checking dependencies..."
    
    local missing_deps=()
    
    if ! command_exists cargo; then
        missing_deps+=("cargo (Rust)")
    fi
    
    if ! command_exists trunk; then
        missing_deps+=("trunk")
    fi
    
    if ! command_exists pnpm; then
        missing_deps+=("pnpm")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        echo ""
        echo "Please install missing dependencies:"
        echo "  cargo install trunk"
        echo "  npm install -g pnpm"
        exit 1
    fi
    
    print_success "All dependencies found"
}

# Function to install Node.js dependencies
install_node_deps() {
    print_status "Installing Node.js dependencies..."
    if ! pnpm install; then
        print_error "Failed to install Node.js dependencies"
        exit 1
    fi
    print_success "Node.js dependencies installed"
}

# Function to build frontend
build_frontend() {
    local mode=$1
    print_status "Building frontend in $mode mode..."
    
    if [ "$mode" = "release" ]; then
        if ! trunk build --release; then
            print_error "Frontend build failed"
            exit 1
        fi
    else
        if ! trunk build; then
            print_error "Frontend build failed"
            exit 1
        fi
    fi
    
    print_success "Frontend build completed"
}

# Function to build Tauri app
build_tauri() {
    local mode=$1
    print_status "Building Tauri app in $mode mode..."
    
    if [ "$mode" = "release" ]; then
        if ! cargo tauri build; then
            print_error "Tauri build failed"
            exit 1
        fi
    else
        if ! cargo tauri build --debug; then
            print_error "Tauri build failed"
            exit 1
        fi
    fi
    
    print_success "Tauri build completed"
}

# Function to optimize assets
optimize_assets() {
    print_status "Optimizing assets..."
    
    # Optimize WASM files if wasm-opt is available
    if command_exists wasm-opt; then
        find dist -name "*.wasm" -exec wasm-opt -Oz {} -o {} \;
        print_success "WASM files optimized"
    else
        print_warning "wasm-opt not found, skipping WASM optimization"
    fi
    
    # Optimize SVG files if svgo is available
    if command_exists svgo; then
        find dist -name "*.svg" -exec svgo {} \;
        print_success "SVG files optimized"
    else
        print_warning "svgo not found, skipping SVG optimization"
    fi
}

# Function to show build summary
show_summary() {
    print_status "Build Summary:"
    echo ""
    
    if [ -d "dist" ]; then
        echo "Frontend build output:"
        echo "  Location: ./dist/"
        echo "  Size: $(du -sh dist | cut -f1)"
        
        # Show individual file sizes
        if [ -n "$(find dist -name "*.wasm" 2>/dev/null)" ]; then
            echo "  WASM files: $(find dist -name "*.wasm" -exec du -ch {} + | grep total | cut -f1)"
        fi
        if [ -n "$(find dist -name "*.js" 2>/dev/null)" ]; then
            echo "  JS files: $(find dist -name "*.js" -exec du -ch {} + | grep total | cut -f1)"
        fi
        if [ -n "$(find dist -name "*.css" 2>/dev/null)" ]; then
            echo "  CSS files: $(find dist -name "*.css" -exec du -ch {} + | grep total | cut -f1)"
        fi
    fi
    
    if [ -d "src-tauri/target" ]; then
        echo ""
        echo "Tauri build output:"
        echo "  Location: ./src-tauri/target/"
        
        # Find the main executable
        if [ -f "src-tauri/target/release/facebook-video-downloader" ]; then
            echo "  Executable: $(du -sh src-tauri/target/release/facebook-video-downloader | cut -f1)"
        elif [ -f "src-tauri/target/debug/facebook-video-downloader" ]; then
            echo "  Executable (debug): $(du -sh src-tauri/target/debug/facebook-video-downloader | cut -f1)"
        fi
    fi
    
    echo ""
    print_success "Build completed successfully!"
}

# Function to show usage
show_usage() {
    echo "Facebook Video Downloader Build Script"
    echo ""
    echo "Usage: $0 [MODE]"
    echo ""
    echo "Modes:"
    echo "  dev          Build in development mode (default)"
    echo "  release      Build in release mode with optimizations"
    echo "  frontend     Build only the frontend"
    echo "  tauri        Build only the Tauri app"
    echo "  clean        Clean build artifacts"
    echo "  help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0           # Build in development mode"
    echo "  $0 release   # Build in release mode"
    echo "  $0 frontend  # Build only frontend"
    echo "  $0 clean     # Clean build artifacts"
}

# Function to clean build artifacts
clean_build() {
    print_status "Cleaning build artifacts..."
    
    # Clean Rust artifacts
    if [ -d "target" ]; then
        rm -rf target
        print_success "Cleaned Rust target directory"
    fi
    
    if [ -d "src-tauri/target" ]; then
        rm -rf src-tauri/target
        print_success "Cleaned Tauri target directory"
    fi
    
    # Clean frontend artifacts
    if [ -d "dist" ]; then
        rm -rf dist
        print_success "Cleaned frontend dist directory"
    fi
    
    # Clean Node.js artifacts
    if [ -d "node_modules" ]; then
        rm -rf node_modules
        print_success "Cleaned node_modules directory"
    fi
    
    print_success "Clean completed"
}

# Main script logic
main() {
    local mode=${1:-dev}
    
    case $mode in
        "help"|"-h"|"--help")
            show_usage
            exit 0
            ;;
        "clean")
            clean_build
            exit 0
            ;;
        "dev"|"development")
            print_status "Starting development build..."
            check_dependencies
            install_node_deps
            build_frontend "dev"
            build_tauri "dev"
            show_summary
            ;;
        "release"|"production")
            print_status "Starting release build..."
            check_dependencies
            install_node_deps
            build_frontend "release"
            optimize_assets
            build_tauri "release"
            show_summary
            ;;
        "frontend")
            print_status "Building frontend only..."
            check_dependencies
            install_node_deps
            build_frontend "release"
            optimize_assets
            print_success "Frontend build completed"
            ;;
        "tauri")
            print_status "Building Tauri app only..."
            check_dependencies
            build_tauri "release"
            print_success "Tauri build completed"
            ;;
        *)
            print_error "Unknown mode: $mode"
            echo ""
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
