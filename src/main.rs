use starla_rs::{http, store::AppState};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = http::router(AppState::seeded());
    let addr = SocketAddr::from(([127, 0, 0, 1], 4747));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind listener");

    axum::serve(listener, app).await.expect("serve app");
}
