#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::{Template};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use tokio::join;
use tokio::task::JoinHandle;
use js8event::pubsub::*;
use js8event::event::*;
use js8event::message::*;

fn get_redis_address() -> String {
    "redis://10.0.4.109:6379".to_string()
}

fn get_web_address() -> (String, u16) {
    ("127.0.0.1".to_string(), 8000)
}

async fn subscribe(redis_address: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        trace!(">>subscribe");
        let pubsub = JS8RedisPubSub::new(redis_address);
        pubsub.subscribe(|event: Event| {
            if *event.message_type() == MessageType::RxActivity {
                let message = RxActivity::try_from(event);
                trace!("WebServer: {:?}", message);

            } else if *event.message_type() == MessageType::RxSpot {
                let message = RxSpot::try_from(event);
                trace!("WebServer: {:?}", message);
            } else if *event.message_type() == MessageType::RxDirected {
                let message = RxDirected::try_from(event);
                trace!("WebServer: {:?}", message);
            } else {
                trace!("WebServer: {}", event.message_type());
            }
        }).unwrap();
        trace!("<<subscribe");
    })
}

#[get("/")]
fn index() -> Template {
    trace!(">>index");
    let context: HashMap<&str, &str> = [("name", "Jonathan")]
        .iter().cloned().collect();
    let t = Template::render("index", &context);

    trace!("<<index");
    return t;
}

#[rocket::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let redis_address = get_redis_address();
    let web = get_web_address();

    let pub_handle = subscribe(redis_address);

    let figment = rocket::Config::figment()
        .merge(("address", web.0))
        .merge(("port", web.1));

    trace!("Start web server");
    let rocket_handle = rocket::custom(figment)
        .mount("/", routes![index])
        .mount("/", FileServer::from(relative!("/")))
        .attach(Template::fairing())
    .launch();

    join!(pub_handle, rocket_handle);
}