use proc_macro::TokenStream;

extern crate proc_macro;
mod utils;

#[proc_macro]
pub fn assert_stream (input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let mut args = input_str.split(',');
    let stream = args.next().unwrap();
    let sequence = args.next().unwrap().trim();
    let invalid_result = args.next().unwrap();

    let sequence = sequence[1..(sequence.len()-1)]
        .replace("\\r", "\r")
        .replace("\\n", "\n");

    let mut result = String::new();
    for char in sequence.chars() {
        result.push_str("{");
        result.push_str("let mut byte = ");
        result.push_str(stream);
        result.push_str(".read_u8().unwrap();");
        result.push_str("if byte != ");
        result.push_str(&(char as u8).to_string());
        result.push_str(" { return ");
        result.push_str(invalid_result);
        result.push_str(" }");
        result.push_str("}");
    }

    return result.parse().unwrap()
}
