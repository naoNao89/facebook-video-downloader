#!/bin/bash

# Concurrent Development Script using concurrently
# Simpler alternative using Node.js-based file watching

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}🚀 Starting Concurrent Development Environment${NC}"
echo -e "${BLUE}   Using concurrently for process management${NC}"
echo -e "${YELLOW}   Press Ctrl+C to stop all processes${NC}\n"

# Set environment variables
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-8}
export CARGO_INCREMENTAL=1

# Initial build
echo -e "${BLUE}📦 Performing initial build...${NC}"
pnpm run build:wasm:dev
pnpm run build:css

# Start concurrent processes
npx concurrently \
  --prefix-colors "cyan,magenta,yellow,green" \
  --names "WASM,CSS,TAURI,RELOAD" \
  --kill-others \
  --restart-tries 3 \
  "chokidar 'src/**/*.rs' -c 'echo \"🔨 Rebuilding WASM...\" && pnpm run build:wasm:dev && touch src-tauri/src/main.rs'" \
  "chokidar 'src/styles/**/*.css' -c 'echo \"🎨 Rebuilding CSS...\" && pnpm run build:css && touch src-tauri/src/main.rs'" \
  "CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS pnpm tauri:dev" \
  "echo \"👀 File watchers active. Backend changes auto-reload via Tauri.\""
