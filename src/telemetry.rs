use crate::errors::{AppError, AppErrorType};
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_subscriber() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .compact()
        .with_level(true)
        .with_max_level(Level::DEBUG)
        .pretty()
        .finish()
        .try_init()
        .map_err(|e| {
            AppError::new(
                "Init of tracing subscriber error".to_string(),
                AppErrorType::TracingError(e),
            )
        })
}
