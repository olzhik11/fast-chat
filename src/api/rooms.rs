use axum::{
    extract::{Json, State},
    response::IntoResponse,
};

use crate::{
    crypt::token::Claims,
    errors::AppError,
    sql::{
        self,
        room::{RoomInput, RoomsResponse},
    },
    startup::AppState,
};

pub async fn get_rooms(
    State(data): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    sql::room::get_rooms(&data.pool, claims.sub)
        .await
        .map(|rooms| Json(RoomsResponse { rooms }))
}

pub async fn create_room(
    State(data): State<AppState>,
    claims: Claims,
    Json(room): Json<RoomInput>,
) -> Result<impl IntoResponse, AppError> {
    sql::room::create_room(&data.pool, room, claims.sub)
        .await
        .map(|room| Json(room))
}
