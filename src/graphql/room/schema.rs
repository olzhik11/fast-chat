use juniper::{GraphQLInputObject, GraphQLObject};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use derivative::{self, Derivative};



#[derive(Serialize, Deserialize, GraphQLInputObject)]
pub struct RoomInput {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize, GraphQLObject, Debug, Clone, Derivative)]
#[derivative(Default)]
pub struct Room {
    #[derivative(Default(value = "Uuid::new_v4()"))]
    id: Uuid,
    name: String,
    description: String,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
