use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    configuration,
    crypt::{
        hash::verify_password,
        token::{encode_token, get_auth_header_pair, Claims},
    },
    errors::AppError,
    sql::user::{get_user, insert_user, SessionUser, UserInput},
    startup::AppState,
};

use crate::sql::user::User;

#[derive(Serialize, Deserialize)]
pub struct SigninForm {
    pub email: String,
    pub password: String,
}

#[instrument(name = "Signin.", skip(data, payload), fields(payload.email = %payload.email))]
pub async fn sign_in(
    State(data): State<AppState>,
    Json(payload): Json<SigninForm>,
) -> Result<Response<String>, AppError> {
    let user = get_user(&data.pool, &payload.email).await?;

    verify_password(Secret::new(payload.password), &user.password)?;

    let session_user = SessionUser {
        id: user.id,
        email: &user.email,
    };

    let ttl = configuration::get_configuration()?.token_max_age;

    let token = encode_token(&Claims::new(&session_user, ttl))?;

    let auth_header_pair = get_auth_header_pair(token.clone());

    Ok(Response::builder()
        .header(auth_header_pair.0, auth_header_pair.1)
        .body("Authorized".to_string())
        .unwrap())
}

#[instrument(name = "Signup.", skip(data, payload), fields(payload.email = %payload.email))]
pub async fn sign_up(
    State(data): State<AppState>,
    Json(payload): Json<UserInput>,
) -> Result<Response<String>, AppError> {
    // Validate user input
    let user_input = UserInput::validate_user_input(payload)?;
    let user = User {
        name: user_input.name,
        email: user_input.email,
        password: user_input.password,
        ..Default::default()
    };

    insert_user(&data.pool, user).await.map(|_| {
        Response::builder()
            .body("User created successfully.".to_string())
            .unwrap()
    })
}

pub async fn who_am_i(
    State(data): State<AppState>,
    claim: Claims,
) -> Result<impl IntoResponse, AppError> {
    get_user(&data.pool, &claim.email)
        .await
        .map(|user| Json(user))
}
