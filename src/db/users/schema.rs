use crate::errors::{AppError, AppErrorType};

use crate::crypt::hash::hash_password;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;
use validator::validate_email;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdate {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

impl From<UserInput> for User {
    fn from(value: UserInput) -> Self {
        Self {
            id: Uuid::new_v4(),
            email: value.email,
            name: value.name,
            password: value.password,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),

        }
    }
}
impl UserInput {
    pub fn parse_user_input(self) -> Result<Self, AppError> {
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
