use std::fmt;

use redis::{
    aio::ConnectionManager,
    streams::{StreamId, StreamKey, StreamReadReply},
    AsyncCommands, Value,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, PgPool};
use tracing::info;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppErrorType},
    sql::message::{delete_messages, insert_message, mark_as_seen, update_message},
    ws::schema::{SocketMessageContent, SocketMessageSendContent},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AsyncEvent {
    Send(SocketMessageSendContent),
    Update(SocketMessageContent),
    Delete(Vec<Uuid>),
    MarkAsSeen(Vec<Uuid>),
}

impl fmt::Display for AsyncEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AsyncEvent::Send(content) => {
                write!(f, "Send event with content: {}", content)
            }
            AsyncEvent::Update(content) => {
                write!(f, "Update event with content: {}", content)
            }
            AsyncEvent::Delete(ids) => {
                write!(f, "Delete event for IDs: {:?}", ids)
            }
            AsyncEvent::MarkAsSeen(ids) => {
                write!(f, "MarkAsSeen event for IDs: {:?}", ids)
            }
        }
    }
}

pub static REDIS_ENTRY_VALUE: &str = "value";
pub static ASYNC_EVENT_SEND: &str = "ASYNC_EVENT_SEND";
pub static ASYNC_EVENT_UPDATE: &str = "ASYNC_EVENT_UPDATE";
pub static ASYNC_EVENT_DELETE: &str = "ASYNC_EVENT_DELETE";
pub static ASYNC_EVENT_MARK_AS_SEEN: &str = "ASYNC_EVENT_MARK_AS_SEEN";

impl AsyncEvent {
    pub fn into_tuple_array(self) -> Vec<(&'static str, Vec<u8>)> {
        vec![(REDIS_ENTRY_VALUE, serde_json::to_vec(&self).unwrap())]
    }

    pub fn from_redis_value(value: &Value) -> Result<Self, AppError> {
        if let Value::Data(data) = value {
            serde_json::from_slice(data).map_err(|_| {
                AppError::new(
                    "Deserialization error.".to_string(),
                    AppErrorType::InternalServerError,
                )
            })
        } else {
            Err(AppError::new(
                "Deserialization error.".to_string(),
                AppErrorType::InternalServerError,
            ))
        }
    }
}

// https://redis.io/glossary/redis-queue/
#[derive(Clone)]
pub struct EventRedisStream {
    stream_key: String,
    redis_connection_manager: ConnectionManager,
}

// stream_key is the name of AsyncEvent
impl EventRedisStream {
    pub fn new(stream_key: &str, redis_connection_manager: ConnectionManager) -> Self {
        Self {
            stream_key: stream_key.into(),
            redis_connection_manager,
        }
    }

    pub async fn add_to_stream(&mut self, event: AsyncEvent) -> Result<(), AppError> {
        let args = event.into_tuple_array();
        let result = self
            .redis_connection_manager
            .xadd(&self.stream_key, "*", args.as_slice())
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Stream read with key - {} failed.", self.stream_key),
                    AppErrorType::RedisError(e),
                )
            })?;

        Ok(result)
    }

    pub async fn read_stream(&mut self) -> Result<Vec<(String, AsyncEvent)>, AppError> {
        let result: Option<StreamReadReply> = self
            .redis_connection_manager
            .xread(&[&self.stream_key], &["0"])
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Error reading from Redis stream: {}", e),
                    AppErrorType::RedisError(e),
                )
            })?;

        let mut stream_events = Vec::<(String, AsyncEvent)>::new();
        match result {
            Some(stream_reply) => {
                for StreamKey { ids, .. } in stream_reply.keys {
                    for StreamId { id, map, .. } in ids {
                        if let Some(value) = map.get(REDIS_ENTRY_VALUE) {
                            let event = AsyncEvent::from_redis_value(value)?;
                            stream_events.push((id, event));
                        } else {
                            return Err(AppError::new(
                                "StreamId read error".to_string(),
                                AppErrorType::InternalServerError,
                            ));
                        }
                    }
                }
            }
            None => {
                info!("No values for stream {}", self.stream_key);
            }
        };

        Ok(stream_events)
    }

    pub async fn process_event<'a>(
        &self,
        db_pool: &'a PgPool,
        event: AsyncEvent,
    ) -> Result<PgQueryResult, AppError> {
        match event {
            AsyncEvent::MarkAsSeen(ids) => mark_as_seen(db_pool, ids).await,
            AsyncEvent::Delete(ids) => delete_messages(db_pool, ids).await,
            AsyncEvent::Send(message) => insert_message(db_pool, message).await,
            AsyncEvent::Update(message) => {
                update_message(db_pool, message.id, message.content).await
            }
        }
    }

    pub async fn delete_events(&mut self, ids: Vec<String>) -> Result<(), AppError> {
        self.redis_connection_manager
            .xdel(&self.stream_key, &ids)
            .await
            .map_err(|e| AppError::new(e.to_string(), AppErrorType::RedisError(e)))?;
        Ok(())
    }
}
