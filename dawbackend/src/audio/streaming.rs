use std::{net::SocketAddr, time::Duration, ops::ControlFlow};
use axum::extract::ws::{WebSocket, Message};
use dawlib::{InstrumentPayloadDto, SoundOutputPacket};
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
                let mut music_box = MusicBox::new(payload.tempo, payload.instruments);
                let mut index = 0;
                let mut is_streaming = true;
                while is_streaming {
                    let output = match music_box.chunk(44100) {
                        Ok(full_chunk) => {
                            debug!("Sending chunk {index}");
                            index += 1;

                            SoundOutputPacket::Data { 
                                channel_data: dawlib::ChannelData::Mono(full_chunk) 
                            }
                        },
                        Err(partial_chunk) => {
                            is_streaming = false;
                            debug!("Sending end.");
                            let length = partial_chunk.len() as u16;
                            let channel_data = if length != 0 {
                                Some(dawlib::ChannelData::Mono(partial_chunk))
                            } else {
                                None
                            };

                            SoundOutputPacket::End { 
                                length,
                                channel_data
                            }
                        },
                    };

                    if sender
                        .send(Message::Binary(output.into()))
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