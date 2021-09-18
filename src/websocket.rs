

use futures::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::{Error, Message, Result}};
use super::JS8Msg;
use super::pubsub;


async fn accept_connection(peer: String, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(redis_address: String, stream: TcpStream) -> Result<()> {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    // create event queue to send between JS8 listener and web socket handler
    let (tx, mut rx): (mpsc::Sender<JS8Msg>, mpsc::Receiver<JS8Msg>) = mpsc::channel(32);

    // start listening to JS8 messages from the JS8-Monitor
    pubsub::subscribe(redis_address, tx).await;

    // Open web socket stream
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);
    let (mut write, _) = ws_stream.split();

    while let Some(msg) = rx.recv().await {

        let json: String = serde_json::to_string(msg.get_event()).unwrap();
        trace!("Received JS8 message: {}", json);
        let event = Message::Text(json);

        trace!("Send WebSocket: {}", event);
        let _response = write.send(event).await;

        let _result = msg.resp.send(Ok(()));
    }

    Ok(())
}

pub async fn start_websocket(redis_address: String, socket_address: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind(&socket_address).await.expect("Failed to bind");
        info!("Listening on: {}", socket_address);

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(accept_connection(redis_address.clone(), stream));
        }
    })
}
