#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::{Template};
use tokio::join;
use tokio::sync::oneshot;
use js8event::event::*;
use std::net::SocketAddr;

/// Provided by the requester and used by the manager task to send the command
/// response back to the requester.
type Responder<T> = oneshot::Sender<redis::RedisResult<T>>;

pub struct JS8Msg {
    event: Event,
    resp: Responder<()>,
}

impl JS8Msg {
    fn get_event(&self) -> &Event {
        &self.event
    }
}

mod config;
mod pubsub;
mod views;
mod websocket;

#[rocket::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let redis_address = config::get_redis_address();
    let web_server_address = config::get_web_server_address();
    let web_socket_address = config::get_web_socket_address();


    let socket_handle = websocket::start_websocket(redis_address, web_socket_address);

    let web_server_socket: SocketAddr = web_server_address.parse().unwrap();

    let figment = rocket::Config::figment()
        .merge(("address", web_server_socket.ip()))
        .merge(("port", web_server_socket.port()));

    trace!("Start web server");
    let views = views::views_factory();
    let rocket_handle = rocket::custom(figment)
        .mount("/", views)
        .mount("/", FileServer::from(relative!("/")))
        .attach(Template::fairing())
        .launch();

    let (_, _) = join!(socket_handle, rocket_handle);
}