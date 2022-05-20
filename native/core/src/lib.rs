mod http1;
mod http;
mod utils;
mod routing;
mod context;
mod parsers;
mod websocket;

#[macro_use]
extern crate napi_derive;

use context::websocket::{WebSocketConnection, ControllerSocketContext};
use futures::future::{Abortable, AbortHandle};
use http::entity::{BoxedHttpConnection, Request};
use napi::{Env, JsFunction, JsObject, JsUnknown, JsNull, Result, Status, Ref, Either, NapiRaw};
use napi::bindgen_prelude::{Undefined, ToNapiValue};
use napi::threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode, ErrorStrategy};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use utils::js::{create_dummy_tsfn, JsDummyResult};
use websocket::{WebSocketEndpoints, DispatchWSMessageTuple};
use crate::http1::Http1Engine;
use crate::http::{ParsedHttpConnection, proceed_connection};
use crate::routing::Router;
use crate::utils::callers::ActionCaller;
use crate::utils::json::JSON;
use crate::utils::macros::js_err;
use std::net::IpAddr;
use std::ops::Deref;


pub static mut APP: Option<App> = None;
#[inline]
pub fn get_app () -> &'static mut App {
    return unsafe { APP.as_mut().unwrap() };
}

pub struct UnsafeEnv(Env);
unsafe impl Send for UnsafeEnv {}
unsafe impl Sync for UnsafeEnv {}
impl Deref for UnsafeEnv {
    type Target = Env;

    fn deref (&self) -> &Self::Target {
        return &self.0;
    }
}
impl Clone for UnsafeEnv {
    fn clone (&self) -> Self {
        Self(self.0.clone())
    }
}
impl UnsafeEnv {
    pub fn from_tsctx<T> (ctx: ThreadSafeCallContext<T>) -> Self { Self(ctx.env) }
    pub fn unwrap (self) -> Env { self.0 }
}

pub struct App {
    json: JSON,
    dummy_fn: JsFunction,
    create_object: JsFunction,
    listener_handle: Option<AbortHandle>,
    patch_context: Ref<()>,

    router: Router,
    dispatch_request: ThreadsafeFunction<ParsedHttpConnection>,

    socket_endpoints: WebSocketEndpoints,
    v5_create_ws_context: ThreadsafeFunction<(IpAddr, Request, usize, Box<dyn FnOnce(&Env, Ref<()>)>)>,
    dispatch_websocket_message: ThreadsafeFunction<DispatchWSMessageTuple>,
    websocket_connections: Vec<WebSocketConnection>
    // dispatch_websocket: ThreadsafeFunction<ParsedHttpConnection>
}

impl App {
    fn new (env: Env, patch_context: JsFunction) -> Result<Self> {
        return Ok(App {
            json: JSON::init(env)?,
            dummy_fn: env.create_function_from_closure("dummy", |ctx| ctx.get::<JsUnknown>(0))?,
            create_object: env.create_function_from_closure("create_object", |ctx| ctx.env.create_object())?,
            //create_dummy_tsfn(&env, "create_object", App::create_object)?,
            listener_handle: None,
            dispatch_request: create_dummy_tsfn(&env, "dispatch_request", App::dispatch_request)?,
            // dispatch_websocket: create_dummy_tsfn(&env, "dispatch_websocket", App::dispatch_websocket)?,
            patch_context: env.create_reference(patch_context)?,

            v5_create_ws_context: create_dummy_tsfn(&env, "v5_create_ws_context", App::v5_create_ws_context)?,
            dispatch_websocket_message: create_dummy_tsfn(&env, "dispatch_websocket_message", App::dispatch_websocket_message)?,
            socket_endpoints: WebSocketEndpoints::empty(),
            websocket_connections: vec![],
            router: Router::empty()
        });
    }

    // pub fn provide_env<R: 'static + NapiRaw> (&self, action: Box<dyn FnMut(&Env) -> Result<R> + Send>) -> Result<Ref<()>> {
    //     let result_ref;
    //     let tsfn: ThreadsafeFunction<(), ErrorStrategy::CalleeHandled> = self.dummy_fn.create_threadsafe_function(1,  move|ctx: ThreadSafeCallContext<()>| -> JsDummyResult {
    //         let result = action(&ctx.env);
    //         result_ref = match result {
    //             Ok(value) => Ok(ctx.env.create_reference(value)?),
    //             Err(error) => Err(error)
    //         };
    //         Ok(vec![])
    //     })?;
    //     tsfn.call(Ok(()), ThreadsafeFunctionCallMode::Blocking);
    //     return result_ref;
    // }

    pub fn v5_create_ws_context (ctx: ThreadSafeCallContext<(IpAddr, Request, usize, Box<dyn FnOnce(&Env, Ref<()>)>)>) -> JsDummyResult {
        let (address, req, endpoint, callback) = ctx.value;
        let controller_ctx = ControllerSocketContext {
            req_headers: req.headers,
            query_string: req.query,
            address
        };

        let controller = get_app().socket_endpoints.at(endpoint).map(|e| &e.controller);
        let controller_ctx = controller_ctx.into_js(&ctx.env, controller)?;
        callback(&ctx.env, ctx.env.create_reference(controller_ctx)?);

        Ok(vec![])
    }

    pub fn dispatch_websocket_message (ctx: ThreadSafeCallContext<DispatchWSMessageTuple>) -> JsDummyResult {
        match ctx.value.msg {
            Message::Text(ref msg) => {
                let mut iter = msg.splitn(2, ':');
                let event_name = match iter.next() {
                    Some(value) => value,
                    None => return Ok(vec![])
                };

                match iter.next() {
                    Some(payload) => {
                        // todo: catch errors
                        let payload = get_app().json.parse(&ctx.env, payload).unwrap();
                        println!("event {}", event_name);
                        let endpoint = ctx.value.get_endpoint();
                        println!("endpoint path {}", endpoint.path);
                        if let Some(handler) = endpoint.get_handler(event_name) {
                            println!("handler matched");
                            let action: JsFunction = ctx.env.get_reference_value_unchecked(&handler.method).unwrap();
                            action.call(None, &[payload]);
                            // handler.method
                        } else {
                            println!("no handler");
                            // todo: no handler message?
                        }
                        // (ctx.value.proceed)(event_name, payload);
                        // ctx.value.reply
                        // println!("{} with payload {}", event_name, payload);
                    },
                    None => {
                        println!("{} without payload", event_name);
                    }
                };
            }
            _ => {}
        }

        return Ok(vec![]);
    }

    pub fn create_object (&self) -> JsObject {
        unsafe { self.create_object.call_without_args(None).unwrap().cast() }
    }

    pub fn dispatch_request (ctx: ThreadSafeCallContext<ParsedHttpConnection>) -> JsDummyResult {
        let mut connection = ctx.value;
        let dispatch_result = get_app().router.dispatch(&mut connection, &ctx.env);

        ctx.env.execute_tokio_future(
            async move { connection.respond_js(dispatch_result).await },
            |env, _| env.get_undefined()
        )?;
        return Ok(vec![]);
    }

    // pub fn dispatch_websocket (ctx: ThreadSafeCallContext<ParsedHttpConnection>) -> JsDummyResult {
    //     let mut connection = ctx.value;
    //     let env = ctx.env;
    //     // let env_scope = UnsafeEnv(ctx.env);
    //     // let env_execute = env_scope.clone();

    //     match dispatch_websocket(&mut connection) {
    //         Ok((endpoint, res)) => {
    //             // let env = UnsafeEnv::from_tsctx(ctx);
    //             let env2 = env.clone();
    //             // let env_maintain = env_execute.clone();
    //             env.execute_tokio_future(
    //                 async move {
    //                     connection.respond_js(res).await?;
    //                     return maintain_websocket(env, connection, endpoint).await;
    //                 },
    //                 |env, _| env.get_undefined()
    //             )?;
    //         }
    //         Err(res) => {
    //             ctx.env.execute_tokio_future(
    //                 async move { connection.respond_js(res).await },
    //                 |env, _| env.get_undefined()
    //             )?;
    //         }
    //     }

    //     return Ok(vec![]);
    // }
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

    let promise = future_to_promise!(env, move abortable)?;
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
fn register_route (env: Env, pattern: String, controller: Either<JsObject, JsNull>, handler: JsFunction) -> Result<Undefined> {
    let caller = ActionCaller::new(
        env,
        handler,
        match controller {
            Either::A(obj) => Some(env.create_reference(obj)?),
            Either::B(_) => None
        }
    );

    get_app().router.register(pattern, caller);
    return Ok(());
}

#[allow(dead_code)]
#[napi]
fn register_socket (env: Env, endpoint: JsObject) -> Result<Undefined> {
    get_app().socket_endpoints.push(&env, endpoint)
}
