// use std::net::TcpListener;

// use once_cell::sync::Lazy;
// use reqwest::StatusCode;
// use secrecy::ExposeSecret;
// use sqlx::PgPool;

// use zero2prod::{
//     configuration::get_configuration,
//     startup::run,
//     telemetry::{get_subscriber, init_subscriber},
// };

// static TRACING: Lazy<()> = Lazy::new(|| {
//     let subscriber = get_subscriber();
//     init_subscriber(subscriber);
// });

// pub struct TestApp {
//     pub address: String,
//     pub db_pool: PgPool,
// }

// async fn spawn_app() -> TestApp {
//     let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind port.");
//     let port = listener.local_addr().unwrap().port();
//     let address = format!("http://127.0.0.1:{}", port);

//     Lazy::force(&TRACING);

//     let configuration = get_configuration().expect("Failed to get configuration.");
//     let db_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
//         .await
//         .expect("Failed to connect to database.");
//     let server = run(listener, db_pool.clone()).expect("Failed to bind address.");
//     let _ = tokio::spawn(server);
//     TestApp { address, db_pool }
// }

// #[tokio::test]
// async fn health_check_works() {
//     let app = spawn_app().await;
//     let client = reqwest::Client::new();
//     let response = client
//         .get(&format!("{}/health_check", &app.address))
//         .send()
//         .await
//         .expect("Failed to execute request.");
//     assert!(response.status().is_success());
//     assert_eq!(Some(0), response.content_length());
// }

// #[tokio::test]
// async fn subscribe_return_200_for_valid_form_data() {
//     let app = spawn_app().await;
//     let client = reqwest::Client::new();

//     let configuration = get_configuration().expect("Failed to get configuration.");

//     let connection = PgPool::connect(&configuration.database.connection_string().expose_secret())
//         .await
//         .expect("Failed to connect to database.");

//     let _saved = sqlx::query("SELECT email, name FROM subscriptions")
//         .fetch_one(&connection)
//         .await
//         .expect("Failed to fetch subscriptions.");

//     let body = "name=olzhas&email=olzhik1123%40gmail.com";

//     let response = client
//         .post(&format!("{}/subscriptions", &app.address))
//         .header("Content-Type", "application/x-www-form-urlencoded")
//         .body(body)
//         .send()
//         .await
//         .expect("Failed to execute request.");

//     assert_eq!(StatusCode::OK, response.status());
// }

// #[tokio::test]
// async fn subscribe_return_400_for_missing_form_data() {
//     let app = spawn_app().await;

//     let client = reqwest::Client::new();

//     let body = "mal";

//     let response = client
//         .post(&format!("{}/subscriptions", &app.address))
//         .header("Content-Type", "application/x-www-form-urlencoded")
//         .body(body)
//         .send()
//         .await
//         .expect("Failed to execute request.");

//     assert_eq!(StatusCode::BAD_REQUEST, response.status());
// }
