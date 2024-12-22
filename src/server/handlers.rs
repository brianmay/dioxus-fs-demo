use axum::extract::ws;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use axum::{extract::WebSocketUpgrade, response::Response};
use tracing::{debug, error};

use crate::server::database::DatabasePool;

#[axum::debug_handler]
pub async fn dioxus_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|mut socket| async move { while let Some(Ok(_msg)) = socket.recv().await {} })
}

/// echo server
#[axum::debug_handler]
pub async fn ws_echo_server(ws: WebSocketUpgrade) -> Response {
    debug!("Got incoming websocket connection.");
    ws.on_upgrade(|mut socket| async move {
        debug!("Upgraded websocket connection.");
        socket
            .send(ws::Message::Text("Why am I waiting?".to_string()))
            .await
            .unwrap();
        while let Some(Ok(msg)) = socket.recv().await {
            let msg = match msg {
                ws::Message::Text(msg) => Some(ws::Message::Text(msg.to_uppercase())),
                ws::Message::Close(..) => None,
                ws::Message::Binary(_) => None,
                ws::Message::Ping(_) => None,
                ws::Message::Pong(_) => None,
            };
            if let Some(msg) = msg {
                socket.send(msg).await.unwrap();
            }
        }
        debug!("Lost connection");
    })
}

// health check
#[axum::debug_handler]
pub async fn health_check(Extension(pool): Extension<DatabasePool>) -> Response {
    let mut conn = pool.get().await.unwrap();
    match crate::server::database::list_penguin_encounters(&mut conn).await {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            error!("Error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response()
        }
    }
}
