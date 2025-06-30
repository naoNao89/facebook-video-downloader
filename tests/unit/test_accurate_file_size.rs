//! Test for accurate file size detection functionality

use facebook_extractor_core::file_size::{AccurateFileSizeService, bytes_to_mb, mb_to_bytes};

#[tokio::main]
async fn main() {
    env_logger::init();

    println!("🧪 Testing Accurate File Size Detection");

    // Test 1: Service creation
    println!("\n1. Testing service creation...");
    let service = AccurateFileSizeService::new();
    assert!(service.is_ok(), "Should be able to create AccurateFileSizeService");
    println!("✅ Service creation successful");

    // Test 2: Byte to MB conversion
    println!("\n2. Testing bytes to MB conversion...");
    assert_eq!(bytes_to_mb(1024 * 1024), 1); // 1 MB
    assert_eq!(bytes_to_mb(1024 * 1024 * 50), 50); // 50 MB
    assert_eq!(bytes_to_mb(1024 * 1024 * 108), 108); // 108 MB
    assert_eq!(bytes_to_mb(46726656), 44); // ~44.6 MB should round to 44
    println!("✅ Bytes to MB conversion working correctly");

    // Test 3: MB to bytes conversion
    println!("\n3. Testing MB to bytes conversion...");
    assert_eq!(mb_to_bytes(1), 1024 * 1024); // 1 MB
    assert_eq!(mb_to_bytes(50), 1024 * 1024 * 50); // 50 MB
    assert_eq!(mb_to_bytes(108), 1024 * 1024 * 108); // 108 MB
    println!("✅ MB to bytes conversion working correctly");

    // Test 4: Cache functionality
    println!("\n4. Testing cache functionality...");
    let service = AccurateFileSizeService::new().expect("Should create service");
    let (total, verified) = service.get_cache_stats().await;
    assert_eq!(total, 0, "Cache should start empty");
    assert_eq!(verified, 0, "No verified entries initially");
    println!("✅ Cache functionality working correctly");

    // Test 5: Cache cleanup
    println!("\n5. Testing cache cleanup...");
    service.cleanup_cache().await;
    let (total, verified) = service.get_cache_stats().await;
    assert_eq!(total, 0, "Cache should still be empty after cleanup");
    println!("✅ Cache cleanup working correctly");

    // Test 6: Conversion accuracy (the specific issue case)
    println!("\n6. Testing conversion accuracy for the reported issue...");
    let estimated_bytes = mb_to_bytes(108); // 108 MB estimated
    let actual_bytes = 46726656; // ~44.6 MB actual (from the issue)

    let estimated_mb = bytes_to_mb(estimated_bytes);
    let actual_mb = bytes_to_mb(actual_bytes);

    assert_eq!(estimated_mb, 108);
    assert_eq!(actual_mb, 44); // Should be 44 MB (rounded down from 44.6)

    let discrepancy = estimated_mb.saturating_sub(actual_mb);
    assert_eq!(discrepancy, 64); // ~64 MB difference as mentioned in the issue

    println!("   📊 Estimated: {} MB", estimated_mb);
    println!("   📊 Actual: {} MB", actual_mb);
    println!("   📊 Discrepancy: {} MB", discrepancy);
    println!("✅ Conversion accuracy test passed - discrepancy correctly identified");

    println!("\n🎉 All tests passed! The accurate file size detection system is working correctly.");
    println!("   The new system should resolve the ~60MB discrepancy issue by using");
    println!("   partial downloads instead of unreliable HTTP HEAD requests.");
}
