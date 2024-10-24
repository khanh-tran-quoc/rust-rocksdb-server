use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::Router;
use rocksdb::DB;
use std::env;
use std::process;
use std::sync::Arc;
use tokio::runtime::Builder;

mod controller;
mod response;
mod service;

fn get_db_path() -> String {
    match env::current_dir() {
        Ok(path) => {
            if let Some(str_path) = path.join("rocks.db").to_str() {
                return str_path.to_string();
            } else {
                eprintln!("Failed to convert path to string.");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error getting current directory: {}", e);
            process::exit(1);
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    rocksdb: Arc<DB>,
}

fn main() {
    let rocksdb_path = get_db_path();
    let db = DB::open_default(rocksdb_path).unwrap();

    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let state = AppState {
        rocksdb: Arc::new(db),
    };

    runtime.block_on(async {
        let app = Router::new()
            .route("/put", post(controller::put))
            .route("/get", post(controller::get))
            .layer(DefaultBodyLimit::max(200000000))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
            .await
            .unwrap();

        println!("Server running",);
        axum::serve(listener, app).await.unwrap();
    });
}
