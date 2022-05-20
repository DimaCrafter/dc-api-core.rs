use napi::{JsObject, Env, Result, JsUnknown};
use crate::{static_regex, context::js_set_member};

fn parse_query_path<'a, C> (name: &'a str, collect_part: &mut C) -> Result<bool>
    where C: FnMut(&'a str) -> (),
{
    // /foo/bar?a[b][c]=123
    // ... [c]
    // ... [b]
    // a

    //  vec!["a", "b", "c"]
    static_regex!(OBJ_REGEX, r"(.*)\[([A-Za-z0-9_-]*)\]$");

    match OBJ_REGEX.captures(name) {
        Some(capture) => {
			let is_last = !parse_query_path(capture.get(1).unwrap().as_str(), collect_part)?;
			if is_last {
                collect_part(capture.get(1).unwrap().as_str());
            }

            collect_part(capture.get(2).unwrap().as_str());
            return Ok(true);
        }
        None => {
            return Ok(false);
        }
    }
}

#[inline]
fn parse_query_value (env: &Env, value: &str) -> Result<JsUnknown> {
	Ok(match value {
		"true" => env.get_boolean(true)?.into_unknown(),
		"false" => env.get_boolean(false)?.into_unknown(),
		"" => env.get_null()?.into_unknown(),
		_ => env.create_string(value)?.into_unknown()
	})
}

pub fn parse_query_string (env: &Env, input: &String) -> Result<JsObject> {
	let mut obj = env.create_object()?;

    for part in input.split('&') {
        if let Some((name, value)) = part.split_once('=') {
			let mut prop_path = Vec::<&str>::new();
			if parse_query_path(name, &mut |part| prop_path.push(part))? {
				js_set_member(env, &mut obj, prop_path, parse_query_value(env, value)?)?;
			} else {
				obj.set_named_property(name, parse_query_value(env, value)?)?;
			}
        } else {
            obj.set_named_property(part, env.get_boolean(true)?)?;
        }
    }

	return Ok(obj);
}
