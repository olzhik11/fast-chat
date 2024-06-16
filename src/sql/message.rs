use sqlx::{postgres::PgQueryResult, PgPool};
use tracing::{instrument, Level};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppErrorType},
    ws::schema::{MessageStatus, SocketMessageContent},
};

#[instrument(name = "Send message", skip(pool), level = Level::INFO)]
pub async fn insert_message(
    pool: &PgPool,
    message: SocketMessageContent,
) -> Result<PgQueryResult, AppError> {
    sqlx::query(
        r#"
        INSERT INTO messages (id, content, author, room, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(message.id)
    .bind(message.content)
    .bind(message.author.id)
    .bind(message.room)
    .bind(message.status)
    .bind(message.created_at)
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Insert message error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

#[instrument(name = "Marking messages as seen.", skip(pool), level = Level::INFO)]
pub async fn mark_as_seen(pool: &PgPool, ids: Vec<Uuid>) -> Result<PgQueryResult, AppError> {
    sqlx::query(
        r#"
        UPDATE messages
        SET status = $1
        WHERE id = ANY($2)
        "#,
    )
    .bind(MessageStatus::Seen)
    .bind(ids)
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Mark-as-seen messages error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

#[instrument(name = "Getting a message", skip(pool), level = Level::INFO)]
pub async fn get_message(pool: &PgPool, id: Uuid) -> Result<PgQueryResult, AppError> {
    sqlx::query(
        r#"
        SELECT * FROM messages m INNER JOIN users u ON m.author = u.id WHERE m.id = $1 LIMIT 1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Get a message error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

#[instrument(name = "Getting messages", skip(pool), level = Level::INFO)]
pub async fn get_messages(pool: &PgPool, id: Uuid) -> Result<PgQueryResult, AppError> {
    sqlx::query(
        r#"
        SELECT * FROM messages
        WHERE room = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Get messageßs error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

#[instrument(name = "Deleting messages", skip(pool), level = Level::INFO)]
pub async fn delete_messages(pool: &PgPool, ids: Vec<Uuid>) -> Result<PgQueryResult, AppError> {
    sqlx::query(
        r#"
        DELETE FROM messages
        WHERE id = ANY($1) 
        "#,
    )
    .bind(ids)
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Delete messages error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

#[instrument(name = "Update message", skip(pool), level = Level::INFO)]
pub async fn update_message(
    pool: &PgPool,
    id: Uuid,
    content: String,
) -> Result<PgQueryResult, AppError> {
    sqlx::query(
        r#"
        UPDATE messages
        SET content = $1
        WHERE id = $2
        "#,
    )
    .bind(content)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Update message error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}
