use axum::http::{header, Method};
use axum::routing::put;
use axum_server::tls_rustls::RustlsConfig;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::{net::SocketAddr, path::PathBuf};
use tokio::join;
use tokio::sync::broadcast;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::api::auth::{sign_in, sign_up, who_am_i};
use crate::api::messages::get_messages;
use crate::api::rooms::{create_room, get_rooms};
use crate::api::users::update_user;
use crate::configuration::RedisWorkerConfig;
use crate::db::collect_rooms;
use crate::errors::AppError;
use crate::service::worker::RedisWorker;
use crate::ws::ws::ws_handler;
use axum::{
    routing::{get, post},
    Router,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub rooms: Arc<Mutex<HashMap<Uuid, broadcast::Sender<Vec<u8>>>>>,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        redis: ConnectionManager,
        rooms: HashMap<Uuid, broadcast::Sender<Vec<u8>>>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            pool,
            redis,
            rooms: Arc::new(Mutex::new(rooms)),
        })
    }
}

pub async fn run(
    address: SocketAddr,
    db_pool: PgPool,
    redis: ConnectionManager,
    redis_worker_config: RedisWorkerConfig,
) -> Result<(), AppError> {
    let rooms = collect_rooms(&db_pool.clone()).await?;

    let app_state = AppState::new(db_pool.clone(), redis.clone(), rooms)
        .expect("Failed to initialize app state.");

    let api_routes = Router::new()
        .route("/rooms", get(get_rooms).post(create_room))
        .route("/rooms/:id/messages", get(get_messages))
        .route("/users", put(update_user))
        .route("/auth/whoami", get(who_am_i))
        .route("/auth/signin", post(sign_in))
        .route("/auth/signup", post(sign_up));

    let cors_layer = CorsLayer::new()
        .allow_origin(AllowOrigin::list(vec!["https://localhost:3000"
            .parse()
            .unwrap()]))
        .allow_methods(AllowMethods::list(vec![
            Method::GET,
            Method::PUT,
            Method::POST,
            Method::DELETE,
        ]))
        .allow_credentials(true)
        .allow_headers(AllowHeaders::list(vec![
            header::CONTENT_TYPE,
            header::COOKIE,
            header::SET_COOKIE,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
        ]))
        .expose_headers([header::SET_COOKIE, header::CONTENT_TYPE]);

    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certificates")
            .join("cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certificates")
            .join("key.pem"),
    )
    .await
    .unwrap();

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/ws/:room", get(ws_handler))
        .layer(cors_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let http = async {
        axum_server::bind_rustls(address, config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    };

    let background = async {
        RedisWorker::new(redis.clone(), db_pool.clone(), redis_worker_config).spawn_worker()
    };

    join!(http, background);

    Ok(())
}
