#!/bin/bash

# Facebook Video Downloader Test Runner
# Comprehensive test execution script for all test categories

set -e  # Exit on any error

echo "🧪 Facebook Video Downloader Test Suite"
echo "======================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run a test with error handling
run_test() {
    local test_name="$1"
    local category="$2"
    
    echo -e "${BLUE}🔍 Running $category test: $test_name${NC}"
    
    if cargo run --bin "$test_name" --features debug-tools 2>/dev/null; then
        echo -e "${GREEN}✅ $test_name passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $test_name failed${NC}"
        return 1
    fi
}

# Function to run tests in a category
run_category() {
    local category="$1"
    shift
    local tests=("$@")
    
    echo ""
    echo -e "${YELLOW}📁 Running $category Tests${NC}"
    echo "$(printf '=%.0s' {1..50})"
    
    local passed=0
    local failed=0
    
    for test in "${tests[@]}"; do
        if run_test "$test" "$category"; then
            ((passed++))
        else
            ((failed++))
        fi
        echo ""
    done
    
    echo -e "${YELLOW}📊 $category Summary: ${GREEN}$passed passed${NC}, ${RED}$failed failed${NC}"
    echo ""
    
    return $failed
}

# Check if debug-tools feature is available
echo "🔧 Checking test environment..."
if ! cargo check --bin test_thumbnail_debug --features debug-tools >/dev/null 2>&1; then
    echo -e "${RED}❌ Error: debug-tools feature not available or compilation issues${NC}"
    echo "Some test files may have formatting issues. Try running individual tests manually."
    echo "Example: cargo run --bin test_thumbnail_debug --features debug-tools"
    exit 1
fi
echo -e "${GREEN}✅ Test environment ready${NC}"
echo ""

# Parse command line arguments
CATEGORY=""
QUICK_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --category)
            CATEGORY="$2"
            shift 2
            ;;
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --category CATEGORY  Run only tests in specified category"
            echo "                      (thumbnail, extraction, unit, integration, core, debug)"
            echo "  --quick             Run only essential tests"
            echo "  --help              Show this help message"
            echo ""
            echo "Categories:"
            echo "  thumbnail    - Thumbnail extraction and processing tests"
            echo "  extraction   - Video extraction functionality tests"
            echo "  unit         - Unit tests for specific components"
            echo "  integration  - Integration and end-to-end tests"
            echo "  core         - Core functionality tests"
            echo "  debug        - Debug tools and utilities"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Define test arrays
THUMBNAIL_TESTS=(
    "test_thumbnail_debug"
    "test_comprehensive_thumbnail_strategies"
    "test_all_thumbnail_cases"
    "test_cli_thumbnail_extraction"
    "test_thumbnail_download"
    "test_thumbnail_fallback_fix"
    "test_thumbnail_fix_verification"
    "test_thumbnail_with_headers"
)

EXTRACTION_TESTS=(
    "test_facebook_extraction"
    "test_facebook_extractor"
    "test_facebook_extractor_enhanced"
    "test_extraction_fix"
    "test_public_video_analysis"
)

UNIT_TESTS=(
    "test_url_validation"
    "test_facebook_url"
    "test_new_url_format"
    "test_specific_url"
    "test_title_parsing"
)

INTEGRATION_TESTS=(
    "test_auth_bypass"
    "test_extract_button"
)

CORE_TESTS=(
    "test_improved_extraction"
    "analyze_current_patterns"
    "analyze_facebook_html"
    "comprehensive_pattern_test"
    "test_no_compression"
)

DEBUG_TOOLS=(
    "debug_thumbnail_cli"
    "download_thumbnails_for_review"
    "debug_facebook_parsing"
    "debug_response_format"
)

# Quick mode - run only essential tests
if [ "$QUICK_MODE" = true ]; then
    echo -e "${YELLOW}⚡ Quick Mode: Running essential tests only${NC}"
    THUMBNAIL_TESTS=("test_thumbnail_debug")
    EXTRACTION_TESTS=("test_facebook_extraction")
    UNIT_TESTS=("test_url_validation")
    INTEGRATION_TESTS=("test_auth_bypass")
    CORE_TESTS=("test_improved_extraction")
    DEBUG_TOOLS=()
fi

# Track overall results
TOTAL_FAILED=0

# Run tests based on category
case "$CATEGORY" in
    "thumbnail")
        run_category "Thumbnail" "${THUMBNAIL_TESTS[@]}"
        TOTAL_FAILED=$?
        ;;
    "extraction")
        run_category "Extraction" "${EXTRACTION_TESTS[@]}"
        TOTAL_FAILED=$?
        ;;
    "unit")
        run_category "Unit" "${UNIT_TESTS[@]}"
        TOTAL_FAILED=$?
        ;;
    "integration")
        run_category "Integration" "${INTEGRATION_TESTS[@]}"
        TOTAL_FAILED=$?
        ;;
    "core")
        run_category "Core" "${CORE_TESTS[@]}"
        TOTAL_FAILED=$?
        ;;
    "debug")
        run_category "Debug" "${DEBUG_TOOLS[@]}"
        TOTAL_FAILED=$?
        ;;
    "")
        # Run all categories
        echo -e "${YELLOW}🚀 Running all test categories${NC}"
        echo ""
        
        run_category "Thumbnail" "${THUMBNAIL_TESTS[@]}"
        TOTAL_FAILED=$((TOTAL_FAILED + $?))
        
        run_category "Extraction" "${EXTRACTION_TESTS[@]}"
        TOTAL_FAILED=$((TOTAL_FAILED + $?))
        
        run_category "Unit" "${UNIT_TESTS[@]}"
        TOTAL_FAILED=$((TOTAL_FAILED + $?))
        
        run_category "Integration" "${INTEGRATION_TESTS[@]}"
        TOTAL_FAILED=$((TOTAL_FAILED + $?))
        
        run_category "Core" "${CORE_TESTS[@]}"
        TOTAL_FAILED=$((TOTAL_FAILED + $?))
        
        if [ "$QUICK_MODE" != true ]; then
            run_category "Debug Tools" "${DEBUG_TOOLS[@]}"
            TOTAL_FAILED=$((TOTAL_FAILED + $?))
        fi
        ;;
    *)
        echo -e "${RED}❌ Unknown category: $CATEGORY${NC}"
        echo "Valid categories: thumbnail, extraction, unit, integration, core, debug"
        exit 1
        ;;
esac

# Final summary
echo ""
echo "🎯 Test Suite Complete"
echo "======================"

if [ $TOTAL_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed successfully!${NC}"
    exit 0
else
    echo -e "${RED}❌ $TOTAL_FAILED test(s) failed${NC}"
    echo ""
    echo "💡 Troubleshooting tips:"
    echo "  - Check internet connectivity for Facebook access"
    echo "  - Verify Facebook video URLs are public and accessible"
    echo "  - Review individual test output for specific error details"
    echo "  - Run tests individually for more detailed debugging"
    exit 1
fi
