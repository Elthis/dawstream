use std::{collections::HashSet, rc::Rc};

use dawlib::{InstrumentPayloadDto, SoundOutputPacket};
use futures::{StreamExt, stream::SplitSink, SinkExt, lock::Mutex};
use gloo_net::websocket::{Message, futures::WebSocket};
use serde::{Serialize, Deserialize};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{WorkerLink, Public, HandlerId};
use gloo_console::log;

pub struct AudioStreamingWorker {
    _link: WorkerLink<Self>,
    write_socket: Rc<Mutex<SplitSink<WebSocket, gloo_net::websocket::Message>>>,
    listeners: Rc<Mutex<HashSet<HandlerId>>>
}

#[derive(Serialize, Deserialize)]
pub enum AudioStreamingWorkerInput {
    SendInstrument(InstrumentPayloadDto)
}

#[derive(Serialize, Deserialize)]
pub enum AudioStreamingWorkerOutput {
    Chunk(Vec<Vec<f32>>),
    End
}

impl yew_agent::Worker for AudioStreamingWorker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = AudioStreamingWorkerInput;
    type Output = AudioStreamingWorkerOutput;

    fn create(link: WorkerLink<Self>) -> Self {
        let ws = WebSocket::open("ws://localhost:3000/ws").unwrap();
        let (write_socket, mut read_socket) = ws.split();
        let listeners = Rc::new(Mutex::new(HashSet::new()));
        
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
                                    let sound = SoundOutputPacket::try_from((bytes, 44100)).expect("Sumfin went rong");
                                    match sound {
                                        SoundOutputPacket::End { channel_data, ..} => {
                                            if let Some(channel_data) = channel_data {
                                                let data = match channel_data {
                                                    dawlib::ChannelData::Mono(data) => {
                                                        vec![data.clone(), data]
                                                    },
                                                    dawlib::ChannelData::Stereo(first_channel, second_channel) => {
                                                        vec![first_channel, second_channel]
                                                    },
                                                };
                                                for listener in listeners.lock().await.iter() {
                                                    link.respond(*listener, AudioStreamingWorkerOutput::Chunk(data.clone()))
                                                }
                                            }
                                            for listener in listeners.lock().await.iter() {
                                                link.respond(*listener, AudioStreamingWorkerOutput::End)
                                            }
                                        },
                                        SoundOutputPacket::Data { channel_data } => {
                                            let data = match channel_data {
                                                dawlib::ChannelData::Mono(data) => {
                                                    vec![data.clone(), data]
                                                },
                                                dawlib::ChannelData::Stereo(first_channel, second_channel) => {
                                                    vec![first_channel, second_channel]
                                                },
                                            };

                                            for listener in listeners.lock().await.iter() {
                                                link.respond(*listener, AudioStreamingWorkerOutput::Chunk(data.clone()))
                                            }
                                        },
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
        

        Self { _link: link, write_socket: Rc::new(Mutex::new(write_socket)), listeners }
    }

    fn connected(&mut self, id: HandlerId) {
        let listeners = self.listeners.clone();
        spawn_local(async move {
            listeners.lock().await.insert(id);
        })
    }

    fn disconnected(&mut self, id: HandlerId) {
        let listeners = self.listeners.clone();
        spawn_local(async move {
            listeners.lock().await.remove(&id);
        })
    }

    fn update(&mut self, _msg: Self::Message) {
        // no messaging
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            AudioStreamingWorkerInput::SendInstrument(payload) => {
                let write_socket = self.write_socket.clone();

                spawn_local(async move {
                    write_socket.lock().await.send(Message::Text(serde_json::to_string(&payload).unwrap())).await.unwrap();
                    log!("Sent Something")
                });
                
            },
        }
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}