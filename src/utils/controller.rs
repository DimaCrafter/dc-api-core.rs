use napi::{Env, JsFunction, JsObject, JsString, JsUnknown, Ref, Result};
use crate::{App, ActionCaller, camel_to_kebab, js_get_class_prototype, js_get_str};

pub struct Controller(pub(crate) Ref<()>);
impl Controller {
    pub fn get_caller (&self, env: Env, action_name: &str) -> Result<ActionCaller> {
        let inner: JsObject = env.get_reference_value(&self.0)?;
        let action = inner.get_named_property::<JsFunction>(action_name)?;

        let owner: JsObject = env.get_reference_value_unchecked(&self.0)?;
        return Ok(ActionCaller::new(env, action, Some(env.create_reference(owner)?)));
    }
}

impl App {
    pub fn register_controller (&mut self, env: Env, name: String, class: JsFunction) -> Result<()> {
        let inner = class.new_instance::<JsUnknown>(&[])?;
        let inner_ref = env.create_reference(inner)?;
        let instance = Controller(inner_ref);
        let prototype = js_get_class_prototype!(class);

        let path_base = camel_to_kebab(&name);
        for i in 0..prototype.get_array_length()? {
            js_get_str!(prop_name, prototype.get_element::<JsString>(i));
            if prop_name.starts_with('_') || prop_name == "onLoad" || prop_name == "constructor" {
                continue;
            }

            let path = format!("/{}/{}", path_base.as_str(), camel_to_kebab(&prop_name.to_string()));
            self.router.register(path, instance.get_caller(env, prop_name)?);
        }

        self.controllers.insert(name, instance);
        return Ok(());
    }
}
