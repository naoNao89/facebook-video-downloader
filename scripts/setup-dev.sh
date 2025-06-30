#!/bin/bash

# Facebook Video Downloader - Development Setup Script
# This script sets up the development environment for the Facebook Video Downloader

set -e

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

# Function to check version
check_version() {
    local cmd="$1"
    local min_version="$2"
    local current_version="$3"
    
    if [ -z "$current_version" ]; then
        print_error "$cmd is not installed or not in PATH"
        return 1
    fi
    
    print_status "$cmd version: $current_version"
    return 0
}

print_status "Setting up Facebook Video Downloader development environment..."

# Check operating system
OS="$(uname -s)"
case "${OS}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    CYGWIN*)    MACHINE=Cygwin;;
    MINGW*)     MACHINE=MinGw;;
    *)          MACHINE="UNKNOWN:${OS}"
esac

print_status "Detected OS: $MACHINE"

# Check for required tools
print_status "Checking for required tools..."

# Check Rust
if command_exists rustc; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    check_version "Rust" "1.70.0" "$RUST_VERSION"
else
    print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check Cargo
if command_exists cargo; then
    CARGO_VERSION=$(cargo --version | cut -d' ' -f2)
    check_version "Cargo" "1.70.0" "$CARGO_VERSION"
else
    print_error "Cargo is not installed. This should come with Rust."
    exit 1
fi

# Check Node.js
if command_exists node; then
    NODE_VERSION=$(node --version | sed 's/v//')
    check_version "Node.js" "18.0.0" "$NODE_VERSION"
else
    print_error "Node.js is not installed. Please install Node.js v18 or later from https://nodejs.org/"
    exit 1
fi

# Check pnpm
if command_exists pnpm; then
    PNPM_VERSION=$(pnpm --version)
    check_version "pnpm" "8.0.0" "$PNPM_VERSION"
else
    print_warning "pnpm is not installed. Installing pnpm..."
    npm install -g pnpm
    if [ $? -eq 0 ]; then
        print_success "pnpm installed successfully"
    else
        print_error "Failed to install pnpm"
        exit 1
    fi
fi

# Install Rust tools
print_status "Installing Rust development tools..."

# Install Tauri CLI
if ! command_exists cargo-tauri; then
    print_status "Installing Tauri CLI..."
    cargo install tauri-cli
    if [ $? -eq 0 ]; then
        print_success "Tauri CLI installed successfully"
    else
        print_error "Failed to install Tauri CLI"
        exit 1
    fi
else
    print_success "Tauri CLI is already installed"
fi

# Install Trunk
if ! command_exists trunk; then
    print_status "Installing Trunk..."
    cargo install trunk
    if [ $? -eq 0 ]; then
        print_success "Trunk installed successfully"
    else
        print_error "Failed to install Trunk"
        exit 1
    fi
else
    print_success "Trunk is already installed"
fi

# Install additional Rust tools
print_status "Installing additional Rust tools..."

# Install rustfmt
if ! command_exists rustfmt; then
    rustup component add rustfmt
fi

# Install clippy
if ! rustup component list --installed | grep -q clippy; then
    rustup component add clippy
fi

# Install WASM target
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    print_status "Installing WASM target..."
    rustup target add wasm32-unknown-unknown
fi

# Platform-specific dependencies
case "$MACHINE" in
    Linux)
        print_status "Installing Linux dependencies..."
        
        # Detect package manager
        if command_exists apt-get; then
            print_status "Using apt package manager..."
            sudo apt-get update
            sudo apt-get install -y \
                libwebkit2gtk-4.0-dev \
                build-essential \
                curl \
                wget \
                file \
                libssl-dev \
                libgtk-3-dev \
                libayatana-appindicator3-dev \
                librsvg2-dev \
                ffmpeg
        elif command_exists dnf; then
            print_status "Using dnf package manager..."
            sudo dnf install -y \
                webkit2gtk4.0-devel \
                openssl-devel \
                curl \
                wget \
                file \
                libappindicator-gtk3-devel \
                librsvg2-devel \
                ffmpeg
            sudo dnf group install -y "C Development Tools and Libraries"
        elif command_exists pacman; then
            print_status "Using pacman package manager..."
            sudo pacman -S --needed \
                webkit2gtk \
                base-devel \
                curl \
                wget \
                file \
                openssl \
                gtk3 \
                libappindicator-gtk3 \
                librsvg \
                ffmpeg
        else
            print_warning "Unknown package manager. Please install dependencies manually."
        fi
        ;;
    Mac)
        print_status "Installing macOS dependencies..."
        
        # Check for Xcode Command Line Tools
        if ! xcode-select -p &> /dev/null; then
            print_status "Installing Xcode Command Line Tools..."
            xcode-select --install
        fi
        
        # Install FFmpeg if Homebrew is available
        if command_exists brew; then
            print_status "Installing FFmpeg via Homebrew..."
            brew install ffmpeg
        else
            print_warning "Homebrew not found. Please install FFmpeg manually."
        fi
        ;;
    *)
        print_warning "Unknown operating system. Please install dependencies manually."
        ;;
esac

# Install Node.js dependencies
print_status "Installing Node.js dependencies..."
pnpm install

if [ $? -eq 0 ]; then
    print_success "Node.js dependencies installed successfully"
else
    print_error "Failed to install Node.js dependencies"
    exit 1
fi

# Build the project to verify setup
print_status "Building project to verify setup..."

# Build frontend
print_status "Building frontend..."
trunk build

if [ $? -eq 0 ]; then
    print_success "Frontend build successful"
else
    print_error "Frontend build failed"
    exit 1
fi

# Build Tauri app (debug mode)
print_status "Building Tauri app (debug mode)..."
cargo tauri build --debug

if [ $? -eq 0 ]; then
    print_success "Tauri app build successful"
else
    print_error "Tauri app build failed"
    exit 1
fi

# Create development scripts
print_status "Creating development scripts..."

# Create start script
cat > start-dev.sh << 'EOF'
#!/bin/bash
echo "Starting Facebook Video Downloader in development mode..."
cargo tauri dev
EOF

chmod +x start-dev.sh

# Create build script
cat > build-release.sh << 'EOF'
#!/bin/bash
echo "Building Facebook Video Downloader for release..."
trunk build --release
cargo tauri build
EOF

chmod +x build-release.sh

print_success "Development scripts created:"
print_status "  - start-dev.sh: Start development server"
print_status "  - build-release.sh: Build release version"

# Final verification
print_status "Running final verification..."

# Check if all tools are working
if command_exists cargo-tauri && command_exists trunk; then
    print_success "All development tools are installed and working!"
else
    print_error "Some tools are missing. Please check the installation."
    exit 1
fi

print_success "Development environment setup complete!"
print_status ""
print_status "Next steps:"
print_status "1. Run './start-dev.sh' to start the development server"
print_status "2. Open your browser to http://localhost:8080 (if running frontend only)"
print_status "3. The Tauri app window will open automatically in dev mode"
print_status "4. Make changes to the code and see them reflected immediately"
print_status ""
print_status "Useful commands:"
print_status "  - pnpm tauri:dev     # Start development with hot reload"
print_status "  - pnpm build         # Build frontend only"
print_status "  - pnpm tauri:build   # Build complete application"
print_status "  - cargo fmt          # Format Rust code"
print_status "  - cargo clippy       # Run linter"
print_status "  - cargo test         # Run tests"
print_status ""
print_success "Happy coding! 🚀"
