# Facebook Video Downloader

A powerful, modern desktop application for downloading Facebook videos, Reels, and private content. Built with Tauri, Yew, and Tailwind CSS for a native desktop experience with web technologies.

![Facebook Video Downloader](docs/images/app-screenshot.png)

## ✨ Features

### 🎥 Video Download Capabilities
- **Regular Facebook Videos**: Download public videos from Facebook posts
- **Facebook Reels**: Download short-form video content
- **Private Content**: Download private videos using the view-source method
- **Multiple Quality Options**: Choose from available quality levels (1080p, 720p, 480p, 360p)
- **DASH Stream Combination**: Automatically combine video and audio streams using FFmpeg

### 🚀 Advanced Features
- **Batch Processing**: Download multiple videos simultaneously
- **Progress Tracking**: Real-time download progress with speed and ETA
- **Resume Downloads**: Resume interrupted downloads
- **Desktop Native**: No CORS restrictions, full file system access
- **Cross-Platform**: Works on Windows, macOS, and Linux

### 🎨 Modern UI/UX
- **Dark/Light Theme**: Automatic theme detection with manual override
- **Responsive Design**: Optimized for different screen sizes
- **Real-time Notifications**: System notifications for download completion
- **Intuitive Interface**: Clean, modern design with smooth animations

### ⚙️ Configuration & Privacy
- **Configurable Settings**: Customize download directory, quality preferences, and more
- **Privacy Focused**: Respects user privacy with optional view-source extraction
- **Local Processing**: All video processing happens locally on your machine
- **No Data Collection**: No telemetry or user data collection

## 🛠️ Technology Stack

- **Backend**: Rust with Tauri framework
- **Frontend**: Yew (Rust WebAssembly framework)
- **Styling**: Tailwind CSS with custom components
- **Video Processing**: FFmpeg for stream combination
- **HTTP Client**: Reqwest with advanced retry logic
- **State Management**: Yew hooks and context providers

## 📦 Installation

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Node.js**: Version 18+ with pnpm package manager
- **FFmpeg**: Required for DASH stream combination
  - Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)
  - macOS: `brew install ffmpeg`
  - Linux: `sudo apt install ffmpeg` (Ubuntu/Debian)

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/username/facebook-video-downloader.git
   cd facebook-video-downloader
   ```

2. **Install dependencies**:
   ```bash
   # Install Node.js dependencies
   pnpm install
   
   # Install Rust tools
   cargo install trunk
   cargo install tauri-cli
   ```

3. **Setup development environment**:
   ```bash
   # Automated setup (recommended)
   pnpm run setup

   # Or manual setup
   pnpm run setup:manual
   ```

4. **Run in development mode**:
   ```bash
   # Enhanced development with hot reload (recommended)
   pnpm run dev

   # Fast development server (optimized)
   pnpm run dev:fast

   # Standard development server
   pnpm tauri:dev
   ```

### Building for Production

1. **Build the application**:
   ```bash
   # Standard production build
   pnpm tauri:build

   # Or use build script
   pnpm run build:script
   ```

2. **Find the built application**:
   - Windows: `src-tauri/target/release/bundle/msi/`
   - macOS: `src-tauri/target/release/bundle/dmg/`
   - Linux: `src-tauri/target/release/bundle/deb/` or `src-tauri/target/release/bundle/appimage/`

## 🚀 Usage

### Basic Video Download

1. **Launch the application**
2. **Paste a Facebook video URL** in the input field
3. **Click "Extract Video Info"** to analyze the video
4. **Select your preferred quality** from the available options
5. **Click "Download"** to start the download

### Downloading Private Videos

For private or restricted videos:

1. **Open the video in your browser**
2. **Right-click and select "View Page Source"** (or press Ctrl+U)
3. **Copy the entire page source**
4. **Switch to the "View Source" tab** in the application
5. **Paste the page source** and click "Extract"

### Batch Downloads

1. **Navigate to the "Batch" tab**
2. **Add multiple URLs** (one per line)
3. **Select output directory** and quality preference
4. **Click "Start Batch Download"**

### Configuration

Access settings through the gear icon:

- **Download Directory**: Choose where videos are saved
- **Quality Preference**: Set default quality selection
- **Concurrent Downloads**: Number of simultaneous downloads
- **Theme**: Light, dark, or system preference
- **Notifications**: Enable/disable system notifications

## 🛠️ Build Scripts

The project includes optimized build scripts located in the `scripts/` directory:

### Available Scripts

- **`scripts/fast-build.sh`**: Optimized development build with parallel compilation
- **`scripts/build.sh`**: Standard production build script
- **`scripts/setup-dev.sh`**: Development environment setup
- **`scripts/run_tests.sh`**: Comprehensive test runner

### Quick Commands

```bash
# Enhanced development with hot reload (recommended)
pnpm run dev

# Fast development (optimized builds)
pnpm run dev:fast

# Setup environment
pnpm run setup

# Run all tests
pnpm run test:all

# Clean fast build
pnpm run clean:fast
```

For detailed information about all available scripts, see [`scripts/README.md`](scripts/README.md).

## 📚 Documentation

Comprehensive documentation is organized in the [`docs/`](docs/) directory:

- **[`docs/features/`](docs/features/)** - Feature documentation and integration guides
- **[`docs/testing/`](docs/testing/)** - Testing guides and test organization
- **[`docs/debugging/`](docs/debugging/)** - Debugging guides and troubleshooting
- **[`docs/development/`](docs/development/)** - Development summaries and project organization
  - **[Hot Reload Setup](docs/development/HOT_RELOAD_SETUP.md)** - Enhanced development environment guide
- **[`docs/README.md`](docs/README.md)** - Main documentation index

### Key Documentation Files

- **Installation Guide**: [`docs/INSTALLATION.md`](docs/INSTALLATION.md)
- **Testing Overview**: [`docs/testing/TEST_ORGANIZATION_SUMMARY.md`](docs/testing/TEST_ORGANIZATION_SUMMARY.md)
- **Feature Guides**: [`docs/features/`](docs/features/) - Duration extraction and other features
- **Debugging Tools**: [`docs/debugging/THUMBNAIL_DEBUGGING_GUIDE.md`](docs/debugging/THUMBNAIL_DEBUGGING_GUIDE.md)

## 🔧 Advanced Configuration

### FFmpeg Integration

The application automatically detects FFmpeg installation. For manual configuration:

1. **Download FFmpeg** from the official website
2. **Add to PATH** or specify the path in settings
3. **Restart the application** to detect the installation

### Custom Filename Templates

Customize download filenames using variables:

- `{title}`: Video title
- `{author}`: Video author/page name
- `{quality}`: Selected quality (e.g., "720p")
- `{date}`: Current date
- `{video_id}`: Facebook video ID

Example: `{title} - {author} - {quality}` → `"My Video - Page Name - 720p.mp4"`

## 🐛 Troubleshooting

### Common Issues

**"No video URLs found"**
- The video might be private or region-restricted
- Try using the view-source method
- Check if the URL is correct and accessible

**"FFmpeg not found"**
- Install FFmpeg and ensure it's in your PATH
- Or specify the FFmpeg path in settings

**Download fails with network error**
- Check your internet connection
- The video might have been deleted or made private
- Try again later as Facebook might be rate-limiting

**Application won't start**
- Ensure all prerequisites are installed
- Check the logs in the application data directory
- Try running from the command line to see error messages

### Getting Help

1. **Check the logs**: Available in the application data directory
2. **Search existing issues**: Check the GitHub issues page
3. **Create a new issue**: Provide detailed information about the problem

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Guidelines

1. **Code Style**: Follow Rust and TypeScript best practices
2. **Testing**: Add tests for new features
3. **Documentation**: Update documentation for changes
4. **Commits**: Use conventional commit messages

### Building from Source

```bash
# Clone and setup
git clone https://github.com/username/facebook-video-downloader.git
cd facebook-video-downloader
pnpm install

# Development
pnpm run dev            # Enhanced development with hot reload
pnpm run dev:fast       # Fast development server
pnpm tauri:dev          # Standard development server

# Testing
pnpm run test:all       # Run all tests
cargo test --workspace # Run Cargo tests
pnpm test              # Run specific tests

# Building
pnpm run build:script   # Production build script
pnpm tauri:build       # Standard Tauri build
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This tool is for educational and personal use only. Please respect Facebook's Terms of Service and copyright laws. Only download videos you have permission to download.

## 🙏 Acknowledgments

- **Tauri Team**: For the amazing desktop app framework
- **Yew Team**: For the Rust WebAssembly framework
- **FFmpeg**: For video processing capabilities
- **Tailwind CSS**: For the utility-first CSS framework

## 📊 Project Status

- ✅ Core video extraction
- ✅ Desktop application framework
- ✅ Modern UI with dark/light themes
- ✅ Batch download support
- ✅ Configuration management
- 🚧 Advanced video processing
- 🚧 Plugin system
- 📋 Mobile companion app

---

**Star this repository** if you find it useful! 🌟
