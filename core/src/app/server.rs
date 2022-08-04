use std::io::Error;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Mutex;
use threadpool::ThreadPool;

use super::{AppHandler, App};
use crate::context::http::HttpContext;
use crate::context::ws::SocketContext;
use crate::http::{entity::*, codes::HttpCode};
use crate::http1::{Http1Engine, Http1Connection};
use crate::websocket::{websocket_handshake, HandshakeResult, maintain_websocket};

pub fn start_server<Handler: AppHandler> (app_mutex: &'static Mutex<App>) {
    let app = app_mutex.lock().unwrap();

	match TcpListener::bind(app.settings.bind_address) {
        Ok(listener) => {
            drop(app);

            let pool = ThreadPool::new(32);
			Handler::on_listen(None);

			loop {
				let socket = listener.accept().unwrap();
				pool.execute(move || proceed_connection::<Handler, Http1Engine, Http1Connection>(app_mutex, socket));
			}
		}
		Err(error) => {
			Handler::on_listen(Some(error));
		}
	}
}

fn proceed_connection<Handler: AppHandler, Http: HttpEngine<Connection> + Send, Connection: HttpConnection>
(app_arc: &Mutex<App>, socket: (TcpStream, SocketAddr)) {
    let mut connection = Http::handle_connection(socket);

    match connection.parse() {
        ParsingResult::Complete(req) => {
            if is_connection_upgrade(&req) {
                if is_websocket_upgrade(&req) {
                    proceed_websocket::<Handler, Connection>(app_arc, connection, req);
                } else {
                    let _ = connection.respond(Response::from_status(HttpCode::BadRequest));
                }
            } else {
                let _ = proceed_http::<Handler, Connection>(app_arc, connection, req);
            }
        }
        ParsingResult::Partial => {}
        ParsingResult::Error(res_code) => {
            let _ = connection.respond(Response::from_status(res_code));
            let _ = connection.disconnect();
        }
        ParsingResult::Invalid => {
            let _ = connection.disconnect();
        }
    }
}

fn proceed_http<Handler: AppHandler, Connection: HttpConnection> (app_mutex: &Mutex<App>, mut connection: Connection, req: Request) -> Result<(), Error> {
    let res;
    let mut app = app_mutex.lock().unwrap();
    if let Some((endpoint, params)) = app.router.match_path(&req.path) {
        let ctx = HttpContext::from(req, params);
        res = (endpoint.call)(ctx);
    } else {
        res = Response::from_code(HttpCode::NotFound, "API endpoint not found");
    }

    connection.respond(res)?;
    return connection.disconnect();
}

fn proceed_websocket<Handler: AppHandler, Connection: HttpConnection> (app_mutex: &Mutex<App>, mut connection: Connection, req: Request) {
    match websocket_handshake(app_mutex, &req) {
        HandshakeResult::Ok(endpoint_index, res) => {
            // todo: handle all `let _ = ...`
            let _ = connection.respond(res);
            let ctx = SocketContext::from::<Connection>(connection, req);
            let _ = maintain_websocket(app_mutex, ctx, endpoint_index);
        }
        HandshakeResult::Err(res) => {
            let _ = connection.respond(res);
            let _ = connection.disconnect();
        }
    }
}

fn is_connection_upgrade (req: &Request) -> bool {
    matches!(req.headers.get("connection"), Some(value) if value == "Upgrade")
}

fn is_websocket_upgrade (req: &Request) -> bool {
    matches!(req.headers.get("upgrade"), Some(value) if value == "websocket")
}