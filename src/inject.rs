use axum::{
    body::{Bytes, HttpBody},
    http,
    response::{self, IntoResponse},
};
use html_editor::{
    operation::{Editable, Htmlifiable, Selector},
    Node,
};
use lazy_static::lazy_static;
use tower_http::services::fs::ServeFileSystemResponseBody;

lazy_static! {
    static ref RELOAD_SCRIPT: Node = {
        const RELOAD_SCRIPT_SOURCE: &str = include_str!("reload.js");
        Node::new_element(
            "script",
            vec![("async", "")],
            vec![Node::Text(RELOAD_SCRIPT_SOURCE.into())],
        )
    };
}

pub fn inject_live_reload(
    response: http::Response<ServeFileSystemResponseBody>,
) -> response::Response {
    let is_html = response
        .headers()
        .get("Content-Type")
        .map(|content_type| content_type == "text/html")
        .unwrap_or(false);
    response
        .map_data(move |data: Bytes| {
            if is_html {
                if let Some(mut nodes) = std::str::from_utf8(&data)
                    .map(String::from)
                    .ok()
                    .and_then(|s| html_editor::parse(&s).ok())
                {
                    let head = Selector::from("head");
                    nodes.insert_to(&head, RELOAD_SCRIPT.to_owned());

                    return Bytes::from(nodes.html());
                }
            }
            data
        })
        .into_response()
}
