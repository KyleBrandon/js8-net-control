use tokio::task::JoinHandle;
use tokio::sync::{mpsc, oneshot};
use js8event::pubsub::*;
use js8event::event::*;
use super::JS8Msg;

pub async fn subscribe(redis_address: String, sender: mpsc::Sender<JS8Msg>) -> JoinHandle<()> {
    tokio::spawn(async move {
        trace!(">>subscribe");

        let pubsub = JS8RedisPubSub::new(redis_address);

        pubsub.subscribe(|event: Event| {
            trace!("Received event from JS8 Monitor: {}", event.message_type());

            // clone the web socket sender
            let sender_clone = sender.clone();

            // Send to web socket handler
            tokio::spawn(async move {
                let (resp_tx, resp_rx) = oneshot::channel();
                let e1 = JS8Msg {
                    event: event,
                    resp: resp_tx
                };

                if let Err(e) = sender_clone.send(e1).await {
                    error!("Error sending to web socket handler: {}", e);
                }

                if let Err(e) = resp_rx.await {
                    error!("Error from web socket handler: {}", e);
                }
            });

        }).unwrap();
        trace!("<<subscribe");
    })
}