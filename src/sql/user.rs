use crate::{
    errors::{AppError, AppErrorType},
    graphql::user::schema::{User, UserUpdate},
};
use sqlx::PgPool;
use tracing::{instrument, Level};

#[instrument(name = "Getting a user.", skip(pool), level = Level::INFO)]
pub async fn get_user(pool: &PgPool, email: &str) -> Result<User, AppError> {
    sqlx::query_as(
        "SELECT id, email, name, password, created_at, updated_at FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Insert user error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

// skip user, context but include user.name
#[instrument(name = "Creating a user.", skip(pool, user), fields(user.name = %user.name), level = Level::INFO)]
pub async fn insert_user(pool: &PgPool, user: User) -> Result<User, AppError> {
    sqlx::query_as(
        r#"
        INSERT INTO users (id, email, name, password, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, password, email, created_at, updated_at"#,
    )
    .bind(user.id)
    .bind(user.email)
    .bind(user.name)
    .bind(user.password)
    .bind(user.created_at)
    .bind(user.updated_at)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Insert user error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

#[instrument(name = "Updating a user.", skip(pool, user), fields(user.name = %user.name), level = Level::INFO)]
pub async fn update_user(pool: &PgPool, user: UserUpdate) -> Result<User, AppError> {
    sqlx::query_as(
        r#"
        UPDATE users
        SET name = $2, updated_at = $3
        WHERE id = $1
        RETURNING id, name, email, password, created_at, updated_at
        "#,
    )
    .bind(user.id)
    .bind(user.name)
    .bind(chrono::Utc::now())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Update user error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}
