#!/bin/bash

# Fast Build Script for Facebook Video Downloader
# Optimized for maximum compilation speed

set -e

echo "🚀 Starting optimized build process..."

# Set environment variables for faster compilation
export CARGO_BUILD_JOBS=8
export CARGO_BUILD_PIPELINING=true
export CARGO_PROFILE_DEV_BUILD_OVERRIDE_OPT_LEVEL=0
export CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=1
export CARGO_INCREMENTAL=1

# Use sccache if available for faster builds (disable for incremental builds)
if command -v sccache &> /dev/null && [[ "$CARGO_INCREMENTAL" != "1" ]]; then
    export RUSTC_WRAPPER=sccache
    echo "📦 Using sccache for faster compilation"
else
    echo "📦 Using incremental compilation for faster builds"
fi

# Use faster allocator if available
if command -v mimalloc-sys &> /dev/null; then
    export CARGO_PROFILE_DEV_BUILD_OVERRIDE_RUSTFLAGS="-C link-arg=-lmimalloc"
fi

# Clean previous builds if requested
if [[ "$1" == "--clean" ]]; then
    echo "🧹 Cleaning previous builds..."
    cargo clean
    rm -rf dist/*
fi

echo "🔧 Building WASM frontend (development mode)..."
# Build WASM with maximum parallelization
cargo build --target wasm32-unknown-unknown --profile dev-fast

echo "📦 Generating WASM bindings..."
# Generate bindings
wasm-bindgen --out-dir dist --target web --no-typescript \
    target/wasm32-unknown-unknown/dev-fast/facebook_video_downloader_frontend.wasm

echo "🎨 Building CSS..."
# Build CSS in parallel
pnpm run build:css &
CSS_PID=$!

echo "🏗️  Starting Tauri development server..."
# Wait for CSS build to complete
wait $CSS_PID

# Start Tauri with optimized settings
CARGO_BUILD_JOBS=8 pnpm tauri:dev

echo "✅ Build completed successfully!"
