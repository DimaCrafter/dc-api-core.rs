use std::collections::HashMap;
use crate::{context::http::HttpContext, http::entity::Response};

type ActionCallerType = dyn FnMut(HttpContext) -> Response + Sync + Send + 'static;

pub struct Route {
    pub matcher: PathMatcher,
    pub call: Box<ActionCallerType>
}

impl Route {
    pub fn new (pattern: String, action: Box<ActionCallerType>) -> Self {
        return Route {
            matcher: PathMatcher::from_pattern(pattern),
            call: action
        }
    }
}

pub struct Router {
    pub routes: Vec<Route>
}

impl Router {
    pub fn empty () -> Self {
        Router { routes: Vec::new() }
    }

    #[inline]
    pub fn register<Caller: FnMut(HttpContext) -> Response + Sync + Send + 'static> (&mut self, pattern: String, action: Caller) {
        self.routes.push(Route::new(pattern, Box::new(action)));
    }

    pub fn match_path<'a> (&'a mut self, path: &String) -> Option<(&'a mut Route, HashMap<String, String>)> {
        for route in &mut self.routes {
            if let Some(params) = route.matcher.exec(&path) {
                return Some((route, params));
            }
        }

        return None;
    }
}

enum PathPart {
    String(String),
    Variable(String, char)
}

pub struct PathMatcher(Vec<PathPart>);
impl PathMatcher {
    pub fn from_pattern (pattern: String) -> Self {
        let mut sequence = Vec::new();

        let mut tmp = String::new();
        let mut is_var = false;
        let mut is_var_end = false;

        for ch in pattern.chars() {
            if is_var {
                if ch == '}' {
                    is_var = false;
                    is_var_end = true;
                } else {
                    tmp.push(ch);
                }
            } else if ch == '{' {
                if tmp.len() != 0 {
                    sequence.push(PathPart::String(tmp));
                    tmp = String::new();
                }

                is_var = true;
            } else if is_var_end {
                sequence.push(PathPart::Variable(tmp, ch));
                tmp = String::new();
            } else {
                tmp.push(ch);
            }
        }

        if is_var_end {
            sequence.push(PathPart::Variable(tmp, '\0'));
        } else if tmp.len() != 0 {
            sequence.push(PathPart::String(tmp));
        }

        return PathMatcher(sequence);
    }

    pub fn exec (&self, path: &String) -> Option<HashMap<String, String>> {
        let mut offset = 0usize;
        let mut path_iter = path.chars();
        let mut params: HashMap<String, String> = HashMap::new();

        for part in &self.0 {
            match part {
                PathPart::String(value) => {
                    let mut part_iter = value.chars();
                    loop {
                        if let Some(ch) = part_iter.next() {
                            let next_ch = path_iter.next();
                            if next_ch.is_none() || ch != next_ch.unwrap() {
                                return None;
                            }
                        } else {
                            if let None = path_iter.next() { break; }
                            else { return None; }
                        }
                    }

                    offset += value.len();
                }
                PathPart::Variable(name, stop_char) => {
                    let mut i = 0usize;
                    loop {
                        let next = path_iter.next();
                        if next.is_none() || next.unwrap() == *stop_char { break }
                        i += 1;
                    }

                    if offset >= path.len() { return None; }
                    let value = &path[(offset)..(offset + i)];
                    offset += i + 1;
					params.insert(name.to_string(), value.to_string());
                }
            }
        }

        return Some(params);
    }
}
