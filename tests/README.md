# Facebook Video Downloader Test Suite

This directory contains all test files for the Facebook Video Downloader project, organized by category and functionality.

## Directory Structure

### `/tests/core/`
Core functionality tests from the facebook-extractor-core crate:
- Pattern analysis and HTML parsing tests
- Core extraction algorithm tests
- Network and compression tests

### `/tests/extraction/`
Video extraction functionality tests:
- Facebook video extraction tests
- Enhanced extractor implementations
- Public video analysis tests

### `/tests/thumbnail/`
Thumbnail extraction and processing tests:
- Comprehensive thumbnail strategy tests
- Thumbnail download and fallback tests
- Header and CDN access tests

### `/tests/integration/`
Integration and end-to-end tests:
- Authentication bypass tests
- UI integration tests
- Full workflow tests

### `/tests/unit/`
Unit tests for specific components:
- URL validation and parsing tests
- Content parsing tests
- Individual function tests

### `/tests/debug/`
Debug tools and utilities:
- HTML parsing debug tools
- Response format analyzers
- Manual review utilities

## Running Tests

### Automated Test Runner (Recommended)
```bash
# Run all tests
./run_tests.sh

# Run tests in a specific category
./run_tests.sh --category thumbnail
./run_tests.sh --category extraction
./run_tests.sh --category unit
./run_tests.sh --category integration
./run_tests.sh --category core
./run_tests.sh --category debug

# Quick mode (essential tests only)
./run_tests.sh --quick

# Get help
./run_tests.sh --help
```

### Manual Test Execution
```bash
# Run specific tests
cargo run --bin test_comprehensive_thumbnail_strategies --features debug-tools
cargo run --bin test_improved_extraction --features debug-tools
cargo run --bin test_thumbnail_debug --features debug-tools

# Run all tests in a category manually
find tests/thumbnail -name "*.rs" -exec basename {} .rs \; | xargs -I {} cargo run --bin {} --features debug-tools
```

### Core Crate Tests
```bash
# Run core crate tests from the core directory
cd crates/facebook-extractor-core
cargo run --bin test_improved_extraction
cargo run --bin analyze_current_patterns
```

## Test Requirements

- Internet connection for Facebook access
- Valid Facebook video URLs (public videos recommended)
- Rust toolchain with tokio async runtime
- Required dependencies (see individual test files)

## Generated Files

Tests may generate various output files:
- HTML reports (*.html)
- JSON data files (*.json)
- Downloaded thumbnails (*.jpg, *.png, *.svg)
- Debug HTML files for inspection

## Test Documentation Standards

Each test file includes comprehensive documentation with:

### Required Documentation Sections
- **Purpose**: Clear description of what the test validates
- **Scope**: Detailed list of test coverage areas
- **Usage**: Command-line examples for running the test
- **Test Cases**: Enumerated list of specific test scenarios
- **Expected Behavior**: Description of successful test outcomes
- **Dependencies**: Required crates and features
- **Setup Requirements**: Prerequisites for running the test

### Function Documentation
- Main test functions have detailed docstrings
- Helper functions include parameter and return value documentation
- Complex logic includes inline comments explaining the approach

## Best Practices

### Test Organization
- Tests are categorized by functionality (thumbnail, extraction, etc.)
- Related tests are grouped in the same directory
- Test names clearly indicate their purpose and scope

### Error Handling
- Tests provide clear error messages for failures
- Specific guidance is given for different error scenarios
- Network and authentication issues are handled gracefully

### Output and Reporting
- Tests generate structured output for easy analysis
- HTML and JSON reports are created for comprehensive tests
- Generated files are documented and explained

## Contributing

When adding new tests:
1. **Placement**: Put tests in the appropriate category directory
2. **Documentation**: Add comprehensive documentation headers following the standards above
3. **Naming**: Use descriptive names that clearly indicate the test purpose
4. **Dependencies**: Document all required dependencies and features
5. **Integration**: Update Cargo.toml with new binary entries
6. **Testing**: Verify tests work with the automated test runner
7. **Documentation**: Update this README if adding new categories or significant functionality

### Adding a New Test
```bash
# 1. Create the test file in the appropriate directory
touch tests/category/test_new_feature.rs

# 2. Add comprehensive documentation header
# 3. Implement the test logic
# 4. Add binary entry to Cargo.toml
# 5. Test with the runner
./run_tests.sh --category category

# 6. Update documentation if needed
```
