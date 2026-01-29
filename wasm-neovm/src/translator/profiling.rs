//! Profiling instrumentation for translation performance analysis (Round 70)
//!
//! This module provides lightweight profiling capabilities for identifying
//! hot paths during WASM to NeoVM translation.

use std::sync::atomic::{AtomicU64, Ordering};

/// Profile counters for major translation phases
pub struct TranslationProfile {
    /// Time spent in parsing (nanoseconds)
    pub parse_time_ns: AtomicU64,
    /// Time spent in function translation (nanoseconds)
    pub translate_time_ns: AtomicU64,
    /// Time spent in finalization (nanoseconds)
    pub finalize_time_ns: AtomicU64,
    /// Number of opcodes translated
    pub opcode_count: AtomicU64,
    /// Number of functions translated
    pub function_count: AtomicU64,
    /// Memory allocations (approximate)
    pub allocation_count: AtomicU64,
}

impl Default for TranslationProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationProfile {
    pub const fn new() -> Self {
        Self {
            parse_time_ns: AtomicU64::new(0),
            translate_time_ns: AtomicU64::new(0),
            finalize_time_ns: AtomicU64::new(0),
            opcode_count: AtomicU64::new(0),
            function_count: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
        }
    }

    /// Record parse time
    #[inline]
    pub fn record_parse(&self, ns: u64) {
        self.parse_time_ns.fetch_add(ns, Ordering::Relaxed);
    }

    /// Record translation time
    #[inline]
    pub fn record_translate(&self, ns: u64) {
        self.translate_time_ns.fetch_add(ns, Ordering::Relaxed);
    }

    /// Record finalization time
    #[inline]
    pub fn record_finalize(&self, ns: u64) {
        self.finalize_time_ns.fetch_add(ns, Ordering::Relaxed);
    }

    /// Increment opcode count
    #[inline]
    pub fn increment_opcodes(&self, count: u64) {
        self.opcode_count.fetch_add(count, Ordering::Relaxed);
    }

    /// Increment function count
    #[inline]
    pub fn increment_functions(&self, count: u64) {
        self.function_count.fetch_add(count, Ordering::Relaxed);
    }

    /// Get current profile stats
    pub fn stats(&self) -> ProfileStats {
        ProfileStats {
            parse_time_ms: self.parse_time_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            translate_time_ms: self.translate_time_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            finalize_time_ms: self.finalize_time_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            opcode_count: self.opcode_count.load(Ordering::Relaxed),
            function_count: self.function_count.load(Ordering::Relaxed),
        }
    }
}

/// Profile statistics snapshot
#[derive(Debug, Clone, Copy)]
pub struct ProfileStats {
    pub parse_time_ms: f64,
    pub translate_time_ms: f64,
    pub finalize_time_ms: f64,
    pub opcode_count: u64,
    pub function_count: u64,
}

impl ProfileStats {
    pub fn total_time_ms(&self) -> f64 {
        self.parse_time_ms + self.translate_time_ms + self.finalize_time_ms
    }
}

/// Global profile instance (lazy initialization)
pub static PROFILE: TranslationProfile = TranslationProfile::new();

/// Scoped timer for measuring operation duration
#[cfg(feature = "profile")]
pub struct ScopeTimer<'a> {
    start: std::time::Instant,
    counter: &'a AtomicU64,
}

#[cfg(feature = "profile")]
impl<'a> ScopeTimer<'a> {
    #[inline]
    pub fn new(counter: &'a AtomicU64) -> Self {
        Self {
            start: std::time::Instant::now(),
            counter,
        }
    }
}

#[cfg(feature = "profile")]
impl<'a> Drop for ScopeTimer<'a> {
    #[inline]
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_nanos() as u64;
        self.counter.fetch_add(elapsed, Ordering::Relaxed);
    }
}

/// No-op timer when profiling is disabled
#[cfg(not(feature = "profile"))]
pub struct ScopeTimer;

#[cfg(not(feature = "profile"))]
impl ScopeTimer {
    #[inline(always)]
    pub fn new(_counter: &AtomicU64) -> Self {
        Self
    }
}

/// Macro for scoped profiling (only evaluates when feature enabled)
#[macro_export]
macro_rules! profile_scope {
    ($phase:ident) => {
        #[cfg(feature = "profile")]
        let _timer = $crate::translator::profiling::ScopeTimer::new(
            &$crate::translator::profiling::PROFILE.$phase,
        );
    };
}

/// Print profile statistics to stderr
pub fn print_stats() {
    let stats = PROFILE.stats();
    eprintln!("=== Translation Profile ===");
    eprintln!("Parse time:     {:>8.3} ms", stats.parse_time_ms);
    eprintln!("Translate time: {:>8.3} ms", stats.translate_time_ms);
    eprintln!("Finalize time:  {:>8.3} ms", stats.finalize_time_ms);
    eprintln!("Total time:     {:>8.3} ms", stats.total_time_ms());
    eprintln!("Opcodes:        {:>8}", stats.opcode_count);
    eprintln!("Functions:      {:>8}", stats.function_count);
    if stats.opcode_count > 0 {
        eprintln!(
            "Time/op:        {:>8.3} µs",
            stats.total_time_ms() * 1000.0 / stats.opcode_count as f64
        );
    }
}
