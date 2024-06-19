#![allow(dead_code)]
use std::env;
use tokio::net::TcpListener;

use self::handlers::handle_connection;

mod handlers;
mod http;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let directory = args
        .iter()
        .position(|arg| arg == "--directory")
        .and_then(|pos| args.get(pos + 1).cloned());

    let files_dir = directory.unwrap_or_else(|| "".to_string());

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let files_dir = files_dir.clone();
        tokio::task::spawn(async move {
            handle_connection(stream, &files_dir).await.unwrap();
        });
    }
}
