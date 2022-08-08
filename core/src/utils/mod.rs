use json::JsonValue;

pub mod log;
pub mod stream;
pub mod macros;

pub fn camel_to_kebab (value: &String) -> String {
    let mut is_last_upper = false;
    let mut result = String::new();

    for ch in value.chars() {
        if ch.to_ascii_uppercase() == ch {
            if is_last_upper {
                result.push(ch.to_ascii_lowercase());
            } else {
                result.push('-');
                result.push(ch.to_ascii_lowercase());
            }

            is_last_upper = true;
        } else {
            result.push(ch);
            is_last_upper = false;
        }
    }

    return if result.starts_with('-') { String::from(&result[1..]) } else { result };
}

pub fn json_access<'a> (obj: &'a mut JsonValue, path: &'a str) -> &'a mut JsonValue {
    let mut result = obj;
	for part in path.split('.') {
		result = &mut result[part];
	}

	return result;
}

pub fn json_read_array<'a, V: 'a, G, E>(obj: &'a JsonValue, getter: G, empty: E) -> Option<Vec<V>>
where
    G: Fn(&'a JsonValue) -> Option<V>,
    E: Fn() -> V,
{
    match obj {
        JsonValue::Array(array) => Some({
            array.iter()
                .map(|raw| {
                    if let Some(value) = (getter)(raw) {
                        value
                    } else {
                        (empty)()
                    }
                })
                .collect::<Vec<V>>()
        }),
        _ => None,
    }
}
