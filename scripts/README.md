# 🛠️ Build Scripts Directory

This directory contains all build, development, and maintenance scripts for the Facebook Video Downloader project.

## 📋 Available Scripts

### 🚀 **Development & Build Scripts**

#### `fast-build.sh`
**Optimized development build script with parallel compilation**
```bash
./scripts/fast-build.sh          # Fast development build
./scripts/fast-build.sh --clean  # Clean build with optimizations
```

**Features:**
- Uses all 8 CPU cores for parallel compilation
- Optimized WASM build (0.38s vs several minutes)
- Environment variable optimizations
- Automatic sccache integration if available
- Parallel CSS building

#### `build.sh`
**Standard production build script**
```bash
./scripts/build.sh
```

**Features:**
- Production-ready builds
- Full optimization
- Release mode compilation

#### `setup-dev.sh`
**Development environment setup**
```bash
./scripts/setup-dev.sh
```

**Features:**
- Installs all required dependencies
- Sets up Rust toolchain
- Configures development environment

### 🧪 **Testing Scripts**

#### `run_tests.sh`
**Comprehensive test runner**
```bash
./scripts/run_tests.sh
```

**Features:**
- Runs all test suites
- Unit tests, integration tests, and more
- Comprehensive test coverage

## 📦 **NPM Script Integration**

All scripts are integrated into `package.json` for easy access:

```bash
# Development
pnpm run dev:fast           # Fast development server
pnpm run setup              # Setup development environment

# Building
pnpm run build:script       # Production build
pnpm run build:wasm:fast    # Fast WASM build

# Testing
pnpm run test:all           # Run all tests
pnpm run test               # Run Cargo tests

# Maintenance
pnpm run clean:fast         # Fast clean build
pnpm run lint               # Code linting
pnpm run format             # Code formatting
```

## ⚡ **Performance Optimizations**

### **Parallel Compilation Settings**
- **8 parallel jobs** (`CARGO_BUILD_JOBS=8`)
- **256 codegen units** for maximum parallelization
- **Optimized dev profile** with reduced debug info

### **Build Speed Improvements**
- **WASM build**: 0.38 seconds (vs. several minutes)
- **Apple Silicon optimized** for M1/M2 Macs
- **sccache integration** for faster rebuilds

### **Configuration Files**
- `.cargo/config.toml` - Cargo optimization settings
- Custom dev profiles for faster compilation
- Environment variable optimizations

## 🔧 **Script Maintenance**

### **Making Scripts Executable**
```bash
chmod +x scripts/*.sh
```

### **Adding New Scripts**
1. Create script in `scripts/` directory
2. Make it executable: `chmod +x scripts/your-script.sh`
3. Add to `package.json` scripts section
4. Document in this README

### **Best Practices**
- Use descriptive names
- Include error handling
- Add usage documentation
- Test on different platforms
- Follow shell scripting conventions

## 📚 **Related Documentation**

- [Installation Guide](../docs/INSTALLATION.md)
- [Main README](../README.md)
- [Project Documentation](../docs/)

## 🐛 **Troubleshooting**

### **Permission Issues**
```bash
chmod +x scripts/*.sh
```

### **Build Failures**
```bash
./scripts/fast-build.sh --clean  # Clean rebuild
```

### **Environment Issues**
```bash
./scripts/setup-dev.sh           # Reinstall dependencies
```
