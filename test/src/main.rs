use std::sync::{Mutex, MutexGuard};
use dc_api_core::{app::{App, config::config_path}, http::codes::HttpCode};

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
        let app = App::new();
        APP = Some(Mutex::new(app));
    }

    {
        println!("{}", config_path("hello"));

        let mut app = get_app();
        app.router.register("/test-endpoint/ctx".to_string(), |ctx| {
            let msg = format!("{:#?}", ctx);
            return ctx.text(&msg);
        });

        app.router.register("/test-endpoint/ip".to_string(), |ctx| {
            let msg = format!("{:?}", ctx.address);
            return ctx.text(&msg);
        });

        app.router.register("/test-endpoint/headers".to_string(), |mut ctx| {
            let hostname = ctx.get_header_default("host", "none".to_string());
            ctx.set_header("x-echo-host", hostname);
            return ctx.text("Check headers!");
        });

        app.router.register("/test-endpoint/{sup}-{sub}".to_string(), |ctx| {
            let msg = format!("{:?}", ctx.params);
            return ctx.text(&msg);
        });

        app.router.register("/test-endpoint/404".to_string(), |ctx| {
            return ctx.text_status("Nothing there!", HttpCode::NotFound);
        });

        app.router.register("/test-endpoint/redirect".to_string(), |ctx| {
            return ctx.redirect("./redirected");
        });

        app.ws_endpoints.register("/socket", "test-event", |ctx| {
            ctx.text("reply", "Event handled!");
        });
    }

    dc_api_core::spawn_server(get_app_mutex());
}
