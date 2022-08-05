use std::io::{Error, Read, Write};
use std::net::{SocketAddr, IpAddr, TcpStream, Shutdown};
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use std::str::FromStr;
use bufstream::BufStream;
use dc_macro::assert_stream;
use crate::http::codes::HttpCode;
use crate::http::entity::{HttpConnection, HttpEngine, HttpMethod, ParsingResult, Request, Response, ResponseType};
use crate::utils::stream::StreamUtils;

#[derive(Copy, Clone)]
pub struct Http1Engine;

impl HttpEngine<Http1Connection> for Http1Engine {
    fn handle_connection (socket: (TcpStream, SocketAddr)) -> Http1Connection {
        Http1Connection::new(socket)
    }
}

pub struct Http1Connection {
    stream: BufStream<TcpStream>,
    address: IpAddr,
    version_minor: char
}

impl Http1Connection {
    fn new (socket: (TcpStream, SocketAddr)) -> Self {
        Http1Connection {
            stream: BufStream::new(socket.0),
            address: socket.1.ip(),
            version_minor: '\0'
        }
    }
}

impl HttpConnection for Http1Connection {
    fn get_address (&self) -> IpAddr { self.address }
    fn into_stream (self) -> BufStream<TcpStream> { self.stream }

    fn parse (&mut self) -> ParsingResult {
        let method = self.stream.read_string_before(' ');
        if method.is_none() { return ParsingResult::Invalid; }

        let method = HttpMethod::from_str(method.unwrap().as_str());
        if method.is_none() { return ParsingResult::Error(HttpCode::MethodNotAllowed) }

        let path = self.stream.read_string_before(' ');
        if path.is_none() { return ParsingResult::Error(HttpCode::RequestEntityTooLarge); }

        // let mut a = [0u8; 7];
        // self.stream.read_exact(&mut a);
        assert_stream!(self.stream, "HTTP/1.", ParsingResult::Invalid);
        self.version_minor = self.stream.read_u8().unwrap() as char;

        let mut req = Request::new(method.unwrap(), path.unwrap());

        assert_stream!(self.stream, "\r", ParsingResult::Invalid);
        loop {
            assert_stream!(self.stream, "\n", ParsingResult::Invalid);
            let mut header_name = Vec::new();
            header_name.push(self.stream.read_u8().unwrap());
            header_name.push(self.stream.read_u8().unwrap());

            if header_name[0] == '\r' as u8 && header_name[1] == '\n' as u8 {
                break;
            }

            let header_read_result = self.stream.read_before(':' as u8, &mut header_name);
            if header_read_result.is_none() { return ParsingResult::Error(HttpCode::RequestHeaderFieldsTooLarge) }

            let header_value = self.stream.read_string_before('\r');
            if header_value.is_none() { return ParsingResult::Error(HttpCode::RequestHeaderFieldsTooLarge) }

            let header_name = unsafe { String::from_utf8_unchecked(header_name) };
            req.headers.set_normal(header_name, header_value.unwrap());
        }

        if matches!(req.method, HttpMethod::POST) {
            let length_header = req.headers.get("Content-Length");
            if length_header.is_none() { return ParsingResult::Error(HttpCode::BadRequest) }

            let size = usize::from_str(length_header.unwrap().as_str());
            if size.is_err() { return ParsingResult::Error(HttpCode::BadRequest) }

            let size = size.unwrap();
            let mut body_raw = Vec::<u8>::with_capacity(size);
            unsafe { body_raw.set_len(size); }

            let body_raw = body_raw.as_mut_slice();
            self.stream.read_exact(body_raw).unwrap();
            println!("Content:\n{:?}", body_raw);
        }

        return ParsingResult::Complete(req);
    }

    fn respond (&mut self, res: Response) -> Result<(), Error> {
        if let ResponseType::Drop = res.payload {
            return Ok(());
        }

        self.stream.write(b"HTTP/1.")?;
        self.stream.write_u8(self.version_minor as u8)?;
        self.stream.write_u8(' ' as u8)?;
        let (res_code, res_reason) = res.code.get_description();

        self.stream.write(res_code.as_bytes())?;
        self.stream.write_u8(' ' as u8)?;
        self.stream.write(res_reason.as_bytes())?;

        for header in &res.headers {
            self.stream.write(b"\r\n")?;
            self.stream.write(header.name.as_bytes())?;
            self.stream.write(b": ")?;
            self.stream.write(header.value.as_bytes())?;
        }

        self.stream.write(b"\r\n\r\n")?;
        if let ResponseType::Payload(payload) = res.payload {
            self.stream.write(&payload)?;
        }

        return Ok(());
    }

    fn disconnect (self) -> Result<(), Error> {
        let stream = self.stream.into_inner()?;
        return stream.shutdown(Shutdown::Both);
    }
}
