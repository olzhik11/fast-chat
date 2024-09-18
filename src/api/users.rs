use crate::{errors::AppError, db, startup::AppState};
use axum::{extract::State, response::IntoResponse, Json};

pub async fn update_user(
    State(data): State<AppState>,
    Json(user): Json<db::users::UserUpdate>,
) -> Result<impl IntoResponse, AppError> {
    db::users::update_user(&data.pool, user)
        .await
        .map(|user| Json(user))
}
