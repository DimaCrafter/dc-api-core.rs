use std::{io::Error, sync::{Mutex, MutexGuard}};
use dc_api_core::app::{Settings, AppHandler, App};

struct ServerHandler;
impl AppHandler for ServerHandler {
    fn on_listen (error: Option<Error>) {
        if let Some(error) = error {
			println!("Listen error: {}", error.to_string());
        } else {
            println!("Listening...");
        }
    }
}

static mut APP: Option<Mutex<App>> = None;
#[inline(always)]
pub fn get_app_mutex () -> &'static mut Mutex<App> {
    unsafe { APP.as_mut().unwrap() }
}
#[inline(always)]
pub fn get_app () -> MutexGuard<'static, App> {
    get_app_mutex().lock().unwrap()
}

fn main () {
    unsafe {
        let app = App::new(Settings {
            bind_address: "0.0.0.0:6080",
            debug: true
        });

        APP = Some(Mutex::new(app));
    }

    {
        let mut app = get_app();
        app.router.register("/test-endpoint/hello".to_string(), |ctx| {
            println!("{:?} {}", ctx.req.method, ctx.req.path);
            return ctx.text("Hello Postman!");
        });

        app.router.register("/test-endpoint/{sup}-{sub}".to_string(), |ctx| {
            println!("{:?}", ctx.params);
            return ctx.text_status("Hello Postman!", dc_api_core::http::codes::HttpCode::NotFound);
        });

        app.ws_endpoints.register("/socket", "test-event", |ctx| {
            ctx.text("reply", "Event handled!");
        });
    }

    dc_api_core::spawn_server::<ServerHandler>(get_app_mutex());
}
