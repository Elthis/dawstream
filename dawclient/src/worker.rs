use std::{sync::RwLock, collections::HashSet, rc::Rc};

use dawlib::InstrumentPayloadDto;
use futures::{StreamExt, stream::SplitSink, SinkExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use serde::{Serialize, Deserialize};
use yew::platform::spawn_local;
use yew_agent::{WorkerLink, Public, HandlerId};
use gloo_console::log;

use crate::play::CHUNK_LENGTH;

pub struct AudioStreamingWorker {
    _link: WorkerLink<Self>,
    write_socket: Rc<RwLock<SplitSink<WebSocket, gloo_net::websocket::Message>>>,
    listeners: Rc<RwLock<HashSet<HandlerId>>>
}

#[derive(Serialize, Deserialize)]
pub enum AudioStreamingWorkerInput {
    SendInstrument(InstrumentPayloadDto)
}

#[derive(Serialize, Deserialize)]
pub enum AudioStreamingWorkerOutput {
    Chunk(Vec<Vec<f32>>)
}

impl yew_agent::Worker for AudioStreamingWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = AudioStreamingWorkerInput;
    type Output = AudioStreamingWorkerOutput;

    fn create(link: WorkerLink<Self>) -> Self {
        let ws = WebSocket::open("ws://localhost:3000/ws").unwrap();
        let (write_socket, mut read_socket) = ws.split();
        let listeners = Rc::new(RwLock::new(HashSet::new()));
        
        {
            let link = link.clone();
            let listeners = listeners.clone();
            spawn_local(async move {
                while let Some(msg) = read_socket.next().await {
                    match msg {
                        Ok(msg) => {
                            match msg {
                                gloo_net::websocket::Message::Text(value) => {
                                    log!(format!("Text: {:#?}", value));
                                }
                                gloo_net::websocket::Message::Bytes(bytes) => {
                                    let samples = bytes.chunks_exact(4)
                                    .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
                                    .collect::<Vec<f32>>();
                                    let mut chunks = samples.chunks_exact(44100 * CHUNK_LENGTH as usize);
                                    
                                    let first_channel = chunks.next().unwrap().to_vec();
                                    let second_channel = chunks.next().unwrap().to_vec();
                                    let chunk = vec![first_channel, second_channel].clone();
                                    
                                    for listener in listeners.read().unwrap().iter() {
                                        link.respond(*listener, AudioStreamingWorkerOutput::Chunk(chunk.clone()))
                                    }
                                } 
                            }
                        },
                        Err(err) => {
                            log!(format!("Error: {:#?}", err))
                        },
                    }
                    
                }
                log!("WebSocket Closed")
            });
        }
        

        Self { _link: link, write_socket: Rc::new(RwLock::new(write_socket)), listeners }
    }

    fn connected(&mut self, id: HandlerId) { 
        self.listeners.write().unwrap().insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.listeners.write().unwrap().remove(&id);
    }

    fn update(&mut self, _msg: Self::Message) {
        // no messaging
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            AudioStreamingWorkerInput::SendInstrument(payload) => {
                let write_socket = self.write_socket.clone();

                spawn_local(async move {
                    write_socket.write().unwrap().send(Message::Text(serde_json::to_string(&payload).unwrap())).await.unwrap();
                    log!("Sent Something")
                });
                
            },
        }
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}