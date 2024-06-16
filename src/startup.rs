use axum::routing::{on, MethodFilter};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tokio::{join, signal};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

use crate::configuration::RedisWorkerConfig;
use crate::errors::AppError;
use crate::graphql::handlers::{graphql, login, playground, register};
use crate::graphql::root::{create_schema, Schema};
use crate::service::worker::RedisWorker;
use crate::ws::ws::ws_handler;
use axum::{
    routing::{get, post},
    Router,
};

pub async fn health_check() -> Result<String, ()> {
    Ok("Hello World!".to_string())
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub schema: Arc<Schema>,
    pub chats: Arc<Mutex<HashMap<Uuid, broadcast::Sender<Vec<u8>>>>>,
}

impl AppState {
    pub fn initialize(pool: PgPool, redis: ConnectionManager) -> Result<Self, AppError> {
        let schema = Arc::new(create_schema());
        Ok(Self {
            pool,
            redis,
            schema,
            chats: Arc::new(Mutex::new(HashMap::default())),
        })
    }
}

pub async fn run(listener: TcpListener, db_pool: PgPool, redis: ConnectionManager, redis_worker_config: RedisWorkerConfig) {
    let app_state = AppState::initialize(db_pool.clone(), redis.clone()).expect("Failed to initialize app state.");

    let app = Router::new()
        .layer(CorsLayer::new().allow_credentials(true))
        .layer(TraceLayer::new_for_http())
        .route("/", get(health_check))
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/ws/:room", get(ws_handler))
        .route(
            "/graphql",
            on(MethodFilter::GET.or(MethodFilter::POST), graphql),
        )
        .route("/graphiql", get(playground))
        .with_state(app_state);

    let http = async {
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap()
    };

    let background = async {
        RedisWorker::new(redis.clone(), db_pool.clone(), redis_worker_config)
     };

    join!(http, background);

    ()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
