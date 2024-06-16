use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Form, Json,
};
use axum_macros::debug_handler;
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
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
    graphql::{
        root::GraphQLContext,
        user::schema::{SessionUser, UserInput},
    },
    sql::user::{get_user, insert_user},
    startup::AppState,
};

use super::user::schema::User;

#[derive(Serialize, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[debug_handler]
pub async fn graphql(
    State(data): State<AppState>,
    claims: Claims,
    Json(graphql_req): Json<GraphQLRequest>,
) -> impl IntoResponse {
    let context = GraphQLContext::new(data.pool.clone(), claims);
    let res = graphql_req.execute(&data.schema, &context).await;

    let json = serde_json::to_string(&res).unwrap();

    let status = if res.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };

    (status, json)
}

pub async fn playground() -> Html<String> {
    let html = graphiql_source("/graphql", None);
    Html(html)
}

#[instrument(name = "Login.", skip(data, form), fields(form.email = %form.email))]
#[debug_handler]
pub async fn login(
    State(data): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Response<String>, AppError> {
    let user = get_user(&data.pool, &form.email).await?;

    verify_password(Secret::new(form.password), &user.password)?;

    let session_user = SessionUser {
        id: user.id,
        email: &user.email,
    };

    let ttl = configuration::get_configuration()?.token_max_age;

    let token = encode_token(&Claims::new(&session_user, ttl))?;

    let auth_header_pair = get_auth_header_pair(token);

    Ok(Response::builder()
        .header(auth_header_pair.0, auth_header_pair.1)
        .body("Authorized".to_string())
        .unwrap())
}

pub async fn register(
    State(data): State<AppState>,
    form: Form<UserInput>,
) -> Result<Response<String>, AppError> {
    // Validate user input
    let user_input = UserInput::validate_user_input(form.0)?;
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
