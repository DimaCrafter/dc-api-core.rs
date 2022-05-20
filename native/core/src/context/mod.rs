pub mod http;
pub mod websocket;

use std::net::IpAddr;
use napi::{Result, JsObject, Env, CallContext, Either, JsUndefined, JsString, JsUnknown, ValueType, Status, Property, JsFunction, Ref};
use crate::{utils::macros::{js_get_string, js_err}, get_app, parsers::query_string::parse_query_string};
use self::http::ControllerHttpContext;

fn create_base_context<T: 'static> (env: &Env, ctx: T, controller: Option<&Ref<()>>) -> Result<JsObject> {
    let mut this = env.create_object()?;

    this.create_named_method("header", ctx_header)?;
    this.define_properties(&[
        Property::new("address")?.with_getter(ctx_address),
        Property::new("query")?.with_getter(ctx_query)
    ])?;

    let patch_context: JsFunction = env.get_reference_value_unchecked(&get_app().patch_context)?;
    if let Some(controller_ref) = controller {
        let controller_obj: JsObject = env.get_reference_value_unchecked(controller_ref)?;
        this = unsafe { patch_context.call(None, &[this, controller_obj])?.cast() };
    } else {
        this = unsafe { patch_context.call(None, &[this])?.cast() };
    }

    env.wrap::<Option<T>>(&mut this, Some(ctx))?;
    return Ok(this);
}

fn extract_ctx<'a> (call_ctx: &'a CallContext) -> Result<&'a mut ControllerHttpContext> {
    let this = call_ctx.this::<JsObject>()?;
    let ctx: &mut Option<ControllerHttpContext> = call_ctx.env.unwrap(&this)?;
    return Ok(ctx.as_mut().unwrap());
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
    let mut this: JsObject = call_ctx.this()?;
    let cache: JsUnknown = this.get_named_property("_query")?;
    if let ValueType::Object = cache.get_type()? {
        return Ok(unsafe { cache.cast() });
    }

    let ctx = extract_ctx(&call_ctx)?;
    let obj = parse_query_string(&call_ctx.env, &ctx.query_string)?;

    this.set_named_property("_query", &obj)?;
    return Ok(obj);
}

pub enum JsCreateType {
    None,
    Object,
    Array
}

pub fn js_access_object (env: &Env, obj: &JsObject, path: &[&str], create: JsCreateType) -> Result<JsObject> {
    let mut result: JsObject = unsafe { std::mem::transmute_copy(obj) };
    for part in path {
        match result.get_named_property(part) {
            Ok(tmp) => result = tmp,
            Err(err) => {
                match create {
                    JsCreateType::None => {
                        return Err(err);
                    }
                    JsCreateType::Object => {
                        let tmp = env.create_object()?;
                        result.set_named_property(part, &tmp)?;
                        result = tmp;
                    }
                    JsCreateType::Array => {
                        let tmp = env.create_empty_array()?;
                        result.set_named_property(part, &tmp)?;
                        result = tmp;
                    }
                }
            }
        }
    }

    return Ok(result);
}

pub fn js_set_member (env: &Env, obj: &mut JsObject, path: Vec<&str>, value: JsUnknown) -> Result<()> {
    let mut last_index = path.len();
    if last_index == 1 {
        return obj.set_named_property(path[0], value);
    } else {
        last_index -= 1;
        let prop = path[last_index];
        if prop == "" {
            let mut target = js_access_object(env, obj, &path[0..last_index], JsCreateType::Array)?;
            return target.set_element(target.get_array_length_unchecked()?, value);
        } else {
            let mut target = js_access_object(env, obj, &path[0..last_index], JsCreateType::Object)?;
            return target.set_named_property(path[last_index], value);
        }
    }
}
