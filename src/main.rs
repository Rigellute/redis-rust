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
        let maybe_frame = conn.read_value().await?;
        if let Some(frame) = maybe_frame {
            let command = frame.to_command()?;
            let response_frame = match command {
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
            };
            conn.write_value(response_frame).await?;
        } else {
            break;
        }
    }

    Ok(())
}
