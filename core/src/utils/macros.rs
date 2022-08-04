#[macro_export]
macro_rules! static_regex {
    ($name:ident, $regex:literal) => {
        lazy_static::lazy_static! {
            static ref $name: regex::Regex = regex::Regex::new($regex).unwrap();
        }
    };
}
