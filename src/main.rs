use axum::{
    body::{Bytes, HttpBody},
    extract::{
        ws::{Message, WebSocketUpgrade},
        Extension,
    },
    http::{self, StatusCode},
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};

use html_editor::operation::{Editable, Htmlifiable, Selector};
use html_editor::Node;
use notify::{watcher, RecursiveMode, Watcher};
use tokio::sync::broadcast::{self, Sender};
use tower::ServiceBuilder;
use tower_http::services::{fs::ServeFileSystemResponseBody, ServeDir};

use std::{
    env,
    net::SocketAddr,
    sync::{self, Arc},
    time::Duration,
};

#[tokio::main]
async fn main() {
    let current_dir = env::current_dir().expect("Unable to get current working directory");
    let port = 8080;
    let ip = [127, 0, 0, 1];

    let addr = SocketAddr::from((ip, port));

    let (broadcast_tx, _) = broadcast::channel::<()>(100);
    let shared_tx = Arc::new(broadcast_tx);

    {
        let current_dir = current_dir.clone();
        let notify = shared_tx.clone();
        std::thread::spawn(move || {
            let (tx, rx) = sync::mpsc::channel();
            //TODO: replace unwrap with proper error handling
            let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
            watcher
                .watch(&current_dir, RecursiveMode::Recursive)
                .unwrap();
            loop {
                if rx.recv().is_ok() {
                    notify.send(());
                }
            }
        });
    }

    let serve_dir = ServiceBuilder::new()
        .map_response(|response: http::Response<ServeFileSystemResponseBody>| {
            let is_html = response
                .headers()
                .get("Content-Type")
                .map(|content_type| content_type == "text/html")
                .unwrap_or(false);
            response
                .map_data(move |data| {
                    if is_html {
                        if let Some(mut nodes) = std::str::from_utf8(&data)
                            .map(String::from)
                            .ok()
                            .and_then(|s| html_editor::parse(&s).ok())
                        {
                            const RELOAD_SCRIPT: &str = include_str!("reload.js");
                            let head = Selector::from("head");
                            let reload_script = Node::new_element(
                                "script",
                                vec![("async", "")],
                                vec![Node::Text(RELOAD_SCRIPT.into())],
                            );
                            nodes.insert_to(&head, reload_script);

                            return Bytes::from(nodes.html());
                        }
                    }
                    data
                })
                .into_response()
        })
        .service(ServeDir::new(&current_dir));

    let router = Router::new()
        .route("/_butler/ws", get(ws_handler))
        .layer(Extension(shared_tx))
        .fallback(Router::new().nest("/", get_service(serve_dir).handle_error(static_serve_error)));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("Failed to start server");
}

async fn ws_handler(
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
                //Connection was closed
                return;
            }
        }
    })
}

async fn static_serve_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
}
