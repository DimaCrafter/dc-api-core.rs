
use std::net::IpAddr;
use napi::{Result, JsObject, Env, CallContext, Either, JsUndefined, JsString, JsUnknown, ValueType, Status, Property, JsNumber, JsBoolean, JsBuffer};
use crate::{http::{entity::{HttpHeaders, Response}, ParsedHttpConnection, codes::HttpCode}, utils::macros::{js_get_string, js_err}, get_app};

pub struct ControllerHttpContext {
    pub req_headers: HttpHeaders,
    pub query_string: String,
    pub address: IpAddr,
    pub res_headers: HttpHeaders,
    pub res_payload: Option<Vec<u8>>,
    pub res_code: HttpCode
}

impl ControllerHttpContext {
    pub fn into_response (self) -> Response {
        Response {
            code: self.res_code,
            headers: self.res_headers,
            payload: self.res_payload
        }
    }
}

fn create_base_context<T: 'static> (env: &Env, ctx: T) -> Result<JsObject> {
    let mut this = env.create_object()?;
    this.create_named_method("header", ctx_header)?;
    this.create_named_method("send", ctx_send)?;

    this.define_properties(&[
        Property::new("address")?.with_getter(ctx_address),
        Property::new("query")?.with_getter(ctx_query)
    ])?;

    env.wrap::<Option<T>>(&mut this, Some(ctx))?;

    return Ok(this);
}

fn extract_ctx<'a> (call_ctx: &'a CallContext) -> Result<&'a mut ControllerHttpContext> {
    let this = call_ctx.this::<JsObject>()?;
    let ctx: &mut Option<ControllerHttpContext> = call_ctx.env.unwrap(&this)?;
    return Ok(ctx.as_mut().unwrap());
}

pub fn create_http_context (env: &Env, connection: &mut ParsedHttpConnection) -> Result<JsObject> {
    let req = connection.req.take().unwrap();

    let query_string = match req.path.split_once('?') {
        Some((_, query_string)) => query_string,
        None => ""
    };

    let ctx = ControllerHttpContext {
        req_headers: req.headers,
        query_string: query_string.to_string(),
        address: connection.get_address(),
        res_headers: HttpHeaders::empty(),
        res_payload: None,
        res_code: HttpCode::OK
    };

    let this = create_base_context(env, ctx)?;
    return Ok(this);
}

#[js_function(2)]
fn ctx_header (call_ctx: CallContext) -> Result<Either<JsString, JsUndefined>> {
    let key = js_get_string(&call_ctx, 0)?;
    let value = call_ctx.get::<JsUnknown>(1)?;
    let ctx = extract_ctx(&call_ctx)?;

    match value.get_type()? {
        ValueType::Null => {
            ctx.res_headers.remove(key);
        }
        ValueType::String => {
            let value = unsafe { value.cast::<JsString>() };
            let value = value.into_utf8()?;
            let value = value.into_owned()?;
            ctx.res_headers.push(key, value);
        }
        ValueType::Undefined => {
            let stored_option = ctx.req_headers.get(&key);
            if let Some(value) = stored_option {
                return Ok(Either::A(call_ctx.env.create_string(&value)?))
            }
        }
        _ => return js_err(Status::InvalidArg, "Invalid header value type")
    }

    return Ok(Either::B(call_ctx.env.get_undefined()?));
}

#[js_function(0)]
fn ctx_address (call_ctx: CallContext) -> Result<JsObject> {
    let ctx = extract_ctx(&call_ctx)?;
    let mut obj = call_ctx.env.create_object()?;
    match ctx.address {
        IpAddr::V4(value) => {
            obj.set_named_property("version", call_ctx.env.create_uint32(4)?)?;
            obj.set_named_property("value", call_ctx.env.create_string(&value.to_string())?)?;
        }
        IpAddr::V6(value) => {
            obj.set_named_property("version", call_ctx.env.create_uint32(6)?)?;
            obj.set_named_property("value", call_ctx.env.create_string(&value.to_string())?)?;
        }
    }

    return Ok(obj);
}

#[js_function(0)]
fn ctx_query (call_ctx: CallContext) -> Result<JsObject> {
    let ctx = extract_ctx(&call_ctx)?;
    let mut obj = call_ctx.env.create_object()?;

    for part in ctx.query_string.split('&') {
        if let Some((name, value)) = part.split_once('=') {
            obj.set_named_property(name, call_ctx.env.create_string(value)?)?;
        } else {
            obj.set_named_property(part, call_ctx.env.get_boolean(true)?)?;
        }
    }

    return Ok(obj);
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

pub fn serialize_object (env: &Env, ctx: &mut ControllerHttpContext, data: JsUnknown) -> Result<()> {
    let value = get_app().json.stringify(env, data)?;
    let value = value.into_bytes();

    ctx.res_headers.push_default("Content-Type".to_string(), "application/json".to_string());
    ctx.res_headers.push("Content-Length".to_string(), value.len().to_string());
    ctx.res_payload = Some(value);

    return Ok(());
}

fn serialize_pure (ctx: &mut ControllerHttpContext, data: JsUnknown) -> Result<()> {
    if data.is_buffer()? {
        let buffer: JsBuffer = unsafe { data.cast() };
        let value = buffer.into_value()?.to_vec();

        ctx.res_headers.push_default("Content-Type".to_string(), "application/octet-stream".to_string());
        ctx.res_headers.push("Content-Length".to_string(), value.len().to_string());
        ctx.res_payload = Some(value);
    } else {
        match data.get_type()? {
            ValueType::String => {
                let buffer: JsString = unsafe { data.cast() };
                let buffer = buffer.into_utf8()?;
                let value = buffer.take();

                ctx.res_headers.push_default("Content-Type".to_string(), "text/plain".to_string());
                ctx.res_headers.push("Content-Length".to_string(), value.len().to_string());
                ctx.res_payload = Some(value);
            }
            _ => {
                return js_err(Status::InvalidArg, "Unsupported response payload type");
            }
        }
    }

    return Ok(());
}
