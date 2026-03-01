use bridgeq::{QueueConfig, QueueStats};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Row};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresQueueMessage {
    pub id: u64,
    pub payload: String,
    pub attempts: u32,
}

#[derive(Debug, Error)]
pub enum PostgresQueueError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Invalid queue id value in database: {0}")]
    InvalidId(i64),
}

pub type Result<T> = std::result::Result<T, PostgresQueueError>;

#[derive(Debug, Clone)]
pub struct PostgresQueue {
    pool: PgPool,
    namespace: String,
    config: QueueConfig,
}

impl PostgresQueue {
    pub async fn connect(database_url: &str, namespace: &str, config: QueueConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Self::connect_with_pool(pool, namespace, config).await
    }

    pub async fn connect_with_pool(
        pool: PgPool,
        namespace: &str,
        config: QueueConfig,
    ) -> Result<Self> {
        let queue = Self {
            pool,
            namespace: namespace.to_string(),
            config,
        };
        queue.ensure_schema().await?;
        Ok(queue)
    }

    pub fn config(&self) -> QueueConfig {
        self.config
    }

    pub async fn enqueue(&self, payload: String) -> Result<u64> {
        self.reconcile().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO bridgeq_messages (namespace, payload, attempts, status, available_at, lease_deadline)
            VALUES ($1, $2, 0, 'ready', NOW(), NULL)
            RETURNING id
            "#,
        )
        .bind(&self.namespace)
        .bind(payload)
        .fetch_one(&self.pool)
        .await?;

        row_id_u64(&row)
    }

    pub async fn enqueue_batch(
        &self,
        payloads: impl IntoIterator<Item = String>,
    ) -> Result<Vec<u64>> {
        let mut ids = Vec::new();
        for payload in payloads {
            ids.push(self.enqueue(payload).await?);
        }
        Ok(ids)
    }

    pub async fn dequeue(&self, batch_size: usize) -> Result<Vec<PostgresQueueMessage>> {
        self.reconcile().await?;

        let size = batch_size.max(1) as i64;
        let lease_ms = duration_to_millis(self.config.visibility_timeout);

        let rows = sqlx::query(
            r#"
            WITH picked AS (
                SELECT id, payload, attempts
                FROM bridgeq_messages
                WHERE namespace = $1
                  AND status = 'ready'
                  AND available_at <= NOW()
                ORDER BY id
                FOR UPDATE SKIP LOCKED
                LIMIT $2
            )
            UPDATE bridgeq_messages AS q
            SET status = 'in_flight',
                lease_deadline = NOW() + ($3::bigint * INTERVAL '1 millisecond'),
                updated_at = NOW()
            FROM picked
            WHERE q.id = picked.id
            RETURNING q.id, picked.payload, picked.attempts
            "#,
        )
        .bind(&self.namespace)
        .bind(size)
        .bind(lease_ms)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_message).collect()
    }

    pub async fn ack(&self, id: u64) -> Result<bool> {
        self.reconcile().await?;
        let result = sqlx::query(
            r#"
            DELETE FROM bridgeq_messages
            WHERE namespace = $1
              AND id = $2
              AND status = 'in_flight'
            "#,
        )
        .bind(&self.namespace)
        .bind(id as i64)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn heartbeat(&self, id: u64) -> Result<bool> {
        self.reconcile().await?;
        let lease_ms = duration_to_millis(self.config.visibility_timeout);

        let result = sqlx::query(
            r#"
            UPDATE bridgeq_messages
            SET lease_deadline = NOW() + ($3::bigint * INTERVAL '1 millisecond'),
                updated_at = NOW()
            WHERE namespace = $1
              AND id = $2
              AND status = 'in_flight'
            "#,
        )
        .bind(&self.namespace)
        .bind(id as i64)
        .bind(lease_ms)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn extend_lease(&self, id: u64, extra: Duration) -> Result<bool> {
        self.reconcile().await?;
        let extra_ms = duration_to_millis(extra);

        let result = sqlx::query(
            r#"
            UPDATE bridgeq_messages
            SET lease_deadline = GREATEST(COALESCE(lease_deadline, NOW()), NOW()) + ($3::bigint * INTERVAL '1 millisecond'),
                updated_at = NOW()
            WHERE namespace = $1
              AND id = $2
              AND status = 'in_flight'
            "#,
        )
        .bind(&self.namespace)
        .bind(id as i64)
        .bind(extra_ms)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn nack(&self, id: u64) -> Result<bool> {
        self.reconcile().await?;

        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            SELECT attempts
            FROM bridgeq_messages
            WHERE namespace = $1
              AND id = $2
              AND status = 'in_flight'
            FOR UPDATE
            "#,
        )
        .bind(&self.namespace)
        .bind(id as i64)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row else {
            tx.commit().await?;
            return Ok(false);
        };

        let attempts: i32 = row.try_get("attempts")?;
        let next_attempts = attempts.saturating_add(1);

        self.apply_retry_transition(&mut tx, id as i64, next_attempts)
            .await?;
        tx.commit().await?;
        Ok(true)
    }

    pub async fn requeue_dead_letter(&self, id: u64) -> Result<bool> {
        self.reconcile().await?;
        let result = sqlx::query(
            r#"
            UPDATE bridgeq_messages
            SET status = 'ready',
                attempts = 0,
                available_at = NOW(),
                lease_deadline = NULL,
                updated_at = NOW()
            WHERE namespace = $1
              AND id = $2
              AND status = 'dead_letter'
            "#,
        )
        .bind(&self.namespace)
        .bind(id as i64)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn drain_dead_letter(&self) -> Result<Vec<PostgresQueueMessage>> {
        self.reconcile().await?;

        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query(
            r#"
            SELECT id, payload, attempts
            FROM bridgeq_messages
            WHERE namespace = $1
              AND status = 'dead_letter'
            ORDER BY id
            FOR UPDATE
            "#,
        )
        .bind(&self.namespace)
        .fetch_all(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM bridgeq_messages
            WHERE namespace = $1
              AND status = 'dead_letter'
            "#,
        )
        .bind(&self.namespace)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        rows.into_iter().map(row_to_message).collect()
    }

    pub async fn stats(&self) -> Result<QueueStats> {
        self.reconcile().await?;
        let row = sqlx::query(
            r#"
            SELECT
              COUNT(*) FILTER (WHERE status = 'ready')::bigint AS ready,
              COUNT(*) FILTER (WHERE status = 'delayed')::bigint AS delayed,
              COUNT(*) FILTER (WHERE status = 'in_flight')::bigint AS in_flight,
              COUNT(*) FILTER (WHERE status = 'dead_letter')::bigint AS dead_letter
            FROM bridgeq_messages
            WHERE namespace = $1
            "#,
        )
        .bind(&self.namespace)
        .fetch_one(&self.pool)
        .await?;

        Ok(QueueStats {
            ready: row.try_get::<i64, _>("ready")? as usize,
            delayed: row.try_get::<i64, _>("delayed")? as usize,
            in_flight: row.try_get::<i64, _>("in_flight")? as usize,
            dead_letter: row.try_get::<i64, _>("dead_letter")? as usize,
        })
    }

    async fn reconcile(&self) -> Result<()> {
        self.promote_due_delayed().await?;
        self.reclaim_expired_in_flight().await?;
        self.promote_due_delayed().await?;
        Ok(())
    }

    async fn promote_due_delayed(&self) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE bridgeq_messages
            SET status = 'ready',
                updated_at = NOW()
            WHERE namespace = $1
              AND status = 'delayed'
              AND available_at <= NOW()
            "#,
        )
        .bind(&self.namespace)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn reclaim_expired_in_flight(&self) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query(
            r#"
            SELECT id, attempts
            FROM bridgeq_messages
            WHERE namespace = $1
              AND status = 'in_flight'
              AND lease_deadline <= NOW()
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(&self.namespace)
        .fetch_all(&mut *tx)
        .await?;

        for row in rows {
            let id: i64 = row.try_get("id")?;
            let attempts: i32 = row.try_get("attempts")?;
            let next_attempts = attempts.saturating_add(1);
            self.apply_retry_transition(&mut tx, id, next_attempts)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn apply_retry_transition(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: i64,
        next_attempts: i32,
    ) -> Result<()> {
        if next_attempts > self.config.max_retries as i32 {
            sqlx::query(
                r#"
                UPDATE bridgeq_messages
                SET status = 'dead_letter',
                    attempts = $3,
                    available_at = NOW(),
                    lease_deadline = NULL,
                    updated_at = NOW()
                WHERE namespace = $1
                  AND id = $2
                "#,
            )
            .bind(&self.namespace)
            .bind(id)
            .bind(next_attempts)
            .execute(&mut **tx)
            .await?;
            return Ok(());
        }

        let delay = self
            .config
            .retry_backoff
            .delay_for_attempt(next_attempts as u32);
        if delay.is_zero() {
            sqlx::query(
                r#"
                UPDATE bridgeq_messages
                SET status = 'ready',
                    attempts = $3,
                    available_at = NOW(),
                    lease_deadline = NULL,
                    updated_at = NOW()
                WHERE namespace = $1
                  AND id = $2
                "#,
            )
            .bind(&self.namespace)
            .bind(id)
            .bind(next_attempts)
            .execute(&mut **tx)
            .await?;
            return Ok(());
        }

        let delay_ms = duration_to_millis(delay);
        sqlx::query(
            r#"
            UPDATE bridgeq_messages
            SET status = 'delayed',
                attempts = $3,
                available_at = NOW() + ($4::bigint * INTERVAL '1 millisecond'),
                lease_deadline = NULL,
                updated_at = NOW()
            WHERE namespace = $1
              AND id = $2
            "#,
        )
        .bind(&self.namespace)
        .bind(id)
        .bind(next_attempts)
        .bind(delay_ms)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn ensure_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bridgeq_messages (
                id BIGSERIAL PRIMARY KEY,
                namespace TEXT NOT NULL,
                payload TEXT NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL CHECK (status IN ('ready', 'delayed', 'in_flight', 'dead_letter')),
                available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                lease_deadline TIMESTAMPTZ NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_bridgeq_messages_ns_status_available
              ON bridgeq_messages(namespace, status, available_at, id);

            CREATE INDEX IF NOT EXISTS idx_bridgeq_messages_ns_status_lease
              ON bridgeq_messages(namespace, status, lease_deadline, id);
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn row_to_message(row: PgRow) -> Result<PostgresQueueMessage> {
    let raw_id: i64 = row.try_get("id")?;
    let id = raw_id
        .try_into()
        .map_err(|_| PostgresQueueError::InvalidId(raw_id))?;
    let attempts_i32: i32 = row.try_get("attempts")?;
    let attempts: u32 = attempts_i32.max(0) as u32;

    Ok(PostgresQueueMessage {
        id,
        payload: row.try_get("payload")?,
        attempts,
    })
}

fn row_id_u64(row: &PgRow) -> Result<u64> {
    let raw_id: i64 = row.try_get("id")?;
    raw_id
        .try_into()
        .map_err(|_| PostgresQueueError::InvalidId(raw_id))
}

fn duration_to_millis(duration: Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridgeq::RetryBackoff;

    #[test]
    fn duration_to_millis_saturates() {
        let huge = Duration::from_secs(u64::MAX);
        assert_eq!(duration_to_millis(huge), i64::MAX);
    }

    #[tokio::test]
    async fn config_is_carried() {
        let config = QueueConfig::new(5).with_retry_backoff(RetryBackoff::Linear {
            base: Duration::from_millis(50),
            max: Duration::from_millis(500),
        });
        let queue = PostgresQueue {
            pool: PgPoolOptions::new()
                .max_connections(1)
                .connect_lazy("postgres://postgres@127.0.0.1/postgres")
                .expect("pool"),
            namespace: "ns".to_string(),
            config,
        };
        assert_eq!(queue.config().max_retries, 5);
    }
}
