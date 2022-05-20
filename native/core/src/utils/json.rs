use napi::{Env, JsFunction, JsObject, JsString, JsUnknown, Ref, Result};

pub struct JSON {
    js_stringify: Ref<()>,
    js_parse: Ref<()>
}

impl JSON {
    pub fn init (env: Env) -> Result<Self> {
        let global = env.get_global()?;
        let json_obj: JsObject = global.get_named_property("JSON")?;

        let js_stringify: JsFunction = json_obj.get_named_property("stringify")?;
        let js_stringify = env.create_reference(js_stringify)?;

        let js_parse: JsFunction = json_obj.get_named_property("parse")?;
        let js_parse = env.create_reference(js_parse)?;

        return Ok(JSON { js_stringify, js_parse });
    }

    pub fn stringify (&self, env: &Env, value: JsUnknown) -> Result<String> {
        let js_stringify: JsFunction = env.get_reference_value(&self.js_stringify)?;
        let result = js_stringify.call(None, &[value])?;
        let result: JsString  = unsafe { result.cast() };
        let result = result.into_utf8()?;
        return result.into_owned();
    }

    pub fn parse (&self, env: &Env, value: &str) -> Result<JsUnknown> {
        let js_parse: JsFunction = env.get_reference_value(&self.js_parse)?;
        return js_parse.call(None, &[env.create_string(value)?]);
    }
}
