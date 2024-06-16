use secrecy::{ExposeSecret, Secret};

use crate::errors::{AppError, AppErrorType};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub redis: RedisSettings,
    pub database: DatabaseSettings,
    pub token_max_age: i64,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize)]
pub struct RedisSettings {
    pub host: String,
    pub port: u16,
    pub redis_worker_config: RedisWorkerConfig,
}

#[derive(serde::Deserialize)]
pub struct RedisEventConfig {
    pub key: String,
    pub interval: u64,
} 

#[derive(serde::Deserialize)]
pub struct RedisWorkerConfig {
    pub task_config: Vec<RedisEventConfig>,
}

impl RedisSettings {
    pub fn redis_connection_string(&self) -> Secret<String> {
        Secret::new(format!("redis://{}:{}", self.host, self.port))
    }
}

impl DatabaseSettings {
    pub fn db_connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }
}

pub fn get_configuration() -> Result<Settings, AppError> {
    let settings = match config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()
    {
        Ok(settings) => settings,
        Err(_err) => {
            // Handle the error gracefully, e.g., return a default configuration or exit the program
            std::process::exit(1);
        }
    };

    settings.try_deserialize().map_err(|e| {
        AppError::new(
            "Configuration error.".to_string(),
            AppErrorType::ConfigurationError(e),
        )
    })
}
