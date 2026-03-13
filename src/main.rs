use starla_rs::{http, store::AppState};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = http::router(AppState::seeded());
    let addr = bind_addr();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind listener");

    axum::serve(listener, app).await.expect("serve app");
}

fn bind_addr() -> SocketAddr {
    std::env::var("STARLA_BIND_ADDR")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 4747)))
}
