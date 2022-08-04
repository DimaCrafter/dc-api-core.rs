use std::{net::TcpStream, io::BufRead};
use bufstream::BufStream;

pub trait StreamUtils {
    fn read_before (&mut self, needle: u8, cb: &mut Vec<u8>) -> Option<usize>;
    fn read_string_before (&mut self, needle: char) -> Option<String>;
}

impl StreamUtils for BufStream<TcpStream> {
    #[inline]
    fn read_before (&mut self, needle: u8, buffer: &mut Vec<u8>) -> Option<usize> {
        let offset = buffer.len();
        match self.read_until(needle as u8, buffer) {
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
    fn read_string_before (&mut self, needle: char) -> Option<String> {
        let mut buffer = Vec::new();
        return if let Some(_) = self.read_before(needle as u8, &mut buffer) {
            unsafe { Some(String::from_utf8_unchecked(buffer)) }
        } else {
            None
        }
    }
}
