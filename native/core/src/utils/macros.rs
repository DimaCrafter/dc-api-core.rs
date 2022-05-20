use napi::{Result, JsString, JsObject, KeyCollectionMode, KeyConversion, KeyFilter, CallContext, Status, bindgen_prelude::FromNapiValue, Env};

#[inline]
pub fn js_get_string (ctx: &CallContext, i: usize) -> Result<String> {
    let value = ctx.get::<JsString>(i)?;
    let value = value.into_utf8()?;
    return value.into_owned();
}

#[macro_export]
macro_rules! js_get_str {
    ($name:ident, $source:expr) => {
        let $name: JsString = $source?;
        let $name = $name.into_utf8()?;
        let $name = $name.as_str()?;
    };
}

#[macro_export]
macro_rules! js_get_class_prototype {
    ($class:expr) => {
        $class
            .get_named_property::<JsObject>("prototype")?
            .get_all_property_names(
                napi::KeyCollectionMode::OwnOnly,
                napi::KeyFilter::Configurable,
                napi::KeyConversion::NumbersToStrings
            )?
    };
}

#[macro_export]
macro_rules! js_get_class_internal_proto {
    ($class:expr) => {
        $class.get_named_property::<JsObject>("__proto__")?
    };
}

pub fn js_debug_object (obj: &JsObject) -> Result<()> {
    let props = obj.get_all_property_names(
        KeyCollectionMode::IncludePrototypes,
        KeyFilter::AllProperties,
        KeyConversion::NumbersToStrings
    )?;

    for i in 0..props.get_array_length()?  {
        js_get_str!(prop_name, props.get_element::<JsString>(i));
        js_get_str!(prop_value, obj.get_named_property::<JsString>(prop_name));
        println!("'{}': '{}'", prop_name, prop_value);
    }

    return Ok(());
}

#[inline]
pub fn js_err<T> (status: Status, reason: &str) -> Result<T> {
    Err(napi::Error::new(status, reason.to_string()))
}

pub fn js_get_error_message (env: &Env, is_full: bool) -> Result<String> {
    let mut exception = std::ptr::null_mut();
    assert_eq!(
        unsafe { napi::sys::napi_get_and_clear_last_exception(env.raw(), &mut exception) },
        napi::sys::Status::napi_ok
    );

    let obj = unsafe { JsObject::from_napi_value(env.raw(), exception)? };
    let message_result = obj.get_named_property::<JsString>(if is_full { "stack" } else { "message" });

    let js_message = match message_result {
        Ok(js_message) => js_message,
        Err(_) => obj.coerce_to_string()?
    };

    let js_message = js_message.into_utf8()?;
    return Ok(js_message.into_owned()?);
}

#[macro_export]
macro_rules! static_regex {
    ($name:ident, $regex:literal) => {
        lazy_static::lazy_static! {
            static ref $name: regex::Regex = regex::Regex::new($regex).unwrap();
        }
    };
}
