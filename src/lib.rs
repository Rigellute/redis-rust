mod command;
mod connection;
mod frame;
mod store;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use connection::Connection;
use frame::Frame;
use store::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    time,
    time::Duration,
};

use crate::command::Command;
use crate::store::Store;

pub async fn run(listener: TcpListener) -> Result<()> {
    // let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let addr = listener.local_addr().unwrap().to_string();
    println!("Server listening on {}", addr);

    let store = Store::new();
    let store = Arc::new(Mutex::new(store));

    // Start a background task to clean up expired values
    let cloned_store = store.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(1));
        loop {
            interval.tick().await;
            let mut store = cloned_store.lock().expect("Mutex is poisoned");
            store.clean_up();
        }
    });

    // Start the main loop for accepting connections
    loop {
        let (socket, _) = listener.accept().await?;
        let store = store.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, store).await {
                println!("Error handling request: {}", e);
            };
        });
    }
}

async fn handle_connection(stream: TcpStream, store: Arc<Mutex<Store>>) -> Result<()> {
    let mut conn = Connection::new(stream);

    while let Some(frame) = conn.read_value().await? {
        let response_frame = match frame.to_command() {
            Ok(command) => match command {
                Command::Get(key) => match store.lock().unwrap().get(&key) {
                    Some(found) => Frame::Bulk(found.value.clone()),
                    None => Frame::Null,
                },
                Command::Set(key, value, expiry) => {
                    let value = Value {
                        value: value.clone(),
                        expiry,
                    };

                    store.lock().unwrap().set(key.clone(), value);

                    Frame::Simple("OK".to_string())
                }
                Command::Echo(to_echo) => Frame::Bulk(to_echo.clone()),
                Command::Ping => Frame::Simple("PONG".to_string()),
            },
            Err(e) => Frame::Error(e.to_string()),
        };

        conn.write_value(response_frame).await?;
    }

    Ok(())
}
