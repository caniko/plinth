//! Shared TTL clock for cache actors.
//!
//! Extracted from the identical expiry blocks in `core_cache` and the
//! blog/portfolio/todo/activity brick caches: one population timestamp with
//! TTL-based expiration. Each actor owns a [`TtlClock`] and keeps only its
//! own map-clearing logic.

use std::time::{Duration, Instant};

/// Cache entry TTL — entries older than this are treated as expired.
pub const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// Maximum number of individually-cached items.
pub const MAX_ITEM_CACHE_SIZE: usize = 500;

/// Population timestamp with TTL expiry semantics.
#[derive(Default)]
pub struct TtlClock {
    populated_at: Option<Instant>,
}

impl TtlClock {
    /// Returns true if the cache has expired and should be cleared.
    pub fn is_expired(&self) -> bool {
        self.populated_at.is_some_and(|t| t.elapsed() > CACHE_TTL)
    }

    /// Mark the cache as freshly populated (first population wins).
    pub fn touch(&mut self) {
        if self.populated_at.is_none() {
            self.populated_at = Some(Instant::now());
        }
    }

    /// Stamp the cache as freshly populated, unconditionally.
    pub fn stamp(&mut self) {
        self.populated_at = Some(Instant::now());
    }

    /// Clear the population timestamp.
    pub fn reset(&mut self) {
        self.populated_at = None;
    }
}
