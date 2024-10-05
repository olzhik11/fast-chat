use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{crypt::token::Claims, db::{self, messages::schema::Message}, errors::AppError, startup::AppState};

#[derive(Serialize, Deserialize)]
struct MessagesResponse {
    messages: Vec<Message>,
}

pub async fn get_messages(
    State(data): State<AppState>,
    Path(id): Path<Uuid>,
    _claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    db::messages::get_messages(&data.pool, id)
        .await
        .map(|messages| Json(MessagesResponse { messages }))
}
