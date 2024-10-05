use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<RoomInput> for Room {
    fn from(value: RoomInput) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: value.name,
            description: value.description,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

