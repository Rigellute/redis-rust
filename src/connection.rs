use anyhow::Result;
use bytes::BytesMut;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::frame::Frame;

pub struct Connection {
    buffer: BytesMut,
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            buffer: BytesMut::with_capacity(512),
            stream,
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Frame>> {
        loop {
            let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

            // Connection closed
            if bytes_read == 0 {
                return Ok(None);
            }

            let input = self.buffer.split().freeze();
            let input = std::str::from_utf8(&input)?;
            if let Some(frame) = Frame::decode(input)? {
                return Ok(Some(frame));
            };
        }
    }
}
