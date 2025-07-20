// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

//! Memory profiling utilities for performance analysis
//!
//! This module is behind the "profiling" feature.
//! It's not intended for external use.
//! It's not in 'benches` because it needs access to the private API.
//!

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A simple memory profiler that tracks allocations
pub struct MemoryProfiler;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK_USAGE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for MemoryProfiler {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size();
            let old_allocated = ALLOCATED.fetch_add(size, Ordering::Relaxed);
            let current_usage = old_allocated + size - DEALLOCATED.load(Ordering::Relaxed);

            // Update peak usage
            let mut peak = PEAK_USAGE.load(Ordering::Relaxed);
            while current_usage > peak {
                match PEAK_USAGE.compare_exchange_weak(
                    peak,
                    current_usage,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub allocated: usize,
    pub deallocated: usize,
    pub current_usage: usize,
    pub peak_usage: usize,
}

impl MemoryStats {
    /// Get current memory statistics
    pub fn current() -> Self {
        let allocated = ALLOCATED.load(Ordering::Relaxed);
        let deallocated = DEALLOCATED.load(Ordering::Relaxed);
        let peak_usage = PEAK_USAGE.load(Ordering::Relaxed);

        Self {
            allocated,
            deallocated,
            current_usage: allocated - deallocated,
            peak_usage,
        }
    }

    /// Reset all counters
    pub fn reset() {
        ALLOCATED.store(0, Ordering::Relaxed);
        DEALLOCATED.store(0, Ordering::Relaxed);
        PEAK_USAGE.store(0, Ordering::Relaxed);
    }
}

/// Macro to profile memory usage of a code block
#[macro_export]
macro_rules! profile_memory {
    ($name:expr, $code:block) => {{
        let start_stats = $crate::profiling::MemoryStats::current();
        let result = $code;
        let end_stats = $crate::profiling::MemoryStats::current();

        println!("{} - Memory Usage:", $name);
        println!(
            "  Allocated: {} bytes",
            end_stats.allocated - start_stats.allocated
        );
        println!("  Peak Usage: {} bytes", end_stats.peak_usage);
        println!("  Current Usage: {} bytes", end_stats.current_usage);

        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats() {
        MemoryStats::reset();
        let initial = MemoryStats::current();

        // Allocate some memory
        let _vec: Vec<u8> = vec![0; 1024];
        let after_alloc = MemoryStats::current();

        assert!(after_alloc.allocated > initial.allocated);
        assert!(after_alloc.current_usage > initial.current_usage);
    }
}
