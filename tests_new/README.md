# Test Organization

This directory contains the consolidated test suite for the Facebook Video Downloader project.

## Structure

```
tests_new/
├── integration/           # Cross-crate integration tests
│   ├── anti_blocking/     # Anti-blocking system tests
│   ├── batch_processing/  # Batch processing tests
│   ├── extraction/        # Video extraction tests
│   └── end_to_end/        # Full workflow tests
├── unit/                  # Unit tests by module
│   ├── extraction/        # Core extraction logic
│   ├── network/           # Network operations
│   ├── processing/        # Video processing
│   └── batch/             # Batch operations
├── fixtures/              # Test data and fixtures
└── common/                # Shared test utilities
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test integration
cargo test --test unit

# Run anti-blocking tests specifically
cargo test anti_blocking
```

## Test Categories

### Integration Tests
- **anti_blocking/**: Tests for IPv6 rotation, user-agent rotation, and network resilience
- **batch_processing/**: Tests for queue management and concurrent processing
- **extraction/**: Tests for video metadata extraction and stream detection
- **end_to_end/**: Complete workflow tests from URL to download

### Unit Tests
- **extraction/**: Core extraction logic, metadata parsing
- **network/**: HTTP client, download manager, anti-blocking components
- **processing/**: Compression, thumbnail generation, file operations
- **batch/**: Queue management, job scheduling, progress tracking

## Test Data

The `fixtures/` directory contains:
- Sample Facebook URLs for testing
- Mock response data
- Test video files
- Configuration templates
