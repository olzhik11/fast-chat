
use std::time::Duration;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tokio::time;
use crate::configuration::{RedisEventConfig, RedisWorkerConfig};

use super::stream::EventRedisStream;



pub struct RedisWorker {
    redis_connection_manager: ConnectionManager,
    db_pool: PgPool,
    config: RedisWorkerConfig,
}

impl RedisWorker {
    pub fn new(redis_connection_manager: ConnectionManager, db_pool: PgPool, config: RedisWorkerConfig) -> Self {
        RedisWorker {
            redis_connection_manager,
            db_pool,
            config,
        }
    }

    pub fn spawn_worker(self) {
        for RedisEventConfig { key, interval } in self.config.task_config {
            let mut stream = EventRedisStream::new(&key, self.redis_connection_manager.clone());
            let pg_pool = self.db_pool.clone();
            let mut interval = time::interval(Duration::from_secs(interval));
            tokio::spawn(async move {
                loop {
                    interval.tick().await;
                    if let Ok(events) = stream.read_stream().await {
                        for event in events {
                            let stream_clone = stream.clone();
                            let _pinned_boxed_future = stream_clone.process_event(&pg_pool, event).await;
                        }
                    }
                }
            });
        }
        ()
    }
}
