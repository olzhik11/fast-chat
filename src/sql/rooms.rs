use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, QueryBuilder};
use std::ops::DerefMut;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    api::{rooms::SearchParams, utils::SearchPaginatedResponse},
    errors::{AppError, AppErrorType},
};
use derivative::{self, Derivative};

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomInput {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomsResponse {
    pub rooms: Vec<RoomEntity>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct RoomEntity {
    #[sqlx(flatten)]
    room: Room,
    users_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Derivative, FromRow)]
#[derivative(Default)]
pub struct Room {
    #[derivative(Default(value = "Uuid::new_v4()"))]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[instrument(name = "Creating a new room.", skip(pool))]
pub async fn create_room(
    pool: &PgPool,
    room_input: RoomInput,
    user_id: Uuid,
) -> Result<Room, AppError> {
    let mut tx = pool.begin().await.map_err(|e| {
        AppError::new(
            "Transaction init error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })?;

    let room = Room {
        name: room_input.name,
        description: room_input.description,
        ..Default::default()
    };

    let new_room = sqlx::query_as::<_, Room>(
        r#"
        INSERT INTO rooms (id, name, description, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, description, created_at, updated_at"#,
    )
    .bind(room.id)
    .bind(room.name)
    .bind(room.description)
    .bind(room.created_at)
    .bind(room.updated_at)
    .fetch_one(tx.deref_mut())
    .await
    .map_err(|e| {
        AppError::new(
            "Transaction insert room error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })?;

    sqlx::query(
        r#"
        INSERT INTO room_users (room_id, user_id, joined_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(room.id)
    .bind(user_id)
    .bind(chrono::Utc::now())
    .execute(tx.deref_mut())
    .await
    .map_err(|e| {
        AppError::new(
            "Transaction insert room_user room error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })?;

    tx.commit().await.map_err(|e| {
        AppError::new(
            "Transaction commit error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })?;

    Ok(new_room)
}

#[instrument(name = "Getting rooms.", skip(pool))]
pub async fn get_rooms(pool: &PgPool, id: Uuid) -> Result<Vec<RoomEntity>, AppError> {
    sqlx::query_as::<_, RoomEntity>(
        r#"
        SELECT r.id, r.name, r.description, r.created_at, r.updated_at, COUNT(ru.user_id) as users_count
        FROM rooms r INNER JOIN room_users ru
        ON r.id = ru.room_id
        WHERE ru.user_id = $1
        GROUP BY r.id
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        AppError::new(
            "Get rooms error.".to_string(),
            AppErrorType::DatabaseError(e),
        )
    })
}

#[instrument(name = "Searching rooms.", skip(pool))]
pub async fn search_rooms(
    pool: &PgPool,
    user_id: Uuid,
    params: SearchParams,
) -> Result<SearchPaginatedResponse<RoomEntity>, AppError> {
    let SearchParams {
        query,
        limit,
        offset,
    } = params;

    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);

    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT r.id, r.name, r.description, r.created_at, r.updated_at, COUNT(ru.user_id) as users_count
        FROM rooms r INNER JOIN room_users ru
        ON r.id = ru.room_id
        WHERE ru.user_id = 
        "#,
    );

    let mut count_query_builder = QueryBuilder::new(
        r#"
        SELECT COUNT(DISTINCT r.id)
        FROM rooms r INNER JOIN room_users ru
        ON r.id = ru.room_id
        WHERE ru.user_id = 
        "#,
    );

    count_query_builder.push_bind(user_id);

    query_builder.push_bind(user_id);

    if let Some(ref search_query) = query {
        query_builder
            .push("AND r.name ILIKE ")
            .push_bind(search_query);
        count_query_builder
            .push("AND r.name ILIKE ")
            .push_bind(search_query);
    }

    query_builder
        .push(
            r#"
        GROUP BY r.id
        ORDER BY r.created_at DESC
        LIMIT
        "#,
        )
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset)
        .push(";");

    let total_count = count_query_builder
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::new(e.to_string(), AppErrorType::DatabaseError(e)))?;

    let rooms = query_builder
        .build_query_as::<RoomEntity>()
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::new(e.to_string(), AppErrorType::DatabaseError(e)))?;

    Ok(SearchPaginatedResponse {
        data: rooms,
        total_count,
    })
}
