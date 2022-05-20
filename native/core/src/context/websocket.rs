use std::net::IpAddr;
use napi::{Env, Result, Ref, JsObject, CallContext, JsUndefined, JsUnknown, Either, JsNumber, JsBoolean, JsBuffer, ValueType, JsString, Status};
use tokio::{io::BufStream, net::TcpStream};
use tokio_tungstenite::WebSocketStream;
use crate::{http::{entity::{HttpHeaders, ResponseType, Request, HttpConnection, BoxedHttpConnection}, codes::HttpCode, ParsedHttpConnection}, utils::macros::js_err, get_app};
use super::{create_base_context, extract_ctx};

pub struct ControllerSocketContext {
    pub req_headers: HttpHeaders,
    pub query_string: String,
    pub address: IpAddr
}

impl ControllerSocketContext {
    pub fn new (connection: &BoxedHttpConnection, req: Request) -> Self {
        Self {
            req_headers: req.headers,
            query_string: req.query,
            address: connection.get_address()
        }
    }

    pub fn into_js (self, env: &Env, controller: Option<&Ref<()>>) -> Result<JsObject> {
        let mut this = create_base_context(env, self, controller)?;
        this.create_named_method("emit", ctx_emit)?;
        this.create_named_method("end", ctx_end)?;
        return Ok(this);
    }
}

// todo: подумать об удалении поля id
static mut NEXT_WS_ID: usize = 0;
pub struct WebSocketConnection {
    pub id: usize,
    pub stream: WebSocketStream<BufStream<TcpStream>>,
    pub ctx_ref: Ref<()>
}

impl WebSocketConnection {
    pub fn new (stream: WebSocketStream<BufStream<TcpStream>>, ctx_ref: Ref<()>) -> &'static mut Self {
        let this = WebSocketConnection {
            id: unsafe { NEXT_WS_ID },
            stream, ctx_ref
        };

        // let this_ref = &mut this;

        unsafe { NEXT_WS_ID += 1; }
        get_app().websocket_connections.push(this);
        let a = get_app().websocket_connections.get_mut(get_app().websocket_connections.len() - 1).unwrap();

        return a;
    }
}


#[js_function(3)]
fn ctx_emit (call_ctx: CallContext) -> Result<JsUndefined> {
    // let data: JsUnknown = call_ctx.get(0)?;
    // let ctx = extract_ctx(&call_ctx)?;

    // let http_code: Either<JsNumber, JsUndefined> = call_ctx.get(1)?;
    // if let Either::A(http_code) = http_code {
    //     if let Some(http_code) = HttpCode::get_by_code(http_code.get_uint32()? as u16) {
    //         ctx.res_code = http_code;
    //     } else {
    //         call_ctx.env.throw_error("Incorrect response HTTP-code", None)?;
    //     }
    // }

    // let is_pure: Either<JsBoolean, JsUndefined> = call_ctx.get(2)?;
    // if let Either::A(is_pure) = is_pure {
    //     if is_pure.get_value()? {
    //         serialize_pure(ctx, data)?;
    //     } else {
    //         serialize_object(&call_ctx.env, ctx, data)?;
    //     }
    // }

    return call_ctx.env.get_undefined();
}

#[js_function(0)]
fn ctx_end (call_ctx: CallContext) -> Result<JsUndefined> {
	// todo: реализовать!
    // let ctx = extract_ctx(&call_ctx)?;
    // ctx.response = ResponseType::end;

    return call_ctx.env.get_undefined();
}
