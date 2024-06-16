use bcrypt::{hash, verify};
use secrecy::{ExposeSecret, Secret};

use crate::errors::{AppError, AppErrorType};

pub fn hash_password(password: String) -> Result<String, AppError> {
    hash(password, 10).map_err(|e| {
        AppError::new(
            "Hash password error".to_string(),
            AppErrorType::EncodingError(e),
        )
    })
}

pub fn verify_password(password: Secret<String>, hash: &str) -> Result<(), AppError> {
    match verify(password.expose_secret().as_bytes(), hash) {
        Err(e) => Err(AppError::new(
            "Encryption error.".to_string(),
            AppErrorType::EncodingError(e),
        )),
        Ok(true) => Ok(()),
        Ok(false) => Err(AppError::new(
            "Password does not match.".to_string(),
            AppErrorType::PasswordWrongError,
        )),
    }
}
