use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufStream};
use tokio::net::TcpStream;

#[async_trait]
pub trait StreamUtils {
    async fn read_before (&mut self, needle: u8, cb: &mut Vec<u8>) -> Option<usize>;
    async fn read_string_before (&mut self, needle: char) -> Option<String>;
}

#[async_trait]
impl StreamUtils for BufStream<TcpStream> {
    #[inline]
    async fn read_before (&mut self, needle: u8, buffer: &mut Vec<u8>) -> Option<usize> {
        let offset = buffer.len();
        match self.read_until(needle as u8, buffer).await {
            Ok(mut len) => {
                if len == 0 { None }
                else {
                    len = len + offset - 1;
                    buffer.truncate(len);
                    Some(len)
                }
            }
            Err(_) => None
        }
    }

    #[inline]
    async fn read_string_before (&mut self, needle: char) -> Option<String> {
        let mut buffer = Vec::new();
        return if let Some(_) = self.read_before(needle as u8, &mut buffer).await {
            unsafe { Some(String::from_utf8_unchecked(buffer)) }
        } else {
            None
        }
    }
}
