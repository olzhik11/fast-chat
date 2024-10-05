use axum::{
    extract::{Json, Query, State},
    response::IntoResponse,
};

use crate::{
    crypt::token::Claims,
    errors::AppError,
    db::{
        self,
        rooms::schema::{RoomInput, RoomsResponse},
    },
    startup::AppState,
};

use crate::api::utils::SearchParams;

pub async fn get_rooms(
    State(data): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    db::rooms::get_rooms(&data.pool, claims.sub)
        .await
        .map(|rooms| Json(RoomsResponse { rooms }))
}

pub async fn create_room(
    State(data): State<AppState>,
    claims: Claims,
    Json(room): Json<RoomInput>,
) -> Result<impl IntoResponse, AppError> {
    db::rooms::create_room(&data.pool, room, claims.sub)
        .await
        .map(|room| Json(room))
}

pub async fn search_rooms(
    State(data): State<AppState>,
    claims: Claims,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    db::rooms::search_rooms(&data.pool, claims.sub, params)
        .await
        .map(|result| Json(result))
}
