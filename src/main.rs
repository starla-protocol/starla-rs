use axum::{Json, Router, routing::get};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct RootResponse {
    implementation: &'static str,
    state: &'static str,
    target_protocol_version: &'static str,
    target_binding: &'static str,
    target_profile: &'static str,
}

async fn root() -> Json<RootResponse> {
    Json(RootResponse {
        implementation: "starla-rs",
        state: "bootstrap",
        target_protocol_version: "v1",
        target_binding: "HTTP Binding v1",
        target_profile: "Core",
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));
    let addr = SocketAddr::from(([127, 0, 0, 1], 4747));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind listener");

    axum::serve(listener, app).await.expect("serve app");
}
