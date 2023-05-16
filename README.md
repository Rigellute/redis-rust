A toy Redis clone in Rust following the [codecrafters](https://app.codecrafters.io/courses/redis/overview) course.

![](demo.gif)

## Features

- Parse the [Redis serialization protocol (RESP) specification](https://redis.io/docs/reference/protocol-spec/)
- Handle concurrent connections
- Implement commands:
  - `PING` 
  - `ECHO`
  - `GET`
  - `SET` with optional expiry

