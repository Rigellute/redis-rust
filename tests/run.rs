use anyhow::Result;
use redis::{aio::Connection, AsyncCommands, RedisResult};
use tokio::{
    net::TcpListener,
    time::{sleep_until, Duration, Instant},
};

#[tokio::test]
async fn test_ping() {
    let mut con = spawn_and_connect().await.unwrap();
    let res: RedisResult<String> = redis::cmd("PING").query_async(&mut con).await;
    assert_eq!(res, Ok("PONG".into()));
}

#[tokio::test]
async fn test_echo() {
    let mut con = spawn_and_connect().await.unwrap();
    let res: RedisResult<String> = redis::cmd("ECHO")
        .arg("hello world")
        .query_async(&mut con)
        .await;
    assert_eq!(res, Ok("hello world".into()));
}

#[tokio::test]
async fn test_set_get() {
    let mut con = spawn_and_connect().await.unwrap();
    let _: () = con.set("my_key", 42).await.unwrap();

    let res: RedisResult<String> = con.get("my_key").await;
    assert_eq!(res, Ok("42".into()));
}

#[tokio::test]
async fn test_expiry_millis() {
    let mut con = spawn_and_connect().await.unwrap();
    let _: () = redis::cmd("SET")
        .arg("my_key")
        .arg("my_value")
        .arg("PX")
        .arg(10)
        .query_async(&mut con)
        .await
        .unwrap();

    // Should find the value before expiry
    let res: RedisResult<String> = con.get("my_key").await;
    assert_eq!(res, Ok("my_value".into()));

    // Should NOT find the value after expiry
    sleep_until(Instant::now() + Duration::from_millis(11)).await;
    let res: RedisResult<Option<String>> = con.get("my_key").await;
    assert_eq!(res, Ok(None));
}

#[tokio::test]
async fn test_expiry_secs() {
    let mut con = spawn_and_connect().await.unwrap();
    let _: () = redis::cmd("SET")
        .arg("my_key")
        .arg("my_value")
        .arg("EX")
        .arg(1)
        .query_async(&mut con)
        .await
        .unwrap();

    // Should find the value before expiry
    let res: RedisResult<String> = con.get("my_key").await;
    assert_eq!(res, Ok("my_value".into()));

    sleep_until(Instant::now() + Duration::from_millis(1001)).await;
    let res: RedisResult<Option<String>> = con.get("my_key").await;
    assert_eq!(res, Ok(None));
}

async fn spawn_and_connect() -> Result<Connection> {
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let addr = listener.local_addr()?.to_string();
    tokio::spawn(async move {
        redis_rust::run(listener).await.expect("Server failed");
    });
    let client = redis::Client::open(format!("redis://{}", addr))?;

    let con = client.get_async_connection().await?;
    Ok(con)
}
