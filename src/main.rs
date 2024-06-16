use secrecy::ExposeSecret;

use fast_chat::{
    configuration::get_configuration,
    db::{init_db_connection, init_redis_connection},
    errors::{AppError, AppErrorType},
    startup::run,
    telemetry::init_subscriber,
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // config init
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    // tracing logger init
    init_subscriber()?;

    // db connection init
    let db_connection = init_db_connection(
        configuration
            .database
            .db_connection_string()
            .expose_secret(),
    )
    .await?;

    // redis connection init
    let redis_connection_manager = init_redis_connection(
        configuration
            .redis
            .redis_connection_string()
            .expose_secret(),
    )
    .await?;

    // server init
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|e| AppError::new(e.to_string(), AppErrorType::InternalServerError))?;

    Ok(run(listener, db_connection, redis_connection_manager, configuration.redis.redis_worker_config).await)
}
