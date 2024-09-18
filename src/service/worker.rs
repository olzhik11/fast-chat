use crate::configuration::{RedisEventConfig, RedisWorkerConfig};
use tracing::debug;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;
use tracing::error;

use super::stream::Stream;

pub struct EventsWorker {
    redis_connection_manager: ConnectionManager,
    db_pool: PgPool,
    config: RedisWorkerConfig,
}

impl EventsWorker {
    pub fn new(
        redis_connection_manager: ConnectionManager,
        db_pool: PgPool,
        config: RedisWorkerConfig,
    ) -> Self {
        EventsWorker {
            redis_connection_manager,
            db_pool,
            config,
        }
    }

    pub fn spawn(self) {
        for RedisEventConfig { key, interval } in self.config.task_config {
            let mut stream = Stream::new(&key, self.redis_connection_manager.clone());
            let pg_pool = self.db_pool.clone();
            let mut interval = time::interval(Duration::from_secs(interval));

            tokio::spawn(async move {
                loop {
                    interval.tick().await;

                    let events = match stream.read().await {
                        Ok(events) => events,
                        Err(_) => {
                            error!("Error reading from stream");
                            continue; // Skip processing if reading fails
                        }
                    };

                    if events.is_empty() {
                        debug!("No events for stream {}", key);
                        continue; // Skip processing if no events
                    }

                    let mut cleanup_events = Vec::<String>::new();
                    for (id, event) in events {
                        match stream.process_event(&pg_pool, event).await {
                            Ok(_) => {
                                cleanup_events.push(id);
                            }
                            Err(_) => error!("Error processing event"),
                        }
                    }

                    let _ = stream.delete_events(cleanup_events).await;
                }
            });
        }
        ()
    }
}
