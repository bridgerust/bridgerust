use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Metrics for Embex operations
#[derive(Clone)]
pub struct EmbexMetrics {
    pub inserts: Arc<AtomicU64>,
    pub searches: Arc<AtomicU64>,
    pub deletes: Arc<AtomicU64>,
    pub creates: Arc<AtomicU64>,
    pub deletes_collection: Arc<AtomicU64>,

    pub errors: Arc<AtomicU64>,
    pub retries: Arc<AtomicU64>,
    pub timeouts: Arc<AtomicU64>,

    pub insert_latency_ms: Arc<AtomicU64>,
    pub search_latency_ms: Arc<AtomicU64>,
    pub delete_latency_ms: Arc<AtomicU64>,
}

impl Default for EmbexMetrics {
    fn default() -> Self {
        Self {
            inserts: Arc::new(AtomicU64::new(0)),
            searches: Arc::new(AtomicU64::new(0)),
            deletes: Arc::new(AtomicU64::new(0)),
            creates: Arc::new(AtomicU64::new(0)),
            deletes_collection: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            retries: Arc::new(AtomicU64::new(0)),
            timeouts: Arc::new(AtomicU64::new(0)),
            insert_latency_ms: Arc::new(AtomicU64::new(0)),
            search_latency_ms: Arc::new(AtomicU64::new(0)),
            delete_latency_ms: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl EmbexMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_insert(&self, latency_ms: u64) {
        self.inserts.fetch_add(1, Ordering::Relaxed);
        self.insert_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    pub fn record_search(&self, latency_ms: u64) {
        self.searches.fetch_add(1, Ordering::Relaxed);
        self.search_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    pub fn record_delete(&self, latency_ms: u64) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
        self.delete_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    pub fn record_create(&self) {
        self.creates.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_delete_collection(&self) {
        self.deletes_collection.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry(&self) {
        self.retries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_timeout(&self) {
        self.timeouts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            inserts: self.inserts.load(Ordering::Relaxed),
            searches: self.searches.load(Ordering::Relaxed),
            deletes: self.deletes.load(Ordering::Relaxed),
            creates: self.creates.load(Ordering::Relaxed),
            deletes_collection: self.deletes_collection.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            retries: self.retries.load(Ordering::Relaxed),
            timeouts: self.timeouts.load(Ordering::Relaxed),
            insert_latency_ms: self.insert_latency_ms.load(Ordering::Relaxed),
            search_latency_ms: self.search_latency_ms.load(Ordering::Relaxed),
            delete_latency_ms: self.delete_latency_ms.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub inserts: u64,
    pub searches: u64,
    pub deletes: u64,
    pub creates: u64,
    pub deletes_collection: u64,
    pub errors: u64,
    pub retries: u64,
    pub timeouts: u64,
    pub insert_latency_ms: u64,
    pub search_latency_ms: u64,
    pub delete_latency_ms: u64,
}

impl MetricsSnapshot {
    /// Total number of operations performed
    pub fn total_operations(&self) -> u64 {
        self.inserts + self.searches + self.deletes + self.creates + self.deletes_collection
    }

    /// Total number of errors encountered
    pub fn total_errors(&self) -> u64 {
        self.errors
    }

    /// Error rate as a percentage (0.0 to 100.0)
    pub fn error_rate(&self) -> f64 {
        let total = self.total_operations();
        if total == 0 {
            return 0.0;
        }
        (self.errors as f64 / total as f64) * 100.0
    }

    pub fn avg_insert_latency_ms(&self) -> f64 {
        if self.inserts == 0 {
            return 0.0;
        }
        self.insert_latency_ms as f64 / self.inserts as f64
    }

    pub fn avg_search_latency_ms(&self) -> f64 {
        if self.searches == 0 {
            return 0.0;
        }
        self.search_latency_ms as f64 / self.searches as f64
    }

    pub fn avg_delete_latency_ms(&self) -> f64 {
        if self.deletes == 0 {
            return 0.0;
        }
        self.delete_latency_ms as f64 / self.deletes as f64
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let total_ops = self.inserts + self.searches + self.deletes;
        if total_ops == 0 {
            return 0.0;
        }
        let total_latency = self.insert_latency_ms + self.search_latency_ms + self.delete_latency_ms;
        total_latency as f64 / total_ops as f64
    }
}

pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(feature = "tracing-subscriber")]
pub fn init_tracing() {
    use tracing_subscriber::{
        fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    Registry::default()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();
}

#[cfg(not(feature = "tracing-subscriber"))]
pub fn init_tracing() {
    // No-op if tracing-subscriber feature is not enabled
    // Users can initialize tracing manually
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let metrics = EmbexMetrics::new();
        
        metrics.record_insert(10);
        metrics.record_search(20);
        metrics.record_error();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.inserts, 1);
        assert_eq!(snapshot.searches, 1);
        assert_eq!(snapshot.errors, 1);
        assert_eq!(snapshot.insert_latency_ms, 10);
        assert_eq!(snapshot.search_latency_ms, 20);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }
}

