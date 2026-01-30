// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileCopyrightText: 2026 Knitli Inc.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Query Cache Integration Example
//!
//! This example demonstrates how to use the query result cache
//! to optimize D1 database queries and reduce latency.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example query_cache_example --features caching
//! ```

#[cfg(feature = "caching")]
use thread_flow::cache::{CacheConfig, QueryCache};
#[cfg(feature = "caching")]
use thread_services::conversion::compute_content_fingerprint;

#[tokio::main]
async fn main() {
    println!("🗃️  Thread Query Cache Example\n");

    #[cfg(feature = "caching")]
    run_cache_example().await;

    #[cfg(not(feature = "caching"))]
    println!(
        "⚠️  Caching feature not enabled. Run with: cargo run --example query_cache_example --features caching"
    );
}

#[cfg(feature = "caching")]
async fn run_cache_example() {
    println!("📋 Creating cache with 1000 entry limit, 5 minute TTL...");
    let cache: QueryCache<String, Vec<String>> = QueryCache::new(CacheConfig {
        max_capacity: 1000,
        ttl_seconds: 300,
    });
    println!("✅ Cache created\n");

    // Example 1: Symbol query caching
    println!("--- Example 1: Symbol Query Caching ---\n");

    let code1 = "fn main() { println!(\"Hello\"); }";
    let fingerprint1 = compute_content_fingerprint(code1);
    let fp1_str = format!("{:?}", fingerprint1); // Convert to string for cache key

    println!("🔍 First query for fingerprint {}", &fp1_str[..16]);
    let symbols1 = cache
        .get_or_insert(fp1_str.clone(), || async {
            println!("   💾 Cache miss - querying D1 database...");
            simulate_d1_query().await
        })
        .await;
    println!("   ✅ Retrieved {} symbols", symbols1.len());

    println!("\n🔍 Second query for same fingerprint");
    let symbols2 = cache
        .get_or_insert(fp1_str.clone(), || async {
            println!("   💾 Cache miss - querying D1 database...");
            simulate_d1_query().await
        })
        .await;
    println!(
        "   ⚡ Cache hit! Retrieved {} symbols (no D1 query)",
        symbols2.len()
    );

    // Example 2: Cache statistics
    println!("\n--- Example 2: Cache Statistics ---\n");

    let stats = cache.stats().await;
    println!("📊 Cache Statistics:");
    println!("   Total lookups: {}", stats.total_lookups);
    println!("   Cache hits:    {}", stats.hits);
    println!("   Cache misses:  {}", stats.misses);
    println!("   Hit rate:      {:.1}%", stats.hit_rate());
    println!("   Miss rate:     {:.1}%", stats.miss_rate());

    // Example 3: Multiple file scenario
    println!("\n--- Example 3: Batch Processing with Cache ---\n");

    let files = vec![
        "fn add(a: i32, b: i32) -> i32 { a + b }",
        "fn subtract(a: i32, b: i32) -> i32 { a - b }",
        "fn multiply(a: i32, b: i32) -> i32 { a * b }",
    ];

    println!("📁 Processing {} files...", files.len());
    for (i, code) in files.iter().enumerate() {
        let fp = compute_content_fingerprint(code);
        let fp_str = format!("{:?}", fp);

        let symbols = cache
            .get_or_insert(fp_str, || async {
                println!("   File {}: Cache miss - querying D1", i + 1);
                simulate_d1_query().await
            })
            .await;

        println!("   File {}: Retrieved {} symbols", i + 1, symbols.len());
    }

    // Example 4: Re-processing (simulating code re-analysis)
    println!("\n--- Example 4: Re-analysis (Cache Benefit) ---\n");

    println!("🔄 Re-analyzing same files (simulating incremental update)...");
    for (i, code) in files.iter().enumerate() {
        let fp = compute_content_fingerprint(code);
        let fp_str = format!("{:?}", fp);

        let symbols = cache
            .get_or_insert(fp_str, || async {
                println!("   File {}: Cache miss - querying D1", i + 1);
                simulate_d1_query().await
            })
            .await;

        println!(
            "   File {}: ⚡ Cache hit! {} symbols (no D1 query)",
            i + 1,
            symbols.len()
        );
    }

    let final_stats = cache.stats().await;
    println!("\n📊 Final Cache Statistics:");
    println!("   Total lookups: {}", final_stats.total_lookups);
    println!(
        "   Cache hits:    {} ({}%)",
        final_stats.hits,
        final_stats.hit_rate() as i32
    );
    println!(
        "   Cache misses:  {} ({}%)",
        final_stats.misses,
        final_stats.miss_rate() as i32
    );

    // Calculate savings
    let d1_query_time_ms = 75.0; // Average D1 query time
    let cache_hit_time_ms = 0.001; // Cache lookup time
    let total_queries = final_stats.total_lookups as f64;
    let hits = final_stats.hits as f64;

    let time_without_cache = total_queries * d1_query_time_ms;
    let time_with_cache =
        (final_stats.misses as f64 * d1_query_time_ms) + (hits * cache_hit_time_ms);
    let savings_ms = time_without_cache - time_with_cache;
    let speedup = time_without_cache / time_with_cache;

    println!("\n💰 Performance Savings:");
    println!("   Without cache: {:.1}ms", time_without_cache);
    println!("   With cache:    {:.1}ms", time_with_cache);
    println!(
        "   Savings:       {:.1}ms ({:.1}x speedup)",
        savings_ms, speedup
    );

    println!("\n✅ Cache example complete!");
}

#[cfg(feature = "caching")]
async fn simulate_d1_query() -> Vec<String> {
    // Simulate D1 query latency (50-100ms)
    tokio::time::sleep(tokio::time::Duration::from_millis(75)).await;

    // Return mock symbols
    vec![
        "main".to_string(),
        "Config".to_string(),
        "process".to_string(),
    ]
}
