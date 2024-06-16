use derivative::Derivative;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

use crate::graphql::user::schema::User;

#[derive(Serialize, Deserialize, Debug)]
pub enum SocketMessage {
    Send(SocketMessageContent),
    Update(SocketMessageContent),
    Delete(Vec<Uuid>),
    Seen(Vec<Uuid>),
    Typing,
    Ping,
    Pong,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type, Default)]
pub enum MessageStatus {
    #[default]
    NotSent = 1,
    Sent = 2,
    Seen = 3,
}

/// MessageContent \
/// `id` - Uuid of the message \
/// `content` - content of the message \
/// `author` - author (creator, sender) of the message \
/// `room_id` - Uuid of the room where message has been sent \
/// `status` - status of message, whether its been sent or seen by the users
#[derive(Serialize, Deserialize, Debug, Clone, FromRow, Derivative)]
#[derivative(Default)]
pub struct SocketMessageContent {
    #[derivative(Default(value = "Uuid::new_v4()"))]
    pub id: Uuid,
    pub content: String,
    #[sqlx(flatten)]
    pub author: User,
    pub room: Uuid,
    pub status: MessageStatus,
    #[derivative(Default(value = "chrono::Utc::now()"))]
    pub created_at: chrono::DateTime<chrono::Utc>,
}
