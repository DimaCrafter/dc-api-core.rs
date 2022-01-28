mod http1;
mod http;
mod utils;
mod routing;
mod context;
mod parsers;

#[macro_use]
extern crate napi_derive;

use std::collections::HashMap;
use futures::future::{Abortable, AbortHandle};
use napi::{Env, JsFunction, JsObject, JsUnknown, Result, Status, Ref};
use napi::bindgen_prelude::Undefined;
use napi::threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use tokio::net::TcpListener;
use crate::http1::Http1Engine;
use crate::http::{ParsedHttpConnection, proceed_connection};
use crate::routing::Router;
use crate::utils::callers::ActionCaller;
use crate::utils::camel_to_kebab;
use crate::utils::controller::Controller;
use crate::utils::json::JSON;
use crate::utils::macros::js_err;


pub static mut APP: Option<App> = None;
#[inline]
pub fn get_app () -> &'static mut App {
    return unsafe { APP.as_mut().unwrap() };
}

pub struct App {
    json: JSON,
    listener_handle: Option<AbortHandle>,

    router: Router,
    controllers: HashMap<String, Controller>,
    dispatch_request: ThreadsafeFunction<ParsedHttpConnection>,
    patch_context: Ref<()>
}

impl App {
    fn new (env: Env, patch_context: JsFunction) -> Result<Self> {
        let dispatch_request_closure = env.create_function_from_closure(
            "dispatch_request",
            |ctx| ctx.get::<JsUnknown>(0)
        )?;
        let dispatch_request = dispatch_request_closure.create_threadsafe_function(0, App::dispatch_request)?;

        return Ok(App {
            json: JSON::init(env)?,
            listener_handle: None,
            dispatch_request,
            patch_context: env.create_reference(patch_context)?,

            controllers: HashMap::new(),
            router: Router::empty()
        });
    }

    pub fn dispatch_request (ctx: ThreadSafeCallContext<ParsedHttpConnection>) -> Result<Vec<JsUnknown>> {
        let mut connection = ctx.value;
        let dispatch_result = get_app().router.dispatch(&mut connection, &ctx.env);

        ctx.env.execute_tokio_future(
            async move { connection.respond(dispatch_result).await; Ok(()) },
            |env, _| env.get_undefined()
        )?;

        return Ok(Vec::new());
    }
}

#[allow(dead_code)]
#[napi]
fn start_app (env: Env, bind_address: String, on_listen: JsFunction, patch_context: JsFunction) -> Result<JsObject> {
    unsafe {
        if APP.is_some() {
            return js_err(Status::GenericFailure, "Server already started");
        }

        APP = Some(App::new(env, patch_context)?);
    }

    let on_listen = env.create_threadsafe_function(
        &on_listen,
        0,
        |_ctx| Ok(Vec::<JsUnknown>::new())
    )?;

    let connection_loop = async move {
        match TcpListener::bind(bind_address).await {
            Ok(listener) => {
                on_listen.call(Ok(()), ThreadsafeFunctionCallMode::NonBlocking);

                loop {
                    let socket = listener.accept().await.unwrap();
                    tokio::spawn(proceed_connection::<Http1Engine>(socket));
                }
            }
            Err(error) => {
                println!("Listen error: {}", error.to_string());
            }
        }
    };

    let (listener_handle, abort_registration) = AbortHandle::new_pair();
    let abortable = Abortable::new(connection_loop, abort_registration);

    let promise = env.execute_tokio_future(
        async { abortable.await; Ok(()) },
        |env: &mut Env, _| env.get_undefined()
    )?;

    get_app().listener_handle = Some(listener_handle);
    return Ok(promise);
}

#[allow(dead_code)]
#[napi]
fn stop_app () {
    let app = get_app();
    if let Some(stop_token) = &app.listener_handle {
        stop_token.abort();
        app.listener_handle = None;
    }
}

#[allow(dead_code)]
#[napi]
fn register_controller (env: Env, controller_name: String, controller_class: JsFunction) -> Result<Undefined> {
    return get_app().register_controller(env, controller_name, controller_class);
}

#[allow(dead_code)]
#[napi]
fn register_route (env: Env, pattern: String, handler: JsUnknown) -> Result<Undefined> {
    return get_app().register_route(env, pattern, handler);
}
