use unicode_segmentation::UnicodeSegmentation;
use validator::validate_email;

use crate::errors::{AppError, AppErrorType};

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
