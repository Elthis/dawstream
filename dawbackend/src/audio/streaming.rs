use std::{net::SocketAddr, time::Duration, ops::ControlFlow};
use axum::extract::ws::{WebSocket, Message};
use dawlib::InstrumentPayloadDto;
use futures::{StreamExt, stream::SplitSink, SinkExt};
use tracing::{error, warn, debug};

use crate::audio::MusicBox;


pub async fn handle_connection(socket: WebSocket, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();

    loop {
        if let Some(msg) = receiver.next().await {
            debug!("Received message.");
            if let Ok(msg) = msg {
                if process_message(msg, who, &mut sender).await.is_break() {
                    return;
                }
            } else {
                error!("Client {who} abruptly disconnected");
                return;
            }
        } else {
            println!("Nothing");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

async fn process_message(msg: Message, who: SocketAddr, sender: &mut SplitSink<WebSocket, Message>) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            if let Ok(payload) = serde_json::from_str::<InstrumentPayloadDto>(&t) {
                let mut music_box = MusicBox::new(payload.instruments);
                let mut index = 0;
                while let Ok(chunk) = music_box.chunk::<44100>() {
                    let mut bytes = chunk.into_iter()
                    .flat_map(|it| it.to_le_bytes())
                    .collect::<Vec<u8>>();


                    bytes.append(&mut bytes.clone());

                    debug!("Sending chunk {index}");
                
                    index += 1;
            
                        
                    if sender
                        .send(Message::Binary(bytes))
                        .await
                        .is_err()
                    {
                        return ControlFlow::Break(());
                    }
                }          
            } else {
                warn!(">>> {} sent invalid payload: {:?}", who, t);
                return ControlFlow::Break(());
            }
        }
        Message::Binary(d) => {
            debug!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                debug!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                error!(">>> {} somehow sent close message without CloseFrame", who);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            debug!(">>> {} sent pong with {:?}", who, v);
        }

        Message::Ping(v) => {
            debug!(">>> {} sent ping with {:?}", who, v);
        }
    }
    ControlFlow::Continue(())
}