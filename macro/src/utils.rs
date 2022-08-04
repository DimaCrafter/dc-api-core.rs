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
