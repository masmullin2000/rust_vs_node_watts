use std::net::SocketAddr;

use ::lib::{file, file_list, users};
use axum::{routing::get, Router, Server};

mod lib;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const PORT: u16 = 8000;

async fn run(port: u16) {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect("postgress://mm:.@localhost/list_of_users")
        .await
        .unwrap();

    let app = Router::with_state(pool)
        .route("/file_list", get(file_list))
        .route("/file", get(file))
        .route("/users", get(users));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let server = Server::bind(&addr);

    server.serve(app.into_make_service()).await.unwrap();
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let port = if args.len() >= 2 {
        args[1].parse::<u16>().unwrap_or(PORT)
    } else {
        PORT
    };

    let threads = if args.len() >= 3 {
        args[2].parse::<usize>().unwrap_or_else(|_| num_cpus::get())
    } else {
        num_cpus::get()
    };

    println!("Running on port {port} with {threads} threads");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async { run(port).await });
}
