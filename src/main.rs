use std::net::SocketAddr;

use axum::{Router, Server, routing::get};
use ::lib::file_list;

mod lib;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(file_list));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    let server = Server::bind(&addr);

    server.serve(app.into_make_service())
        .await
        .unwrap();
}
