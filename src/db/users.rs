use crate::errors::{AppError, AppErrorType};
use sqlx::PgPool;
use tracing::{instrument, Level};

use crate::crypt::hash::hash_password;
use derivative::{self, Derivative};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;
use validator::validate_email;

#[derive(Serialize, Deserialize)]
pub struct SessionUser<'a> {
    pub id: Uuid,
    pub email: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdate {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromRow, Derivative)]
#[derivative(Default)]
pub struct User {
    #[derivative(Default(value = "Uuid::new_v4()"))]
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
pub struct QueryUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for QueryUser {
    fn from(value: User) -> Self {
        QueryUser {
            id: value.id,
            name: value.name,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserInput {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl UserInput {
    pub fn validate_user_input(self) -> Result<Self, AppError> {
        let name = UserName::parse(self.name)?;
        let email = UserEmail::parse(self.email)?;

        let hash = hash_password(self.password).expect("Failed to hash password.");

        Ok(UserInput {
            name: name.inner(),
            email: email.inner(),
            password: hash,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct GetUserInput {
    email: String,
}

#[instrument(name = "Getting a user.", skip(pool), level = Level::INFO)]
pub async fn get_user(pool: &PgPool, id: &Uuid) -> Result<QueryUser, AppError> {
    sqlx::query_as::<_, QueryUser>(
        "SELECT id, email, name, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Get user error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

pub async fn get_full_user(pool: &PgPool, email: String) -> Result<User, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            AppError::new(
                "Get user error.".to_string(),
                AppErrorType::DatabaseError(e),
            )
        })
}

// skip user, context but include user.name
#[instrument(name = "Creating a user.", skip(pool, user), fields(user.name = %user.name), level = Level::INFO)]
pub async fn insert_user(pool: &PgPool, user: User) -> Result<QueryUser, AppError> {
    sqlx::query_as::<_, QueryUser>(
        r#"
        INSERT INTO users (id, email, name, password, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, email, created_at, updated_at"#,
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
pub async fn update_user(pool: &PgPool, user: UserUpdate) -> Result<QueryUser, AppError> {
    sqlx::query_as::<_, QueryUser>(
        r#"
        UPDATE users
        SET name = $2, updated_at = $3
        WHERE id = $1
        RETURNING id, name, email, created_at, updated_at
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

pub struct UserName(String);

pub struct UserEmail(String);

impl AsRef<str> for UserEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl UserEmail {
    pub fn parse(email: String) -> Result<UserEmail, AppError> {
        if validate_email(&email) {
            Ok(Self(email))
        } else {
            Err(AppError::new(
                "Failed to parse email.".to_string(),
                AppErrorType::ValidationError("Failed to parse email.".to_string()),
            ))
        }
    }

    pub fn inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for UserName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl UserName {
    pub fn parse(s: String) -> Result<UserName, AppError> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));
        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(AppError::new(
                format!("{} is not a valid subscriber name.", s),
                AppErrorType::ValidationError(format!("{} is not a valid subscriber name.", s)),
            ))
        } else {
            Ok(Self(s))
        }
    }

    pub fn inner(self) -> String {
        self.0
    }
}
