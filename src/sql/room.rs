use sqlx::{postgres::PgQueryResult, PgPool};
use tracing::instrument;

use crate::{errors::AppError, graphql::room::schema::Room};

#[instrument(name = "")]
pub async fn create_room(pool: &PgPool) -> Result<PgQueryResult, AppError> {
    todo!()
    // sqlx::query("
    // INSERT INTO rooms

    // ")
}

#[instrument(name = "")]
pub async fn get_rooms(pool: &PgPool) -> Result<Vec<Room>, AppError> {
    todo!()
}