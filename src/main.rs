mod inject;
mod watch;
mod ws;

use axum::{
    extract::{
        ws::{Message, WebSocketUpgrade},
        Extension,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};

use tokio::sync::broadcast::{self, Sender};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

use std::{env, net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() {
    let current_dir = env::current_dir().expect("Unable to get current working directory");
    let port = 8080;
    let ip = [127, 0, 0, 1];

    let addr = SocketAddr::from((ip, port));

    let (broadcast_tx, _) = broadcast::channel::<()>(100);
    let shared_tx = Arc::new(broadcast_tx);

    watch::start_notify(shared_tx.clone(), current_dir.clone());

    let serve_dir = ServiceBuilder::new()
        .map_response(inject::inject_live_reload)
        .service(ServeDir::new(&current_dir));

    let router = Router::new()
        .route("/_butler/ws", get(ws::ws_handler))
        .layer(Extension(shared_tx))
        .fallback(Router::new().nest("/", get_service(serve_dir).handle_error(static_serve_error)));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("Failed to start server");
}

async fn static_serve_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
}
