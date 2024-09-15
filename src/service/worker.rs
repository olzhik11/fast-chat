use crate::configuration::{RedisEventConfig, RedisWorkerConfig};
use log::info;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;
use tracing::error;

use super::stream::EventRedisStream;

pub struct RedisWorker {
    redis_connection_manager: ConnectionManager,
    db_pool: PgPool,
    config: RedisWorkerConfig,
}

impl RedisWorker {
    pub fn new(
        redis_connection_manager: ConnectionManager,
        db_pool: PgPool,
        config: RedisWorkerConfig,
    ) -> Self {
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

                    let events = match stream.read_stream().await {
                        Ok(events) => events,
                        Err(e) => {
                            error!("Error reading from stream: {:?}", e);
                            continue; // Skip processing if reading fails
                        }
                    };

                    if events.is_empty() {
                        info!("No events for stream {}", key);
                        continue; // Skip processing if no events
                    }

                    let mut cleanup_events = Vec::<String>::new();
                    for (id, event) in events {
                        match stream.process_event(&pg_pool, event).await {
                            Ok(_) => {
                                cleanup_events.push(id);
                            }
                            Err(e) => error!("Error processing event {}", e),
                        }
                    }

                    let _ = stream.delete_events(cleanup_events).await;
                }
            });
        }
        ()
    }
}
