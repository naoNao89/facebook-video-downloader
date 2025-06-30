# Hot Reload Development Environment

This document describes the enhanced development environment with automatic compilation and hot reload functionality for the Facebook Video Downloader Tauri application.

## Overview

The development environment now provides comprehensive automatic compilation and hot reload for:

- **Rust Backend Code** (`src-tauri/`, `crates/`) - Automatic Tauri reload
- **Frontend Yew/WASM Code** (`src/`) - Automatic WASM rebuild + hot reload
- **CSS Files** (`src/styles/`) - Automatic CSS rebuild + hot reload

## Quick Start

### Start Development Environment

```bash
# Primary development command (recommended)
pnpm run dev

# Alternative commands
pnpm run dev:concurrent    # Same as above
pnpm run dev:watch         # Native file watcher version
pnpm run dev:simple        # Basic Tauri dev without enhanced watching
```

### What Happens

1. **Initial Build**: Performs WASM and CSS builds
2. **File Watchers**: Sets up intelligent file monitoring
3. **Tauri Server**: Starts the development server
4. **Hot Reload**: Automatically rebuilds and reloads on changes

## File Watching Behavior

### Frontend Changes (`src/**/*.rs`)
- **Trigger**: Any `.rs` file change in `src/` directory
- **Action**: 
  1. Rebuilds WASM with `cargo build --target wasm32-unknown-unknown --profile dev-fast`
  2. Generates new bindings with `wasm-bindgen`
  3. Triggers Tauri reload by touching `src-tauri/src/main.rs`
- **Speed**: ~3-5 seconds for incremental builds

### CSS Changes (`src/styles/**/*.css`)
- **Trigger**: Any `.css` file change in `src/styles/`
- **Action**:
  1. Rebuilds CSS with PostCSS and Tailwind
  2. Triggers Tauri reload
- **Speed**: ~1-2 seconds

### Backend Changes (`src-tauri/`, `crates/`)
- **Trigger**: Any Rust file change in backend directories
- **Action**: Automatic Tauri hot reload (built-in)
- **Speed**: ~2-4 seconds for incremental builds

## Development Scripts

### Main Scripts

| Script | Description | Use Case |
|--------|-------------|----------|
| `pnpm run dev` | Enhanced concurrent development | **Primary development** |
| `pnpm run dev:concurrent` | Same as above | Alternative command |
| `pnpm run dev:watch` | Native file watcher version | Advanced users |
| `pnpm run dev:simple` | Basic Tauri dev | Debugging/fallback |

### Build Scripts

| Script | Description |
|--------|-------------|
| `pnpm run build:wasm:dev` | Build WASM in dev-fast mode |
| `pnpm run build:css` | Build CSS with PostCSS |
| `pnpm run watch:css` | Watch CSS files continuously |
| `pnpm run watch:frontend` | Watch frontend files |

## File Structure

```
scripts/
├── dev-concurrent.sh      # Main concurrent development script
├── dev-watch.sh          # Advanced native file watcher
├── watch-frontend.sh     # Frontend-specific watcher
├── watch-wasm.sh         # WASM-specific watcher
└── fast-build.sh         # Optimized build script
```

## Configuration

### Environment Variables

```bash
# Number of parallel Cargo build jobs (default: 8)
export CARGO_BUILD_JOBS=8

# Enable incremental compilation (default: 1)
export CARGO_INCREMENTAL=1

# File watch debounce time in milliseconds (default: 500)
export WATCH_DEBOUNCE=500
```

### Build Profiles

The development environment uses optimized build profiles:

- **WASM**: `dev-fast` profile (faster compilation)
- **Backend**: `dev` profile with incremental compilation
- **CSS**: Development mode with source maps

## Troubleshooting

### Common Issues

1. **File watchers not working**
   ```bash
   # Install native file watchers (macOS)
   brew install fswatch
   
   # Or use polling mode (automatic fallback)
   ```

2. **Slow rebuilds**
   ```bash
   # Increase build jobs
   export CARGO_BUILD_JOBS=16
   
   # Clean and restart
   pnpm run clean
   pnpm run dev
   ```

3. **Port conflicts**
   ```bash
   # Kill existing processes
   pkill -f "tauri dev"
   pnpm run dev
   ```

### Debug Mode

For debugging the development environment:

```bash
# Enable verbose logging
DEBUG=1 pnpm run dev

# Use simple mode without file watchers
pnpm run dev:simple
```

## Performance Optimization

### Build Speed

- **Incremental Compilation**: Enabled by default
- **Parallel Jobs**: Configurable via `CARGO_BUILD_JOBS`
- **Fast Profile**: Uses `dev-fast` for WASM builds
- **Smart Watching**: Only rebuilds changed components

### Memory Usage

- **Debounced Rebuilds**: Prevents rapid successive builds
- **Selective Watching**: Only watches relevant file types
- **Process Management**: Proper cleanup on exit

## Advanced Usage

### Custom File Watchers

You can create custom watchers for specific needs:

```bash
# Watch specific directories
chokidar "src/components/**/*.rs" -c "echo 'Component changed'"

# Watch with custom debounce
chokidar "src/**/*.rs" --debounce 1000 -c "pnpm run build:wasm:dev"
```

### Integration with IDEs

The hot reload works seamlessly with:

- **VS Code**: File changes trigger automatic rebuilds
- **IntelliJ IDEA**: Save actions trigger hot reload
- **Vim/Neovim**: Write operations trigger rebuilds

## Dependencies

### Required Tools

- `cargo` - Rust compiler
- `wasm-bindgen-cli` - WASM bindings generator
- `pnpm` - Package manager
- `postcss-cli` - CSS processor

### Optional Tools

- `fswatch` (macOS) - Native file watcher
- `inotifywait` (Linux) - Native file watcher
- `chokidar-cli` - Cross-platform file watcher (installed)
- `concurrently` - Process manager (installed)

## Comparison with Standard Tauri Dev

| Feature | Standard `tauri dev` | Enhanced Environment |
|---------|---------------------|---------------------|
| Backend Hot Reload | ✅ | ✅ |
| Frontend Hot Reload | ❌ | ✅ |
| CSS Hot Reload | ❌ | ✅ |
| Build Optimization | Basic | Advanced |
| File Watching | Backend only | Full stack |
| Process Management | Single | Concurrent |
| Debug Output | Limited | Color-coded |

## Contributing

When contributing to the development environment:

1. Test changes with `pnpm run dev`
2. Ensure file watchers work on your platform
3. Update documentation for new features
4. Test with both native and polling file watchers

## Support

For issues with the development environment:

1. Check the terminal output for error messages
2. Try `pnpm run dev:simple` as a fallback
3. Verify all dependencies are installed
4. Check file permissions on scripts
5. Report issues with platform and error details
