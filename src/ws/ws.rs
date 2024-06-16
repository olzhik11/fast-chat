use crate::service::stream::{AsyncEvent, EventRedisStream, ASYNC_EVENT_DELETE, ASYNC_EVENT_MARK_AS_SEEN, ASYNC_EVENT_SEND, ASYNC_EVENT_UPDATE};
use crate::ws::schema::SocketMessage;
use crate::{crypt::token::Claims, errors::AppError, startup::AppState};
use axum::extract::ws::Message;
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use axum_macros::debug_handler;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use redis::aio::ConnectionManager;
use std::ops::ControlFlow;
use tokio::sync::broadcast::{self, Sender};
use tracing::info;
use uuid::Uuid;

#[debug_handler]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    _claims: Claims,
    State(state): State<AppState>,
    Path(room): Path<Uuid>,
) -> Result<Response, AppError> {
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, room)))
}

pub async fn handle_socket(socket: WebSocket, state: AppState, chat: Uuid) {
    let (mut sender, mut receiver) = socket.split();

    let tx = {
        let mut chats = state.chats.lock().expect("Failed to lock for chats.");
        match chats.get(&chat) {
            Some(chat) => chat.clone(),
            None => {
                let (tx, _rx) = broadcast::channel(100);
                chats.insert(chat.clone(), tx.clone());
                tx
            }
        }
    };

    let mut rx = tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            if sender.send(Message::Binary(message)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        while let Some(Ok(Message::Binary(message))) = receiver.next().await {
            if process_message(message, &tx, state.redis.clone()).is_break() {
                return;
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

fn process_message(
    msg: Vec<u8>,
    tx: &Sender<Vec<u8>>,
    redis_connection_manager: ConnectionManager,
) -> ControlFlow<(), ()> {
    if let Ok(msg) = serde_json::from_slice::<SocketMessage>(&msg) {
        match msg {
            SocketMessage::Send(message) => {
                let _ = tx.send(serde_json::to_vec(&SocketMessage::Send(message.clone())).unwrap());

                tokio::spawn(async move {
                    EventRedisStream::new(
                        ASYNC_EVENT_SEND,
                        redis_connection_manager,
                    )
                    .add_to_stream(AsyncEvent::Send(message.clone()))
                    .await
                });
            }
            SocketMessage::Seen(ids) => {
                // TODO: decide whether to notify other users that message was seen
                // let _ tx.send(serde_json::to_vec())

                tokio::spawn(async move {
                    EventRedisStream::new(
                        ASYNC_EVENT_MARK_AS_SEEN,
                        redis_connection_manager,
                    )
                    .add_to_stream(AsyncEvent::MarkAsSeen(ids.clone()))
                    .await
                });
            }
            SocketMessage::Update(message) => {
                let _ =
                    tx.send(serde_json::to_vec(&SocketMessage::Update(message.clone())).unwrap());

                tokio::spawn(async move {
                    EventRedisStream::new(
                        ASYNC_EVENT_UPDATE,
                        redis_connection_manager,
                    )
                    .add_to_stream(AsyncEvent::Update(message.clone()))
                    .await
                });
            }
            SocketMessage::Delete(ids) => {
                let _ = tx.send(serde_json::to_vec(&SocketMessage::Delete(ids.clone())).unwrap());

                tokio::spawn(async move {
                    EventRedisStream::new(ASYNC_EVENT_DELETE, redis_connection_manager)
                        .add_to_stream(AsyncEvent::Delete(ids.clone()))
                        .await
                });
            }
            SocketMessage::Pong => {
                let _ = tx.send(serde_json::to_vec(&SocketMessage::Pong).unwrap());
            }
            // health check
            SocketMessage::Ping => {
                let _ = tx.send(serde_json::to_vec(&SocketMessage::Pong).unwrap());
            }
            SocketMessage::Typing => {
                let _ = tx.send(serde_json::to_vec(&msg).unwrap());
            }
            SocketMessage::Close => return ControlFlow::Break(()),
        }
    } else {
        info!("Couldn't deserialize message");
    }

    ControlFlow::Continue(())
}
