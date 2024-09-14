use std::fmt;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SocketMessage {
    Send(SocketMessageSendContent),
    Update(SocketMessageContent),
    Delete(Vec<Uuid>),
    Seen(Vec<Uuid>),
    Typing,
    Ping,
    Pong,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type, Default)]
#[repr(i16)]
pub enum MessageStatus {
    #[default]
    NotSent = 1,
    Sent = 2,
    Seen = 3,
}

#[derive(Serialize, Deserialize, Derivative, Debug, Clone)]
pub struct Author {
    pub id: Uuid,
    pub name: String,
}

/// MessageContent \
/// `id` - Uuid of the message \
/// `content` - content of the message \
/// `author` - author (creator, sender) of the message \
/// `room_id` - Uuid of the room where message has been sent \
/// `status` - status of message, whether its been sent or seen by the users
///
///

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct SocketMessageContent {
    pub id: Uuid,
    pub content: String,
    pub author: Author,
    pub room: Uuid,
    pub status: MessageStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow, Derivative)]
pub struct SocketMessageSendContent {
    pub room: Uuid,
    pub content: String,
    pub author: Author,
}

impl From<SocketMessageSendContent> for SocketMessageContent {
    fn from(value: SocketMessageSendContent) -> Self {
        SocketMessageContent {
            id: Uuid::new_v4(),
            content: value.content,
            author: value.author,
            room: value.room,
            status: MessageStatus::Sent,
            created_at: chrono::Utc::now(),
        }
    }
}

impl fmt::Display for SocketMessageSendContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Room: {}, Content: {}", self.room, self.content)
    }
}

impl fmt::Display for SocketMessageContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ID: {}, Content: {}, Room: {}, Status: {:?}, Created at: {}",
            self.id, self.content, self.room, self.status, self.created_at
        )
    }
}
