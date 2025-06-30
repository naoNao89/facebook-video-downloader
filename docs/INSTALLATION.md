# Installation Guide

This guide will help you install and set up the Facebook Video Downloader on your system.

## System Requirements

### Minimum Requirements
- **RAM**: 4 GB
- **Storage**: 500 MB free space
- **Internet**: Stable internet connection

### Operating System Support
- **Windows**: Windows 10 (version 1903) or later
- **macOS**: macOS 10.15 (Catalina) or later
- **Linux**: Ubuntu 18.04 or equivalent distributions

### Optional Dependencies
- **FFmpeg**: Required for DASH stream combination and advanced video processing
- **Hardware Acceleration**: GPU support for faster video processing (optional)

## Download Options

### Pre-built Binaries (Recommended)

Download the latest release from the [GitHub Releases page](https://github.com/username/facebook-video-downloader/releases):

#### Windows
- **Installer**: `facebook-video-downloader_x.x.x_x64_en-US.msi`
- **Portable**: `facebook-video-downloader_x.x.x_x64.exe`

#### macOS
- **Intel Macs**: `facebook-video-downloader_x.x.x_x64.dmg`
- **Apple Silicon**: `facebook-video-downloader_x.x.x_aarch64.dmg`

#### Linux
- **AppImage**: `facebook-video-downloader_x.x.x_amd64.AppImage`
- **Debian/Ubuntu**: `facebook-video-downloader_x.x.x_amd64.deb`
- **RPM**: `facebook-video-downloader_x.x.x_x86_64.rpm`

## Installation Instructions

### Windows Installation

#### Using MSI Installer (Recommended)
1. **Download** the MSI installer from the releases page
2. **Right-click** the installer and select "Run as administrator"
3. **Follow** the installation wizard
4. **Launch** from Start Menu or Desktop shortcut

#### Using Portable Executable
1. **Download** the portable executable
2. **Create** a folder for the application (e.g., `C:\Tools\FacebookDownloader`)
3. **Extract** or move the executable to the folder
4. **Run** the executable directly

#### Windows Defender SmartScreen
If Windows Defender blocks the installation:
1. Click "More info" on the SmartScreen dialog
2. Click "Run anyway"
3. This is normal for new applications without code signing certificates

### macOS Installation

#### Using DMG Package (Recommended)
1. **Download** the appropriate DMG file for your Mac
2. **Double-click** the DMG file to mount it
3. **Drag** the application to the Applications folder
4. **Launch** from Applications or Spotlight

#### Gatekeeper Security
If macOS blocks the application:
1. **Right-click** the application in Applications
2. **Select** "Open" from the context menu
3. **Click** "Open" in the security dialog
4. The app will be allowed to run in the future

#### Alternative Method
```bash
# Remove quarantine attribute
sudo xattr -rd com.apple.quarantine /Applications/FacebookVideoDownloader.app
```

### Linux Installation

#### Using AppImage (Universal)
1. **Download** the AppImage file
2. **Make it executable**:
   ```bash
   chmod +x facebook-video-downloader_x.x.x_amd64.AppImage
   ```
3. **Run** the AppImage:
   ```bash
   ./facebook-video-downloader_x.x.x_amd64.AppImage
   ```

#### Using Debian Package
```bash
# Download and install
wget https://github.com/username/facebook-video-downloader/releases/download/vx.x.x/facebook-video-downloader_x.x.x_amd64.deb
sudo dpkg -i facebook-video-downloader_x.x.x_amd64.deb

# Fix dependencies if needed
sudo apt-get install -f
```

#### Using RPM Package
```bash
# Fedora/CentOS/RHEL
sudo rpm -i facebook-video-downloader_x.x.x_x86_64.rpm

# Or using dnf
sudo dnf install facebook-video-downloader_x.x.x_x86_64.rpm
```

## Building from Source

### Prerequisites

#### Install Rust
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Install Node.js and pnpm
```bash
# Install Node.js (v18 or later)
# Visit https://nodejs.org/ or use a package manager

# Install pnpm
npm install -g pnpm

# Verify installation
node --version
pnpm --version
```

#### Install Tauri CLI
```bash
cargo install tauri-cli
cargo install trunk
```

#### Platform-specific Dependencies

**Linux (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**Linux (Fedora)**:
```bash
sudo dnf install webkit2gtk4.0-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
sudo dnf group install "C Development Tools and Libraries"
```

**macOS**:
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Windows**:
- Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Build Process

1. **Clone the repository**:
   ```bash
   git clone https://github.com/username/facebook-video-downloader.git
   cd facebook-video-downloader
   ```

2. **Install dependencies**:
   ```bash
   pnpm install
   ```

3. **Build the application**:
   ```bash
   # Development build
   pnpm tauri:dev
   
   # Production build
   pnpm tauri:build
   ```

4. **Find the built application**:
   - **Windows**: `src-tauri/target/release/bundle/msi/`
   - **macOS**: `src-tauri/target/release/bundle/dmg/`
   - **Linux**: `src-tauri/target/release/bundle/appimage/`

## FFmpeg Installation (Optional)

FFmpeg is required for combining DASH video and audio streams.

### Windows
1. **Download** FFmpeg from [ffmpeg.org](https://ffmpeg.org/download.html)
2. **Extract** to a folder (e.g., `C:\ffmpeg`)
3. **Add** `C:\ffmpeg\bin` to your PATH environment variable
4. **Restart** the application

### macOS
```bash
# Using Homebrew
brew install ffmpeg

# Using MacPorts
sudo port install ffmpeg
```

### Linux
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# Fedora
sudo dnf install ffmpeg

# Arch Linux
sudo pacman -S ffmpeg
```

### Verify FFmpeg Installation
```bash
ffmpeg -version
```

## Post-Installation Setup

### First Launch
1. **Launch** the application
2. **Grant** necessary permissions when prompted
3. **Configure** default download directory in Settings
4. **Test** with a public Facebook video URL

### Configuration
- **Download Directory**: Choose where videos will be saved
- **Quality Preferences**: Set default quality selection
- **Theme**: Choose light, dark, or system theme
- **Network Settings**: Configure timeouts and retry attempts

### Troubleshooting

#### Common Issues

**Application won't start**:
- Check system requirements
- Verify all dependencies are installed
- Check antivirus software isn't blocking the app

**Downloads fail**:
- Verify internet connection
- Check if FFmpeg is installed (for DASH streams)
- Try different video URLs
- Check application logs

**Permission errors**:
- Run as administrator (Windows)
- Check file system permissions
- Verify download directory is writable

#### Getting Help
- Check the [Troubleshooting Guide](TROUBLESHOOTING.md)
- Search [GitHub Issues](https://github.com/username/facebook-video-downloader/issues)
- Create a new issue with detailed information

## Uninstallation

### Windows
- **MSI Installer**: Use "Add or Remove Programs" in Windows Settings
- **Portable**: Simply delete the application folder

### macOS
- **Drag** the application from Applications to Trash
- **Empty** Trash to complete removal

### Linux
- **AppImage**: Delete the AppImage file
- **Package**: Use your package manager to remove
  ```bash
  # Debian/Ubuntu
  sudo apt remove facebook-video-downloader
  
  # Fedora
  sudo dnf remove facebook-video-downloader
  ```

### Clean Removal
To remove all application data:
- **Windows**: Delete `%APPDATA%\facebook-video-downloader`
- **macOS**: Delete `~/Library/Application Support/facebook-video-downloader`
- **Linux**: Delete `~/.config/facebook-video-downloader`
