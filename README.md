# 🪀 Toy Redis Clone in Rust 🚀

Purely for fun and learning, kicked off by following the excellent [codecrafters](https://app.codecrafters.io/courses/redis/overview) Redis course.

![](demo.gif)

## ✨ Features

- Parse the [Redis serialization protocol (RESP) specification](https://redis.io/docs/reference/protocol-spec/)
- Handle concurrent connections
- Implement commands:
  - `PING` 
  - `ECHO`
  - `GET`
  - `SET` with optional expiry
- Integration tests

## 👷‍♂️ Run locally

In one terminal, start our toy Redis server:

```sh
cargo run
```

And then use the `redis-cli` to query our server:

```
$ redis-cli PING
PONG
$ redis-cli ECHO 'Hello World!'
"Hello World!"
$ redis-cli SET myKey 1234
OK
$ redis-cli GET myKey
"1234"
```

## 🧪 Run tests

```sh
cargo test
```

