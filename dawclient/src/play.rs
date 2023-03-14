use std::{rc::Rc, sync::{RwLock}};

use gloo_console::{__macro::JsValue, log};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{AudioBufferSourceNode, AudioNode, AudioContext};

use yew_agent::use_bridge;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{worker::{AudioStreamingWorker, AudioStreamingWorkerInput, AudioStreamingWorkerOutput}, instrument::InstrumentState};

pub enum Message {
    Click,
    WorkerMsg(AudioStreamingWorkerOutput),
}

#[derive(Debug, Clone, PartialEq)]
struct AudioStreamer {
    ctx: AudioContext,
    chunks: Vec<Vec<Vec<f32>>>,
    scheduled: Vec<AudioBufferSourceNode>,
    on_end: Option<Callback<(), ()>>
}

impl Drop for AudioStreamer {
    fn drop(&mut self) {
        self.ctx.close().unwrap().is_undefined();
    }
}

pub const CHUNK_LENGTH: u32 = 1;


impl AudioStreamer {
    fn empty() -> Result<Self, JsValue> {
        let ctx = web_sys::AudioContext::new()?;
        log!("Sample Rate: ", ctx.sample_rate());

        Ok(Self {
            ctx,
            chunks: vec![],
            scheduled: vec![],
            on_end: None
        })
    }

    fn state(&self) -> StreamerState {
        if !self.scheduled.is_empty() {
            StreamerState::Playing
        } else if !self.chunks.is_empty() {
            StreamerState::Waiting
        } else {
            StreamerState::Empty
        }
    }

    fn set_on_ended(&mut self, callback: Callback<(), ()> ) {
        self.on_end = Some(callback);
    }

    fn push_chunk(&mut self, chunk: Vec<Vec<f32>>) {
        self.chunks.push(chunk);
    }


    fn play(&mut self) -> Result<(), JsValue> {
        self.stop()?;
        let current_time = self.ctx.current_time();
        let last_chunk_index = self.chunks.len() - 1;
        for (index, chunk) in self.chunks.iter().enumerate() {
            let audio_buffer =
            self.ctx.create_buffer(2, (self.ctx.sample_rate()) as u32 * CHUNK_LENGTH, self.ctx.sample_rate())?;

            for (channel_number, channel_data) in chunk.iter().enumerate() {
                audio_buffer.copy_to_channel_with_start_in_channel(channel_data, channel_number as i32, 0)?;
            }

            let buffer_source = self.ctx.create_buffer_source().unwrap();
            buffer_source.set_buffer(Some(&audio_buffer));
            buffer_source.connect_with_audio_node(
                wasm_bindgen::JsCast::dyn_ref::<AudioNode>(&self.ctx.destination()).unwrap(),
            ).unwrap();
            let on_end = self.on_end.clone();

            if index == last_chunk_index {
                let cb: Closure<dyn FnMut() -> Result<(), JsValue>> = Closure::new(move || {
                    if let Some(on_end) = &on_end {
                        on_end.emit(());
                    }         
                    Ok(())
                });
                buffer_source
                    .add_event_listener_with_callback("ended", cb.as_ref().unchecked_ref()).unwrap();
                cb.forget();
            }
            
            
            buffer_source.start_with_when(index as f64 * CHUNK_LENGTH as f64 + current_time).unwrap();
            
            self.scheduled.push(buffer_source);
        }

        self.chunks.clear();

        Ok(())
    }

    fn stop(&mut self) -> Result<(), JsValue> {
        for node in &self.scheduled {
            node.stop()?;
        }
        self.scheduled.clear();
        Ok(())
    }
}


enum StreamerState {
    Empty,
    Playing,
    Waiting
}


#[function_component(PlayButtonComponent)]
pub fn play_button() -> Html {
    let audio_streamer_state = use_state(|| StreamerState::Empty);
    let audio_streamer = use_state(|| Rc::new(RwLock::new(AudioStreamer::empty().unwrap())));

    {
        let audio_streamer_state = audio_streamer_state.clone();
        let audio_streamer_handle = audio_streamer.clone();
        audio_streamer.write().unwrap().set_on_ended(Callback::from(move |_| {
            audio_streamer_handle.write().unwrap().stop().unwrap();
            audio_streamer_state.set(audio_streamer_handle.read().unwrap().state());
        }));
    }

    let instruments = use_store_value::<InstrumentState>();
    let worker_bridge = {
        let audio_streamer = audio_streamer.clone();
        let audio_streamer_state = audio_streamer_state.clone();
        use_bridge::<AudioStreamingWorker, _>(move |response| {
            match response {
                AudioStreamingWorkerOutput::Chunk(chunk) => {
                    audio_streamer.write().unwrap().push_chunk(chunk);
                    audio_streamer_state.set(audio_streamer.read().unwrap().state());
                }
            }
        })
    };

    match *audio_streamer_state {
        StreamerState::Empty => {
            let download = move |_| {
                worker_bridge.send(AudioStreamingWorkerInput::SendInstrument(
                    instruments.as_ref().clone().into()
                ));
            };
            html! {
                <button class={format!("bg-transparent text-white font-semibold py-0 px-1 border border-gray-500 rounded h-7 w-7 hover:bg-gray-500 hover:border-transparent")} onclick={download}> {"↓"} </button>
            }
        },
        StreamerState::Playing => {
            let stop = move |_| {
                audio_streamer.write().unwrap().stop().unwrap();
                audio_streamer_state.set(audio_streamer.read().unwrap().state());
            };
            html! {
                <button class="bg-transparent hover:bg-gray-500 text-sm text-white font-semibold py-0 px-1 border border-gray-500 hover:border-transparent rounded h-7 w-7" onclick={stop}> {"■"} </button>
            }
        },
        StreamerState::Waiting => {
            let play = move |_| {
                audio_streamer.write().unwrap().play().unwrap();
                audio_streamer_state.set(audio_streamer.read().unwrap().state());
            };
            html! {
                <button class={format!("bg-transparent text-white font-semibold py-0 px-1 border border-gray-500 rounded h-7 w-7 hover:bg-gray-500 hover:border-transparent")} onclick={play}> {"⏵"} </button>
            }
        },
    }
}