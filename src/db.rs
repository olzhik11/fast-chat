use futures_util::TryFutureExt;
use sqlx::PgPool;

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
