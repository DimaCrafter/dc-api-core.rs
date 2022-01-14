use napi::{Result, JsFunction, JsObject, JsUnknown, NapiRaw, Ref, Env, bindgen_prelude::FromNapiValue};

use crate::utils::macros::{js_err, js_get_string};

use super::macros::js_get_error_message;

pub trait ActionCaller {
    fn call (&self, env: &Env, ctx: &JsObject) -> Result<JsUnknown>;
}

pub struct ControllerActionCaller {
    method: Ref<()>
}

impl ControllerActionCaller {
    pub fn new (env: Env, method: JsFunction) -> Box<Self> {
        Box::new(ControllerActionCaller {
            method: env.create_reference(method).unwrap()
        })
    }
}

impl ActionCaller for ControllerActionCaller {
    fn call (&self, env: &Env, ctx: &JsObject) -> Result<JsUnknown> {
        let method: JsFunction = env.get_reference_value(&self.method)?;

        match method.call_without_args(Some(ctx)) {
            Ok(result_obj) => {
                return Ok(result_obj);
            }
            Err(err) => {
                if let napi::Status::PendingException = err.status {
                    return js_err(napi::Status::GenericFailure, js_get_error_message(env, true)?.as_str());
                } else {
                    return Err(err);
                }
            }
        }
    }
}
