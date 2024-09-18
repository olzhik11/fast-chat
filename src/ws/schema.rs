use std::fmt;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

use crate::db::users::QueryUser;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SocketMessage {
    Send(MessageRequest),
    Update(Message),
    Delete(Vec<Uuid>),
    Seen(Vec<Uuid>),
    Typing,
    Ping,
    Pong,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[repr(i16)]
pub enum MessageStatus {
    NotSent = 1,
    Sent = 2,
    Seen = 3,
}

#[derive(Serialize, Deserialize, Debug, FromRow, Clone)]
pub struct Author {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub content: String,
    #[sqlx(flatten)]
    pub author: QueryUser,
    pub room: Uuid,
    pub status: MessageStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Message {
    pub fn new(value: MessageRequest, author: QueryUser) -> Self {
        Message {
            id: Uuid::new_v4(),
            content: value.content,
            author,
            room: value.room,
            status: MessageStatus::Sent,
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow, Derivative)]
pub struct MessageRequest {
    pub room: Uuid,
    pub content: String,
    pub author: Uuid,
}

impl fmt::Display for MessageRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Room: {}, Content: {}", self.room, self.content)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ID: {}, Content: {}, Room: {}, Status: {:?}, Created at: {}",
            self.id, self.content, self.room, self.status, self.created_at
        )
    }
}
