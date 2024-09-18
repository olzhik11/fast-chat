use axum::{
    extract::{Json, Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
    crypt::token::Claims,
    errors::AppError,
    sql::{
        self,
        rooms::{RoomInput, RoomsResponse},
    },
    startup::AppState,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchParams {
    pub query: Option<String>,
    pub limit: Option<i16>,
    pub offset: Option<i16>,
}

impl Default for SearchParams {
    fn default() -> Self {
        SearchParams {
            query: None,
            limit: Some(20),
            offset: Some(0),
        }
    }
}

pub async fn get_rooms(
    State(data): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    sql::rooms::get_rooms(&data.pool, claims.sub)
        .await
        .map(|rooms| Json(RoomsResponse { rooms }))
}

pub async fn create_room(
    State(data): State<AppState>,
    claims: Claims,
    Json(room): Json<RoomInput>,
) -> Result<impl IntoResponse, AppError> {
    sql::rooms::create_room(&data.pool, room, claims.sub)
        .await
        .map(|room| Json(room))
}

pub async fn search_rooms(
    State(data): State<AppState>,
    claims: Claims,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    sql::rooms::search_rooms(&data.pool, claims.sub, params)
        .await
        .map(|result| Json(result))
}
