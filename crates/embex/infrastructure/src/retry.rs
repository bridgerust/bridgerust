use bridge_embex_core::error::{EmbexError, Result};
use std::time::Duration;
use tokio::time::sleep;

pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }
}

pub async fn retry_with_backoff<F, T>(config: &RetryConfig, mut f: F) -> Result<T>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                let is_retryable = e.is_retryable();
                let error_msg = e.to_string();
                
                if !is_retryable {
                    return Err(e);
                }

                last_error = Some(EmbexError::Other(anyhow::anyhow!("{}", error_msg)));

                if attempt < config.max_retries {
                    sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_secs_f64(
                            delay.as_secs_f64() * config.backoff_multiplier,
                        ),
                        config.max_delay,
                    );
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        EmbexError::Other(anyhow::anyhow!("Retry exhausted without error"))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let config = RetryConfig::default();
        let call_count = std::sync::Arc::new(AtomicU32::new(0));

        let result = retry_with_backoff(&config, {
            let count = call_count.clone();
            move || {
                let count = count.clone();
                Box::pin(async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok::<(), EmbexError>(())
                })
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_retries() {
        let config = RetryConfig::new(3);
        let call_count = std::sync::Arc::new(AtomicU32::new(0));

        let result = retry_with_backoff(&config, {
            let count = call_count.clone();
            move || {
                let count = count.clone();
                Box::pin(async move {
                    let attempts = count.fetch_add(1, Ordering::SeqCst);
                    if attempts < 2 {
                        Err(EmbexError::Connection("test".to_string()))
                    } else {
                        Ok::<(), EmbexError>(())
                    }
                })
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_on_non_retryable_error() {
        let config = RetryConfig::new(3);
        let call_count = std::sync::Arc::new(AtomicU32::new(0));

        let result: Result<()> = retry_with_backoff(&config, {
            let count = call_count.clone();
            move || {
                let count = count.clone();
                Box::pin(async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err(EmbexError::Validation("non-retryable".to_string()))
                })
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_exhausts_after_max_retries() {
        let config = RetryConfig::new(2);
        let call_count = std::sync::Arc::new(AtomicU32::new(0));

        let result: Result<()> = retry_with_backoff(&config, {
            let count = call_count.clone();
            move || {
                let count = count.clone();
                Box::pin(async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err(EmbexError::Connection("always fails".to_string()))
                })
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }
}

