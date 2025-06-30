# Facebook Extraction Test Suite - Quick Reference

## Quick Start Commands

### Test a Single URL
```bash
cargo run --bin comprehensive_extraction_test --features debug-tools "https://www.facebook.com/watch?v=VIDEO_ID"
```

### Run All Tests
```bash
cargo run --bin comprehensive_extraction_test --features debug-tools -- --test-all
```

### Debug Mode
```bash
cargo run --bin comprehensive_extraction_test --features debug-tools -- --debug "FACEBOOK_URL"
```

## Command Options

| Option | Description | Example |
|--------|-------------|---------|
| `--test-all` | Run complete test suite | `-- --test-all` |
| `--test-patterns` | Test URL patterns only | `-- --test-patterns` |
| `--debug` | Enable debug mode | `-- --debug` |
| `--verbose` | Verbose logging | `-- --verbose` |
| `--save-debug` | Save debug files | `-- --save-debug` |
| `--test-private` | Include private video tests | `-- --test-private` |

## Expected Output

### ✅ Successful Extraction
```
🎯 SIMPLE VIDEO URL EXTRACTION TEST
✅ Video ID: 657552417147464
✅ Extraction succeeded!
📝 Title: Video Title
🎬 Available video URLs:
   1. 1080p Full HD - https://video-server.fbcdn.net/...
   2. 720p HD - https://video-server.fbcdn.net/...
```

### ❌ Failed Extraction
```
❌ Extraction failed: HTML parsing failed: All alternative public extraction methods failed
🔒 Authentication required - video may be private
```

## Common Issues

### Rate Limiting
**Symptoms:** "Rate limited" or "Blocking detected"
**Solution:** Wait 10-15 minutes before retrying

### Invalid URL
**Symptoms:** "Invalid Facebook URL format"
**Solution:** Check URL format matches supported patterns

### Network Issues
**Symptoms:** "Network connectivity issue"
**Solution:** Check internet connection and firewall settings

## Supported URL Formats

✅ **Supported:**
- `https://www.facebook.com/watch/?v=VIDEO_ID`
- `https://www.facebook.com/watch?v=VIDEO_ID`
- `https://www.facebook.com/reel/VIDEO_ID`
- `https://m.facebook.com/watch/?v=VIDEO_ID`
- `https://www.facebook.com/share/v/VIDEO_ID`

❌ **Limited/Unsupported:**
- `https://fb.watch/SHORT_CODE` (limited support)
- Private videos (requires authentication)
- Stories (not supported)

## Test Results Interpretation

### Success Indicators
- ✅ Green checkmarks
- 📝 Video title extracted
- 🎬 Multiple video URLs listed
- Success rate > 60%

### Warning Signs
- ⚠️ Yellow warnings
- 🔒 Authentication warnings
- Success rate < 50%

### Error Indicators
- ❌ Red X marks
- 🚫 Access denied messages
- 🌐 Network errors

## Debug Files

When using `--save-debug`:
- `debug_extraction_N.json` - Complete extraction results
- `debug_direct_extraction.html` - Raw Facebook HTML (if applicable)

**Cleanup:** `rm debug_*.json debug_*.html`

## Performance Expectations

| Test Type | Duration | Success Rate |
|-----------|----------|--------------|
| Single URL | 5-15s | 70-90% |
| Full Suite | 1-3min | 60-80% |
| Patterns | 10-30s | 95%+ |

## Troubleshooting Checklist

1. ✅ Check internet connectivity
2. ✅ Verify URL format is supported
3. ✅ Try with a known working URL first
4. ✅ Check if rate limited (wait if so)
5. ✅ Enable debug mode for detailed output
6. ✅ Review error messages for specific issues

## Integration Context

This test suite validates the same extraction functionality used by:
- 🖥️ Tauri desktop application
- 📱 Mobile application (if applicable)
- 🔧 facebook-extractor-core crate

**Purpose:** Ensure extraction works reliably for end users before deployment.

---

For complete documentation, see: `docs/FACEBOOK_EXTRACTION_TEST_SUITE.md`
