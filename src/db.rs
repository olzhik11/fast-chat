use std::collections::HashMap;

use futures_util::TryFutureExt;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tracing::instrument;
use uuid::Uuid;

use crate::errors::{AppError, AppErrorType};

pub async fn init_db_connection(connection_string: &str) -> Result<PgPool, AppError> {
    PgPool::connect_lazy(connection_string).map_err(|e| {
        AppError::new(
            "Database connection initialization error".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

pub async fn init_redis_connection(
    connection_string: &str,
) -> Result<redis::aio::ConnectionManager, AppError> {
    let con = redis::Client::open(connection_string).map_err(|e| {
        AppError::new(
            "Redis connection initialization error.".to_string(),
            AppErrorType::RedisError(e),
        )
    })?;

    con.get_connection_manager()
        .map_err(|e| {
            AppError::new(
                "Redis connection initialization error.".to_string(),
                AppErrorType::RedisError(e),
            )
        })
        .await
}

#[instrument(name = "Collecting rooms.", skip(pool))]
pub async fn collect_rooms(
    pool: &PgPool,
) -> Result<HashMap<Uuid, broadcast::Sender<Vec<u8>>>, AppError> {
    let map = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM rooms
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Failed to load rooms".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })?
    .into_iter()
    .map(|room| {
        let (sender, _receiver) = broadcast::channel(1000);
        (room, sender)
    })
    .collect::<HashMap<Uuid, broadcast::Sender<Vec<u8>>>>();

    Ok(map)
}
