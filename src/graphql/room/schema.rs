use juniper::{GraphQLInputObject, GraphQLObject};
use serde::{Serialize, Deserialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
use derivative::{self, Derivative};



#[derive(Serialize, Deserialize, Debug, GraphQLInputObject)]
pub struct RoomInput {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, GraphQLObject, Debug, Clone, Derivative, FromRow)]
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
