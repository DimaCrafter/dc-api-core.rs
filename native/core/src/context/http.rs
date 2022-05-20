use std::net::IpAddr;
use napi::{Env, Result, Ref, JsObject, CallContext, JsUndefined, JsUnknown, Either, JsNumber, JsBoolean, JsBuffer, ValueType, JsString, Status};
use crate::{http::{entity::{HttpHeaders, ResponseType, Response}, codes::HttpCode, ParsedHttpConnection}, utils::macros::{js_get_string, js_err}, get_app};
use super::{create_base_context, extract_ctx};

pub struct ControllerHttpContext {
    pub req_headers: HttpHeaders,
    pub query_string: String,
    pub address: IpAddr,
    pub res_headers: HttpHeaders,
    pub response: ResponseType,
    pub res_code: HttpCode
}

impl ControllerHttpContext {
    pub fn into_response (self) -> Response {
        Response {
            code: self.res_code,
            headers: self.res_headers,
            payload: self.response
        }
    }
}

pub fn create_http_context (env: &Env, connection: &mut ParsedHttpConnection, controller: Option<&Ref<()>>) -> Result<JsObject> {
    let req = connection.req.take().unwrap();
    let ctx = ControllerHttpContext {
        req_headers: req.headers,
        query_string: req.query,
        address: connection.get_address(),
        res_headers: HttpHeaders::empty(),
        response: ResponseType::NoContent,
        res_code: HttpCode::OK
    };

    let mut this = create_base_context(env, ctx, controller)?;
    this.create_named_method("send", ctx_send)?;
    this.create_named_method("drop", ctx_drop)?;
    this.create_named_method("redirect", ctx_redirect)?;
    return Ok(this);
}


#[js_function(3)]
fn ctx_send (call_ctx: CallContext) -> Result<JsUndefined> {
    let data: JsUnknown = call_ctx.get(0)?;
    let ctx = extract_ctx(&call_ctx)?;

    let http_code: Either<JsNumber, JsUndefined> = call_ctx.get(1)?;
    if let Either::A(http_code) = http_code {
        if let Some(http_code) = HttpCode::get_by_code(http_code.get_uint32()? as u16) {
            ctx.res_code = http_code;
        } else {
            call_ctx.env.throw_error("Incorrect response HTTP-code", None)?;
        }
    }

    let is_pure: Either<JsBoolean, JsUndefined> = call_ctx.get(2)?;
    if let Either::A(is_pure) = is_pure {
        if is_pure.get_value()? {
            serialize_pure(ctx, data)?;
        } else {
            serialize_object(&call_ctx.env, ctx, data)?;
        }
    }

    return call_ctx.env.get_undefined();
}

#[js_function(0)]
fn ctx_drop (call_ctx: CallContext) -> Result<JsUndefined> {
    let ctx = extract_ctx(&call_ctx)?;
    ctx.response = ResponseType::Drop;

    return call_ctx.env.get_undefined();
}

#[js_function(1)]
fn ctx_redirect (call_ctx: CallContext) -> Result<JsUndefined> {
    let ctx = extract_ctx(&call_ctx)?;
    ctx.response = ResponseType::NoContent;
    ctx.res_code = HttpCode::Found;
    ctx.res_headers.push("Location".to_string(), js_get_string(&call_ctx, 0)?);

    return call_ctx.env.get_undefined();
}

// TODO: rename "serialize_*" functions
pub fn serialize_object (env: &Env, ctx: &mut ControllerHttpContext, data: JsUnknown) -> Result<()> {
    let value = get_app().json.stringify(env, data)?;
    let value = value.into_bytes();

    ctx.res_headers.push_default("Content-Type".to_string(), "application/json".to_string());
    ctx.res_headers.push("Content-Length".to_string(), value.len().to_string());
    ctx.response = ResponseType::Payload(value);

    return Ok(());
}

fn serialize_pure (ctx: &mut ControllerHttpContext, data: JsUnknown) -> Result<()> {
    if data.is_buffer()? {
        let buffer: JsBuffer = unsafe { data.cast() };
        let value = buffer.into_value()?.to_vec();

        ctx.res_headers.push_default("Content-Type".to_string(), "application/octet-stream".to_string());
        ctx.res_headers.push("Content-Length".to_string(), value.len().to_string());
        ctx.response = ResponseType::Payload(value);
    } else {
        match data.get_type()? {
            ValueType::String => {
                let buffer: JsString = unsafe { data.cast() };
                let buffer = buffer.into_utf8()?;
                let value = buffer.take();

                ctx.res_headers.push_default("Content-Type".to_string(), "text/plain".to_string());
                ctx.res_headers.push("Content-Length".to_string(), value.len().to_string());
                ctx.response = ResponseType::Payload(value);
            }
            _ => {
                return js_err(Status::InvalidArg, "Unsupported response payload type");
            }
        }
    }

    return Ok(());
}
