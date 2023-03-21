mod connection;
mod frame;
mod store;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use connection::Connection;
use tokio::{
    net::{TcpListener, TcpStream},
    time,
    time::Duration,
};

use crate::store::Store;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server listening on :6379");

    let store = Store::new();
    let store = Arc::new(Mutex::new(store));

    // Start a task to clean up expired values
    let cloned_store = store.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(1));
        loop {
            interval.tick().await;
            let mut store = cloned_store.lock().expect("Mutex is poisoned");
            store.clean_up();
        }
    });

    loop {
        let (socket, _) = listener.accept().await?;
        let store = store.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, store).await {
                println!("Error handling request {}", e);
            };
        });
    }
}

async fn handle_connection(stream: TcpStream, store: Arc<Mutex<Store>>) -> Result<()> {
    let mut conn = Connection::new(stream);

    loop {
        if let Some(frame) = conn.read_value().await? {
            dbg!(frame);
        } else {
            break;
        };
    }

    Ok(())
}
