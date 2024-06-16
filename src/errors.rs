use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::BcryptError;
use config::ConfigError;
use juniper::{FieldError, IntoFieldError, Value};
use redis::RedisError;
use serde_json::json;
use sqlx;
use std::fmt;
use thiserror::Error;
use tracing_subscriber::util::TryInitError;

#[derive(Error, Debug)]
pub enum AppErrorType {
    #[error("Configuration error occured: ${0}")]
    ConfigurationError(ConfigError),

    #[error("Database error occured: {0}.")] // pass sqlx errors
    DatabaseError(sqlx::Error),

    #[error("Token encoding error occured: {0}.")]
    TokenEncodingError(jsonwebtoken::errors::Error),

    #[error("Password encoding error occured: {0}.")]
    EncodingError(BcryptError),

    #[error("Password is wrong.")]
    PasswordWrongError,

    #[error("Authorization error occured: {0}.")] // pass jwt errors
    AuthorizationError(String),

    #[error("User not found.")]
    UserNotFound,

    #[error("Validation error occured: {0}.")]
    ValidationError(String),

    #[error("Internal server error occured.")]
    InternalServerError,

    #[error("Redis error occured.")]
    RedisError(RedisError),

    #[error("Tracing Subscriber error occured. {0}")]
    TracingError(TryInitError),
}

#[derive(Error, Debug)]
pub struct AppError {
    message: Option<String>,
    error_type: AppErrorType,
}

impl AppError {
    pub fn default() -> Self {
        AppError {
            error_type: AppErrorType::InternalServerError,
            message: Some("Error.".to_string()),
        }
    }

    pub fn new(message: String, error_type: AppErrorType) -> Self {
        AppError {
            message: Some(message),
            error_type,
        }
    }
    fn message(&self) -> String {
        self.error_type.to_string()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError {
                error_type: AppErrorType::ConfigurationError(config_error),
                ..
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error{}", config_error),
            ),
            AppError {
                error_type: AppErrorType::DatabaseError(sqlx_error),
                ..
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error. {}", sqlx_error),
            ),
            AppError {
                error_type: AppErrorType::TokenEncodingError(jwt_error),
                ..
            } => (
                StatusCode::UNAUTHORIZED,
                format!("Unauthorized. {}", jwt_error),
            ),
            AppError {
                error_type: AppErrorType::EncodingError(bcrypt_error),
                ..
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Encryption error. {}", bcrypt_error),
            ),
            AppError {
                error_type: AppErrorType::PasswordWrongError,
                message,
            } => (
                StatusCode::UNAUTHORIZED,
                format!("Unauthorized. Credentials are wrong. {}", message.unwrap()),
            ),
            AppError {
                error_type: AppErrorType::AuthorizationError(error),
                ..
            } => (StatusCode::UNAUTHORIZED, format!("Unauthorized. {}", error)),
            AppError {
                error_type: AppErrorType::UserNotFound,
                message,
            } => (
                StatusCode::NOT_FOUND,
                format!("User not found. {}", message.unwrap()),
            ),
            AppError {
                error_type: AppErrorType::ValidationError(error),
                ..
            } => (
                StatusCode::BAD_REQUEST,
                format!("Validation error. {}", error),
            ),
            AppError {
                error_type: AppErrorType::InternalServerError,
                message,
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error. {}", message.unwrap()),
            ),
            AppError {
                error_type: AppErrorType::RedisError(error),
                ..
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Redis error. {}", error.to_string()),
            ),
            AppError {
                error_type: AppErrorType::TracingError(error),
                ..
            } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Tracing subscriber error. {}", error.to_string()),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        // its often easiest to implement `IntoResponse` by calling other implementations
        (status, body).into_response()
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message())
    }
}

impl IntoFieldError for AppError {
    fn into_field_error(self) -> FieldError {
        // improve extensions, add path, etc. for gql
        FieldError::new(self.message(), Value::Null)
    }
}
