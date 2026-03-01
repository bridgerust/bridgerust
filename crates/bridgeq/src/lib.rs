use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RetryBackoff {
    #[default]
    Immediate,
    Fixed(Duration),
    Linear {
        base: Duration,
        max: Duration,
    },
    Exponential {
        base: Duration,
        max: Duration,
    },
}

impl RetryBackoff {
    pub fn delay_for_attempt(self, attempt: u32) -> Duration {
        let attempt = attempt.max(1);

        match self {
            Self::Immediate => Duration::ZERO,
            Self::Fixed(delay) => delay,
            Self::Linear { base, max } => scaled_delay(base, u128::from(attempt), max),
            Self::Exponential { base, max } => {
                let shift = attempt.saturating_sub(1).min(63);
                let factor = 1u128 << shift;
                scaled_delay(base, factor, max)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueConfig {
    pub max_retries: u32,
    pub visibility_timeout: Duration,
    pub retry_backoff: RetryBackoff,
}

impl QueueConfig {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            visibility_timeout: Duration::from_secs(30),
            retry_backoff: RetryBackoff::Immediate,
        }
    }

    pub fn with_visibility_timeout(mut self, visibility_timeout: Duration) -> Self {
        self.visibility_timeout = visibility_timeout;
        self
    }

    pub fn with_retry_backoff(mut self, retry_backoff: RetryBackoff) -> Self {
        self.retry_backoff = retry_backoff;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueMessage<T> {
    id: u64,
    payload: T,
    attempts: u32,
}

impl<T> QueueMessage<T> {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn into_payload(self) -> T {
        self.payload
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueStats {
    pub ready: usize,
    pub delayed: usize,
    pub in_flight: usize,
    pub dead_letter: usize,
}

#[derive(Debug, Clone)]
struct DelayedMessage<T> {
    message: QueueMessage<T>,
    available_at: Instant,
}

#[derive(Debug, Clone)]
struct InFlightMessage<T> {
    message: QueueMessage<T>,
    lease_deadline: Instant,
}

#[derive(Debug, Clone)]
struct QueueState<T> {
    ready: VecDeque<QueueMessage<T>>,
    delayed: Vec<DelayedMessage<T>>,
    in_flight: HashMap<u64, InFlightMessage<T>>,
    dead_letter: Vec<QueueMessage<T>>,
}

impl<T> Default for QueueState<T> {
    fn default() -> Self {
        Self {
            ready: VecDeque::new(),
            delayed: Vec::new(),
            in_flight: HashMap::new(),
            dead_letter: Vec::new(),
        }
    }
}

/// In-memory queue engine designed as the Rust core for future Python/Node bindings.
///
/// Messages move through four states:
/// - ready: available for immediate consumption
/// - delayed: waiting for retry backoff
/// - in_flight: dequeued and currently leased for processing
/// - dead_letter: permanently failed after `max_retries`
#[derive(Debug)]
pub struct BridgeQueue<T> {
    state: Arc<Mutex<QueueState<T>>>,
    next_id: AtomicU64,
    config: QueueConfig,
}

impl<T: Clone> BridgeQueue<T> {
    pub fn new(max_retries: u32) -> Self {
        Self::with_config(QueueConfig::new(max_retries))
    }

    pub fn with_config(config: QueueConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(QueueState::default())),
            next_id: AtomicU64::new(1),
            config,
        }
    }

    pub fn config(&self) -> QueueConfig {
        self.config
    }

    pub fn enqueue(&self, payload: T) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let message = QueueMessage {
            id,
            payload,
            attempts: 0,
        };
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);
        state.ready.push_back(message);
        id
    }

    pub fn enqueue_batch(&self, payloads: impl IntoIterator<Item = T>) -> Vec<u64> {
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);

        let mut ids = Vec::new();
        for payload in payloads {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            ids.push(id);
            state.ready.push_back(QueueMessage {
                id,
                payload,
                attempts: 0,
            });
        }
        ids
    }

    /// Dequeues up to `batch_size` messages and marks them in-flight.
    pub fn dequeue(&self, batch_size: usize) -> Vec<QueueMessage<T>> {
        let size = batch_size.max(1);
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);

        let mut out = Vec::with_capacity(size);
        let lease_deadline = now
            .checked_add(self.config.visibility_timeout)
            .unwrap_or(now);

        for _ in 0..size {
            let Some(msg) = state.ready.pop_front() else {
                break;
            };
            state.in_flight.insert(
                msg.id,
                InFlightMessage {
                    message: msg.clone(),
                    lease_deadline,
                },
            );
            out.push(msg);
        }

        out
    }

    /// Acknowledges successful processing of a message.
    pub fn ack(&self, id: u64) -> bool {
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);
        state.in_flight.remove(&id).is_some()
    }

    /// Renews message lease from now using configured `visibility_timeout`.
    pub fn heartbeat(&self, id: u64) -> bool {
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);

        let Some(in_flight) = state.in_flight.get_mut(&id) else {
            return false;
        };

        in_flight.lease_deadline = now
            .checked_add(self.config.visibility_timeout)
            .unwrap_or(now);
        true
    }

    /// Extends current lease by `extra` duration.
    pub fn extend_lease(&self, id: u64, extra: Duration) -> bool {
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);

        let Some(in_flight) = state.in_flight.get_mut(&id) else {
            return false;
        };

        let base = in_flight.lease_deadline.max(now);
        in_flight.lease_deadline = base.checked_add(extra).unwrap_or(base);
        true
    }

    /// Marks processing as failed and schedules retry using configured backoff.
    pub fn nack(&self, id: u64) -> bool {
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);

        let Some(in_flight) = state.in_flight.remove(&id) else {
            return false;
        };

        self.retry_or_dead_letter_locked(&mut state, in_flight.message, now);
        true
    }

    /// Requeues a dead-letter message by id and resets attempts.
    pub fn requeue_dead_letter(&self, id: u64) -> bool {
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);

        let Some(index) = state.dead_letter.iter().position(|m| m.id == id) else {
            return false;
        };
        let mut msg = state.dead_letter.remove(index);
        msg.attempts = 0;
        state.ready.push_back(msg);
        true
    }

    pub fn stats(&self) -> QueueStats {
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);
        QueueStats {
            ready: state.ready.len(),
            delayed: state.delayed.len(),
            in_flight: state.in_flight.len(),
            dead_letter: state.dead_letter.len(),
        }
    }

    pub fn drain_dead_letter(&self) -> Vec<QueueMessage<T>> {
        let mut state = self.state.lock().expect("bridgeq mutex poisoned");
        let now = Instant::now();
        self.sweep_locked(&mut state, now);
        std::mem::take(&mut state.dead_letter)
    }

    fn sweep_locked(&self, state: &mut QueueState<T>, now: Instant) {
        self.release_delayed_locked(state, now);
        self.expire_in_flight_locked(state, now);
        self.release_delayed_locked(state, now);
    }

    fn release_delayed_locked(&self, state: &mut QueueState<T>, now: Instant) {
        if state.delayed.is_empty() {
            return;
        }

        let mut still_delayed = Vec::with_capacity(state.delayed.len());
        for delayed in state.delayed.drain(..) {
            if delayed.available_at <= now {
                state.ready.push_back(delayed.message);
            } else {
                still_delayed.push(delayed);
            }
        }
        state.delayed = still_delayed;
    }

    fn expire_in_flight_locked(&self, state: &mut QueueState<T>, now: Instant) {
        if state.in_flight.is_empty() {
            return;
        }

        let expired_ids: Vec<u64> = state
            .in_flight
            .iter()
            .filter_map(|(id, in_flight)| (in_flight.lease_deadline <= now).then_some(*id))
            .collect();

        for id in expired_ids {
            if let Some(in_flight) = state.in_flight.remove(&id) {
                self.retry_or_dead_letter_locked(state, in_flight.message, now);
            }
        }
    }

    fn retry_or_dead_letter_locked(
        &self,
        state: &mut QueueState<T>,
        mut message: QueueMessage<T>,
        now: Instant,
    ) {
        message.attempts += 1;

        if message.attempts > self.config.max_retries {
            state.dead_letter.push(message);
            return;
        }

        let delay = self
            .config
            .retry_backoff
            .delay_for_attempt(message.attempts);
        if delay.is_zero() {
            state.ready.push_back(message);
            return;
        }

        let available_at = now.checked_add(delay).unwrap_or(now);
        state.delayed.push(DelayedMessage {
            message,
            available_at,
        });
    }
}

fn scaled_delay(base: Duration, factor: u128, max: Duration) -> Duration {
    let base_ms = base.as_millis();
    let max_ms = max.as_millis();
    let candidate = base_ms.saturating_mul(factor).min(max_ms);
    Duration::from_millis(candidate.min(u64::MAX as u128) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_backoff_calculations_are_stable() {
        let fixed = RetryBackoff::Fixed(Duration::from_millis(200));
        assert_eq!(fixed.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(fixed.delay_for_attempt(5), Duration::from_millis(200));

        let linear = RetryBackoff::Linear {
            base: Duration::from_millis(100),
            max: Duration::from_millis(350),
        };
        assert_eq!(linear.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(linear.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(linear.delay_for_attempt(4), Duration::from_millis(350));

        let exp = RetryBackoff::Exponential {
            base: Duration::from_millis(100),
            max: Duration::from_millis(1_000),
        };
        assert_eq!(exp.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(exp.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(exp.delay_for_attempt(4), Duration::from_millis(800));
        assert_eq!(exp.delay_for_attempt(8), Duration::from_millis(1_000));
    }

    #[test]
    fn enqueue_dequeue_and_ack_flow() {
        let queue = BridgeQueue::new(3);
        let id = queue.enqueue("job-1".to_string());

        let messages = queue.dequeue(1);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id(), id);
        assert_eq!(messages[0].payload(), "job-1");

        assert!(queue.ack(id));
        assert!(!queue.ack(id));
        assert_eq!(
            queue.stats(),
            QueueStats {
                ready: 0,
                delayed: 0,
                in_flight: 0,
                dead_letter: 0
            }
        );
    }

    #[test]
    fn nack_requeues_until_dead_letter() {
        let queue = BridgeQueue::new(1);
        let id = queue.enqueue("job-2".to_string());

        let msg = queue.dequeue(1);
        assert_eq!(msg.len(), 1);
        assert!(queue.nack(id));

        // Second failure exceeds max_retries=1 and moves to dead letter.
        let msg = queue.dequeue(1);
        assert_eq!(msg.len(), 1);
        assert!(queue.nack(id));

        let stats = queue.stats();
        assert_eq!(stats.ready, 0);
        assert_eq!(stats.delayed, 0);
        assert_eq!(stats.in_flight, 0);
        assert_eq!(stats.dead_letter, 1);
    }

    #[test]
    fn requeue_dead_letter_message() {
        let queue = BridgeQueue::new(0);
        let id = queue.enqueue("job-3".to_string());

        let msg = queue.dequeue(1);
        assert_eq!(msg.len(), 1);
        assert!(queue.nack(id));
        assert_eq!(queue.stats().dead_letter, 1);

        assert!(queue.requeue_dead_letter(id));
        assert_eq!(queue.stats().dead_letter, 0);
        assert_eq!(queue.stats().ready, 1);
    }

    #[test]
    fn visibility_timeout_moves_messages_to_delayed_retries() {
        let queue = BridgeQueue::with_config(
            QueueConfig::new(2)
                .with_visibility_timeout(Duration::from_millis(20))
                .with_retry_backoff(RetryBackoff::Fixed(Duration::from_millis(40))),
        );

        let id = queue.enqueue("job-visibility".to_string());
        let first = queue.dequeue(1);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].attempts(), 0);

        std::thread::sleep(Duration::from_millis(30));
        let stats_after_timeout = queue.stats();
        assert_eq!(stats_after_timeout.in_flight, 0);
        assert_eq!(stats_after_timeout.delayed, 1);
        assert_eq!(stats_after_timeout.ready, 0);

        assert!(queue.dequeue(1).is_empty());

        std::thread::sleep(Duration::from_millis(50));
        let second = queue.dequeue(1);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].id(), id);
        assert_eq!(second[0].attempts(), 1);
    }

    #[test]
    fn heartbeat_renews_lease_before_timeout() {
        let queue = BridgeQueue::with_config(
            QueueConfig::new(2)
                .with_visibility_timeout(Duration::from_millis(50))
                .with_retry_backoff(RetryBackoff::Immediate),
        );

        let id = queue.enqueue("job-heartbeat".to_string());
        let first = queue.dequeue(1);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].attempts(), 0);

        std::thread::sleep(Duration::from_millis(35));
        assert!(queue.heartbeat(id));

        std::thread::sleep(Duration::from_millis(35));
        let mid_stats = queue.stats();
        assert_eq!(mid_stats.in_flight, 1);
        assert!(queue.dequeue(1).is_empty());

        std::thread::sleep(Duration::from_millis(35));
        let second = queue.dequeue(1);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].id(), id);
        assert_eq!(second[0].attempts(), 1);
    }

    #[test]
    fn extend_lease_pushes_deadline_forward() {
        let queue = BridgeQueue::with_config(
            QueueConfig::new(2)
                .with_visibility_timeout(Duration::from_millis(30))
                .with_retry_backoff(RetryBackoff::Immediate),
        );

        let id = queue.enqueue("job-extend".to_string());
        let first = queue.dequeue(1);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].attempts(), 0);

        assert!(queue.extend_lease(id, Duration::from_millis(80)));
        std::thread::sleep(Duration::from_millis(45));

        let mid_stats = queue.stats();
        assert_eq!(mid_stats.in_flight, 1);
        assert!(queue.dequeue(1).is_empty());

        std::thread::sleep(Duration::from_millis(75));
        let second = queue.dequeue(1);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].id(), id);
        assert_eq!(second[0].attempts(), 1);
    }
}
