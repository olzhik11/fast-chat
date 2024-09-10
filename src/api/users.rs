use crate::{errors::AppError, sql, startup::AppState};
use axum::{extract::State, response::IntoResponse, Json};

pub async fn update_user(
    State(data): State<AppState>,
    Json(user): Json<sql::user::UserUpdate>,
) -> Result<impl IntoResponse, AppError> {
    sql::user::update_user(&data.pool, user)
        .await
        .map(|user| Json(user))
}
