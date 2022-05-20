use futures::StreamExt;
use napi::{Ref, Env, Result, JsObject, JsString, JsFunction, JsUnknown};
use sha1::{Sha1, Digest};
use tokio_tungstenite::{tungstenite::{Message, protocol::Role}, WebSocketStream};
use crate::{utils::js::js_for_each, http::{ParsedHttpConnection, entity::{Response, HttpHeaders, ResponseType, Request, BoxedHttpConnection}, codes::HttpCode}, get_app, UnsafeEnv, context::websocket::{WebSocketConnection, ControllerSocketContext}};

type WSDispatchResult = std::result::Result<(usize, Response), Response>;
pub fn dispatch_websocket (connection: &mut ParsedHttpConnection) -> WSDispatchResult {
    let req = connection.req.as_ref().unwrap();
    let (endpoint, _) = match get_app().socket_endpoints.get_pair(&req.path) {
        Some(value) => value,
        None => return Err(Response::from_code(HttpCode::NotFound, "API endpoint not found"))
    };

    let mut res_headers = HttpHeaders::empty();
    res_headers.push("connection".to_string(), "Upgrade".to_string());
    res_headers.push("upgrade".to_string(), "websocket".to_string());

    if matches!(req.headers.get("sec-websocket-version"), Some(version) if version == "13") {
        res_headers.push("sec-websocket-version".to_string(), "13".to_string());
    } else {
        return Err(Response::from_code(HttpCode::Forbidden, "Unsupported WebSocket version"));
    }

    if let Some(client_key) = req.headers.get("sec-websocket-key") {
        let hash: &[u8] = &Sha1::digest(format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", client_key))[..];
        res_headers.push("sec-websocket-accept".to_string(), base64::encode(hash));
    } else {
        return Err(Response::from_code(HttpCode::BadRequest, "WebSocket accept header not provided"));
    }

    return Ok((endpoint, Response {
        code: HttpCode::SwitchingProtocols,
        headers: res_headers,
        payload: ResponseType::Upgrade
    }));
}

pub fn v5_dispatch_websocket (req: &Request) -> WSDispatchResult {
    let (endpoint, _) = match get_app().socket_endpoints.get_pair(&req.path) {
        Some(value) => value,
        None => return Err(Response::from_code(HttpCode::NotFound, "API endpoint not found"))
    };

    let mut res_headers = HttpHeaders::empty();
    res_headers.push("connection".to_string(), "Upgrade".to_string());
    res_headers.push("upgrade".to_string(), "websocket".to_string());

    if matches!(req.headers.get("sec-websocket-version"), Some(version) if version == "13") {
        res_headers.push("sec-websocket-version".to_string(), "13".to_string());
    } else {
        return Err(Response::from_code(HttpCode::Forbidden, "Unsupported WebSocket version"));
    }

    if let Some(client_key) = req.headers.get("sec-websocket-key") {
        let hash: &[u8] = &Sha1::digest(format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", client_key))[..];
        res_headers.push("sec-websocket-accept".to_string(), base64::encode(hash));
    } else {
        return Err(Response::from_code(HttpCode::BadRequest, "WebSocket accept header not provided"));
    }

    return Ok((endpoint, Response {
        code: HttpCode::SwitchingProtocols,
        headers: res_headers,
        payload: ResponseType::Upgrade
    }));
}

pub struct DispatchWSMessageTuple {
    pub msg: Message,
    pub endpoint: usize,
    pub proceed: fn(&str, JsUnknown)
}

impl DispatchWSMessageTuple {
    pub fn get_endpoint<'a> (&'a self) -> &'a WebSocketEndpoint {
        get_app().socket_endpoints.at(self.endpoint).unwrap()
    }
}

// todo: СЖЕЧЬ ВСЁ К ЧЁРТУ
// pub async fn v5_maintain_websocket (stream: BufStream<TcpStream>, req: Request, endpoint: usize) -> Result<()> {
pub async fn v5_maintain_websocket (http_connection: BoxedHttpConnection, req: Request, endpoint: usize) -> Result<()> {
    // let mut ctx = Some(ControllerSocketContext {
    //     req_headers: req.headers,
    //     query_string: req.query,
    //     address: http_connection.get_address()
    // });
    // let ctx_ref = get_app().provide_env(Box::new(move |env| {
    //     let controller = get_app().socket_endpoints.at(endpoint).map(|e| &e.controller);
    //     ctx.take().unwrap().into_js(env, controller)
    // }))?;
    let addr = http_connection.get_address();
    let ws_stream = WebSocketStream::from_raw_socket(http_connection.into_stream(), Role::Server, None).await;
    let ctx_callback = move |env: &Env, ctx_ref| {
        let connection = WebSocketConnection::new(ws_stream, ctx_ref);

        let fut = async move {
            loop {
                if let Some(msg_res) = connection.stream.next().await {
                    match msg_res {
                        Ok(msg) => {
                            let tuple = DispatchWSMessageTuple {
                                msg,
                                endpoint,
                                proceed: |event_name, payload| {
                                    println!("{}", event_name);
                                }
                            };

                            get_app().dispatch_websocket_message.call(Ok(tuple), napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking);

                            // dispatch_websocket_message(&env, msg, endpoint, |event_name, payload| {

                            // });
                        },
                        Err(err) => {
                            // todo: fire "error" event
                            println!("WebSocket error! {:?}", err);
                        }
                    }
                } else {
                    // todo: fire "close" event
                    break;
                }
            }

            return Ok(());
        };

        env.execute_tokio_future(fut, |env, _| { env.get_undefined() });
    };
    get_app().v5_create_ws_context.call(Ok((addr, req, endpoint, Box::new(ctx_callback))), napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking);




    return Ok(());
}

pub async fn maintain_websocket (env: Env, connection: ParsedHttpConnection, endpoint: &WebSocketEndpoint) -> Result<()> {
    let stream = connection.into_stream();
    let mut ws_stream = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;

    // let env = env.unwrap();
    loop {
        if let Some(msg_res) = ws_stream.next().await {
            match msg_res {
                Ok(msg) => {
                    dispatch_websocket_message(&env, msg, endpoint, |event_name, payload| {

                    });
                },
                Err(err) => {
                    // todo: fire "error" event
                    println!("WebSocket error! {:?}", err);
                }
            }
        } else {
            // todo: fire "close" event
            break;
        }
    }

    return Ok(());
}

pub fn dispatch_websocket_message<R> (env: &Env, msg: Message, endpoint: &WebSocketEndpoint, mut reply: R)
where R: FnMut(&str, JsUnknown) -> ()
{
    match msg {
        Message::Text(content) => {
            let (event_name, payload) = split_socket_message(&content);
            if let Some(handler) = endpoint.get_handler(event_name) {

                handler.call();
            } else {
                // todo: prettify
                return reply("error", env.create_string_from_std(format!("Event {} not defined", event_name)).unwrap().into_unknown());
                // return Err(Response::from_code(code, message))
            }
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

    pub fn push (&mut self, env: &Env, endpoint: JsObject) -> Result<()> {
        self.0.push(WebSocketEndpoint::from_js(env, endpoint)?);
        return Ok(());
    }

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

    pub fn at (&self, index: usize) -> Option<&WebSocketEndpoint> {
        return self.0.get(index);
    }
}

pub struct WebSocketEndpoint {
    pub path: String,
    pub controller: Ref<()>,
    pub handlers: Vec<SocketEventHandler>
}

impl WebSocketEndpoint {
    pub fn from_js (env: &Env, obj: JsObject) -> Result<Self> {
        let mut handlers = Vec::new();
        let handlers_js: JsObject = obj.get_named_property_unchecked("handlers")?;
        js_for_each(handlers_js, |handler: JsObject| {
            handlers.push(SocketEventHandler::from_js(env, handler)?);
            Ok(())
        })?;

        let path: JsString = obj.get_named_property_unchecked("path")?;
        return Ok(WebSocketEndpoint {
            path: path.into_utf8()?.into_owned()?,
            controller: env.create_reference(obj.get_named_property_unchecked::<JsObject>("controller")?)?,
            handlers
        });
    }

    pub fn get_handler (&self, name: &str) -> Option<&SocketEventHandler> {
        for handler in &self.handlers {
            if handler.event == name {
                return Some(handler);
            }
        }

        return None;
    }
}

pub struct SocketEventHandler {
    pub event: String,
    pub method: Ref<()>
}

impl SocketEventHandler {
    pub fn from_js (env: &Env, obj: JsObject) -> Result<Self> {
        let event: JsString = obj.get_named_property_unchecked("event")?;
        let event = event.into_utf8()?.into_owned()?;

        let method: JsFunction = obj.get_named_property_unchecked("method")?;
        let method = env.create_reference(method)?;

        return Ok(SocketEventHandler { event, method });
    }

    pub fn call (&self, ) {

    }
}
