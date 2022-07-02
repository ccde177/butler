use std::sync::Arc;

use axum::{
    extract::{ws::Message, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use tokio::sync::broadcast::Sender;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(shared_tx): Extension<Arc<Sender<()>>>,
) -> impl IntoResponse {
    let mut rx = shared_tx.subscribe();
    ws.on_upgrade(|mut socket| async move {
        while rx.recv().await.is_ok() {
            rx.resubscribe();
            if socket
                .send(Message::Text(String::from("reload")))
                .await
                .is_err()
            {
                return;
            }
        }
    })
}
