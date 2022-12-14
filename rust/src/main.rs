use std::net::SocketAddr;

use axum::{routing::get, Router, Server};
use lib::file_handle::{file, file_list};
use lib::{users_html, users_json, users_json_agg, users_json_manual};

use mimalloc::MiMalloc;
use tokio_postgres::NoTls;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const PORT: u16 = 8000;

async fn run(port: u16, user: &str, pword: &str, dbname: &str) {
    let conn_param = format!("host=localhost user={user} password={pword} dbname={dbname}");

    let manager =
        bb8_postgres::PostgresConnectionManager::new_from_stringlike(&conn_param, NoTls).unwrap();

    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/file_list", get(file_list))
        .route("/file", get(file))
        .route("/users_json", get(users_json))
        .route("/users_json_agg", get(users_json_agg))
        .route("/users_json_manual", get(users_json_manual))
        .route("/users_html", get(users_html))
        .with_state(pool);

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

    rt.block_on(async { run(port, "postgres", "password", "list_of_users").await });
}
