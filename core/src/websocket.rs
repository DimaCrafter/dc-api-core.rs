use std::sync::Mutex;
use sha1::{Sha1, Digest};
use tungstenite::{Message, Error};
use crate::{http::{entity::{Response, HttpHeaders, ResponseType, Request}, codes::HttpCode}, app::App, context::ws::SocketContext};

type EventCallerType = dyn FnMut(&mut SocketContext) + Sync + Send + 'static;

pub enum HandshakeResult {
    /* endpoint, response */
    Ok(usize, Response),
    Err(Response)
}

impl HandshakeResult {
    pub fn err (code: HttpCode, message: &str) -> Self {
        HandshakeResult::Err(Response::from_code(code, message))
    }

    pub fn ok (endpoint: usize, res_headers: HttpHeaders) -> Self {
        HandshakeResult::Ok(
            endpoint,
            Response {
                code: HttpCode::SwitchingProtocols,
                headers: res_headers,
                payload: ResponseType::Upgrade
            }
        )
    }
}

pub fn websocket_handshake (app_mutex: &Mutex<App>, req: &Request) -> HandshakeResult {
    let endpoint = {
        let app = app_mutex.lock().unwrap();
        match app.ws_endpoints.get_pair(&req.path) {
            Some(value) => value,
            None => return HandshakeResult::err(HttpCode::NotFound, "API endpoint not found")
        }
        .0
    };

    let mut res_headers = HttpHeaders::empty();
    res_headers.set("connection".to_string(), "Upgrade".to_string());
    res_headers.set("upgrade".to_string(), "websocket".to_string());

    if matches!(req.headers.get("sec-websocket-version"), Some(version) if version == "13") {
        res_headers.set("sec-websocket-version".to_string(), "13".to_string());
    } else {
        return HandshakeResult::err(HttpCode::Forbidden, "Unsupported WebSocket version");
    }

    if let Some(client_key) = req.headers.get("sec-websocket-key") {
        let hash: &[u8] = &Sha1::digest(format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", client_key))[..];
        res_headers.set("sec-websocket-accept".to_string(), base64::encode(hash));
    } else {
        return HandshakeResult::err(HttpCode::BadRequest, "WebSocket accept header not provided");
    }

    return HandshakeResult::ok(endpoint, res_headers);
}

pub fn maintain_websocket (app_mutex: &Mutex<App>, mut ctx: SocketContext, endpoint_index: usize) -> Result<(), ()> {
    loop {
        match ctx.stream.read_message() {
            Ok(msg) => {
                dispatch_websocket_message(app_mutex, &mut ctx, msg, endpoint_index);
            },
            Err(err) => {
                if let Error::ConnectionClosed = err {
                    // todo: fire "close" event
                    break;
                }

                // todo: fire "error" event
                println!("WebSocket error! {:?}", err);
            }
        }
    }

    return Ok(());
}

pub fn dispatch_websocket_message (app_mutex: &Mutex<App>, ctx: &mut SocketContext, msg: Message, endpoint_index: usize) {
    match msg {
        Message::Text(content) => {
            let (event_name, payload) = split_socket_message(&content);

            let mut app = app_mutex.lock().unwrap();
            let endpoint = app.ws_endpoints.at(endpoint_index).unwrap();
            let handler_opt = endpoint.handlers.get(event_name);

            if let Some(handler) = handler_opt {
                handler.call(ctx);
            } else {
                // todo: prettify
                return ctx.text("error", format!("Event {} not defined", event_name).as_str());
            }

            // todo: eager drop
            drop(app);
        }
        Message::Close(frame) => {
            if let Some(frame) = frame {
                println!(":close {}", frame.reason);
            }
        }
        _ => {}
    }
}

fn split_socket_message (payload: &str) -> (&str, &str) {
    if let Some(delim_index) = payload.find(':') {
        return (&payload[0..delim_index], &payload[delim_index+1..]);
    } else {
        return (payload, "");
    }
}

pub struct WebSocketEndpoints(Vec<WebSocketEndpoint>);
impl WebSocketEndpoints {
    pub fn empty () -> Self { WebSocketEndpoints(Vec::new()) }

    pub fn get (&self, path: &str) -> Option<&WebSocketEndpoint> {
        for endpoint in &self.0 {
            if endpoint.path == path {
                return Some(endpoint);
            }
        }

        return None;
    }

    pub fn get_pair (&self, path: &str) -> Option<(usize, &WebSocketEndpoint)> {
        for pair in self.0.iter().enumerate() {
            if pair.1.path == path {
                return Some(pair);
            }
        }

        return None;
    }

    pub fn at (&mut self, index: usize) -> Option<&mut WebSocketEndpoint> {
        return self.0.get_mut(index);
    }

    pub fn register<Caller: FnMut(&mut SocketContext) + Sync + Send + 'static> (&mut self, path: &str, event: &str, method: Caller) {
        let handler = SocketEventHandler { event: event.to_string(), method: Box::new(method) };
        for endpoint in &mut self.0 {
            if endpoint.path == path {
                endpoint.handlers.push(handler);
                return;
            }
        }

        self.0.push(WebSocketEndpoint::new(path, handler));
    }
}

pub struct WebSocketEndpoint {
    pub path: String,
    pub handlers: WebSocketHandlers
}

impl WebSocketEndpoint {
    pub fn new (path: &str, handler: SocketEventHandler) -> Self {
        WebSocketEndpoint {
            path: path.to_string(),
            handlers: WebSocketHandlers(vec![handler])
        }
    }
}

pub struct WebSocketHandlers(Vec<SocketEventHandler>);

impl WebSocketHandlers {
    pub fn get (&mut self, name: &str) -> Option<&mut SocketEventHandler> {
        for handler in &mut self.0 {
            if handler.event == name {
                return Some(handler);
            }
        }

        return None;
    }

    #[inline]
    pub fn push (&mut self, handler: SocketEventHandler) {
        self.0.push(handler)
    }
}

pub struct SocketEventHandler {
    pub event: String,
    pub method: Box<EventCallerType>
}

impl SocketEventHandler {
    pub fn call (&mut self, ctx: &mut SocketContext) {
        (self.method)(ctx);
    }
}
