//! Profiling instrumentation for translation performance analysis (Rounds 70, 90)
//!
//! This module provides:
//! - Round 70: Lightweight profiling for major translation phases
//! - Round 90: Profile-Guided Optimization (PGO) instrumentation

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Profile counters for major translation phases (Round 70)
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
    /// Round 90: PGO - opcode distribution histogram
    pub opcode_histogram: Mutex<HashMap<String, u64>>,
    /// Round 90: PGO - branch prediction stats
    pub branch_hits: AtomicU64,
    pub branch_misses: AtomicU64,
    /// Round 90: PGO - cache performance
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
}

impl Default for TranslationProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationProfile {
    pub fn new() -> Self {
        Self {
            parse_time_ns: AtomicU64::new(0),
            translate_time_ns: AtomicU64::new(0),
            finalize_time_ns: AtomicU64::new(0),
            opcode_count: AtomicU64::new(0),
            function_count: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            opcode_histogram: Mutex::new(HashMap::with_capacity(64)),
            branch_hits: AtomicU64::new(0),
            branch_misses: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
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

    /// Round 90: Record opcode for PGO histogram
    #[inline]
    pub fn record_opcode(&self, op: &str) {
        #[cfg(feature = "pgo")]
        {
            if let Ok(mut hist) = self.opcode_histogram.try_lock() {
                *hist.entry(op.to_string()).or_insert(0) += 1;
            }
        }
        let _ = op;
    }

    /// Round 90: Record branch prediction hit
    #[inline]
    pub fn record_branch_hit(&self) {
        #[cfg(feature = "pgo")]
        self.branch_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Round 90: Record branch prediction miss
    #[inline]
    pub fn record_branch_miss(&self) {
        #[cfg(feature = "pgo")]
        self.branch_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current profile stats
    pub fn stats(&self) -> ProfileStats {
        ProfileStats {
            parse_time_ms: self.parse_time_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            translate_time_ms: self.translate_time_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            finalize_time_ms: self.finalize_time_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            opcode_count: self.opcode_count.load(Ordering::Relaxed),
            function_count: self.function_count.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            branch_hit_rate: self.calculate_branch_hit_rate(),
        }
    }

    fn calculate_branch_hit_rate(&self) -> f64 {
        let hits = self.branch_hits.load(Ordering::Relaxed);
        let misses = self.branch_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Round 90: Get opcode histogram for PGO
    pub fn opcode_histogram(&self) -> HashMap<String, u64> {
        self.opcode_histogram.lock().unwrap().clone()
    }

    /// Round 90: Get top opcodes by frequency
    pub fn top_opcodes(&self, n: usize) -> Vec<(String, u64)> {
        let mut histogram: Vec<_> = self.opcode_histogram().into_iter().collect();
        histogram.sort_by(|a, b| b.1.cmp(&a.1));
        histogram.into_iter().take(n).collect()
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
    pub allocation_count: u64,
    pub branch_hit_rate: f64,
}

impl ProfileStats {
    pub fn total_time_ms(&self) -> f64 {
        self.parse_time_ms + self.translate_time_ms + self.finalize_time_ms
    }
}

/// Global profile instance (lazy initialization)
use once_cell::sync::Lazy;
pub static PROFILE: Lazy<TranslationProfile> = Lazy::new(TranslationProfile::new);

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

/// Round 90: Macro for recording opcode in PGO histogram
#[macro_export]
macro_rules! pgo_record_opcode {
    ($op:expr) => {
        #[cfg(feature = "pgo")]
        $crate::translator::profiling::PROFILE.record_opcode($op);
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
    eprintln!("Allocations:    {:>8}", stats.allocation_count);
    if stats.opcode_count > 0 {
        eprintln!(
            "Time/op:        {:>8.3} µs",
            stats.total_time_ms() * 1000.0 / stats.opcode_count as f64
        );
    }

    // Round 90: PGO stats
    #[cfg(feature = "pgo")]
    {
        eprintln!("\n=== PGO Statistics ===");
        eprintln!("Branch hit rate: {:.1}%", stats.branch_hit_rate * 100.0);
        eprintln!("\nTop 10 opcodes:");
        for (i, (op, count)) in PROFILE.top_opcodes(10).iter().enumerate() {
            let percentage = if stats.opcode_count > 0 {
                *count as f64 / stats.opcode_count as f64 * 100.0
            } else {
                0.0
            };
            eprintln!("  {}. {:20} {:>8} ({:5.2}%)", i + 1, op, count, percentage);
        }
    }
}
