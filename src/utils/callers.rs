use napi::{Result, JsFunction, JsObject, JsUnknown, Ref, Env};
use crate::utils::macros::js_err;
use super::macros::js_get_error_message;

pub struct ActionCaller {
    method: Ref<()>,
    pub(crate) owner: Option<Ref<()>>
}

impl ActionCaller {
    pub fn new (env: Env, method: JsFunction, owner: Option<Ref<()>>) -> Self {
        ActionCaller {
            method: env.create_reference(method).unwrap(),
            owner
        }
    }

    pub fn call (&self, env: &Env, ctx: &JsObject) -> Result<JsUnknown> {
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
