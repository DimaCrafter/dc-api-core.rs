use napi::{Env, JsObject};
use crate::context::http::{create_http_context, ControllerHttpContext, serialize_object};
use crate::http::ParsedHttpConnection;
use crate::http::entity::Response;
use crate::http::codes::HttpCode;
use crate::ActionCaller;

pub struct Router {
    pub routes: Vec<Route>
}

impl Router {
    pub fn empty () -> Self {
        return Router {
          routes: Vec::new()
        };
    }

    #[inline]
    pub fn register (&mut self, pattern: String, caller: ActionCaller) {
        self.routes.push(Route::new(pattern, caller));
    }

    pub fn match_path<'a> (&'a self, path: &String, env: &Env) -> Option<(&'a Route, JsObject)> {
        for route in &self.routes {
            if let Some(params) = route.matcher.exec(&path, env) {
                return Some((route, params));
            }
        }

        return None;
    }

    pub fn dispatch (&self, connection: &mut ParsedHttpConnection, env: &Env) -> Response {
        let req = connection.req.as_ref().unwrap();
        if let Some((route, params)) = self.match_path(&req.path, env) {
            let mut js_ctx = create_http_context(env, connection, route.caller.owner.as_ref()).unwrap();
            js_ctx.set_named_property("params", params).unwrap();

            match route.caller.call(env, &js_ctx) {
                Ok(call_result) => {
                    let ctx: &mut Option<ControllerHttpContext> = env.unwrap(&js_ctx).unwrap();
                    let mut ctx = ctx.take().unwrap();
                    serialize_object(env, &mut ctx, call_result);
                    return ctx.into_response();
                }
                Err(err) => {
                    return Response::from_error(err);
                }
            }
        }

        return Response::from_code(HttpCode::NotFound, "API endpoint not found");
    }
}

pub struct Route {
    pub matcher: PathMatcher,
    pub caller: ActionCaller
}

impl Route {
    pub fn new (pattern: String, caller: ActionCaller) -> Self {
        return Route {
            matcher: PathMatcher::from_pattern(pattern),
            caller
        }
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

    pub fn exec (&self, path: &String, env: &Env) -> Option<JsObject> {
        let mut offset = 0usize;
        let mut path_iter = path.chars();
        let mut params = env.create_object().unwrap();

        for part in &self.0 {
            match part {
                PathPart::String(value) => {
                    let mut part_iter = value.chars();
                    loop {
                        if let Some(ch) = part_iter.next() {
                            let next_ch = path_iter.next();
                            if next_ch.is_none() || ch != next_ch.unwrap() {
                                return None
                            }
                        } else {
                            break;
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

                    let value = &path[(offset)..(offset + i)];
                    offset += i + 1;
                    params.set_named_property(name.as_str(), env.create_string(value).unwrap());
                }
            }
        }

        return Some(params);
    }
}
