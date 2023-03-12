use std::{collections::VecDeque, rc::Rc, sync::{RwLock, atomic::AtomicBool}, time::Duration};

use futures::StreamExt;
use gloo_console::{__macro::JsValue, log};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioNode, HtmlInputElement, OscillatorType, AudioContext};
use yew::{prelude::*, platform::time::interval};
use yew_agent::{Bridge, Bridged, use_bridge};
use yewdux::{prelude::{use_store, use_store_value}, store::Store};

use crate::{worker::{AudioStreamingWorker, AudioStreamingWorkerInput, AudioStreamingWorkerOutput}, instrument::InstrumentState};

pub enum Message {
    Click,
    WorkerMsg(AudioStreamingWorkerOutput),
}

#[derive(Debug, Clone, PartialEq)]
struct AudioStreamer {
    ctx: AudioContext,
    chunks: Vec<Vec<Vec<f32>>>,
    scheduled: Vec<AudioBufferSourceNode>
}

impl Drop for AudioStreamer {
    fn drop(&mut self) {
        self.ctx.close().unwrap();
    }
}

impl Default for AudioStreamer {
    fn default() -> Self {
        Self::empty().unwrap()
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
            scheduled: vec![]
        })
    }

    fn state(&self) -> StreamerState {
        if self.scheduled.len() != 0 {
            StreamerState::PLAYING
        } else if self.chunks.len() != 0 {
            StreamerState::WAITING
        } else {
            StreamerState::EMPTY
        }
    }

    fn push_chunk(&mut self, chunk: Vec<Vec<f32>>) {
        self.chunks.push(chunk);
    }


    fn play(&mut self) -> Result<(), JsValue> {
        self.stop()?;
        let current_time = self.ctx.current_time();
        
        for (index, chunk) in self.chunks.iter().enumerate() {
            let audio_buffer =
            self.ctx.create_buffer(2, (self.ctx.sample_rate()) as u32 * CHUNK_LENGTH, self.ctx.sample_rate())?;

            for (channel_number, channel_data) in chunk.iter().enumerate() {
                audio_buffer.copy_to_channel_with_start_in_channel(&channel_data, channel_number as i32, 0)?;
            }

            let buffer_source = self.ctx.create_buffer_source().unwrap();
            buffer_source.set_buffer(Some(&audio_buffer));
            buffer_source.connect_with_audio_node(
                wasm_bindgen::JsCast::dyn_ref::<AudioNode>(&self.ctx.destination()).unwrap(),
            ).unwrap();

            let cb: Closure<dyn FnMut() -> Result<(), JsValue>> = Closure::new(move || {
                log!("Ended");
                Ok(())
            });

            
            buffer_source
                .add_event_listener_with_callback("ended", cb.as_ref().unchecked_ref()).unwrap();
            buffer_source.start_with_when(index as f64 * CHUNK_LENGTH as f64 + current_time).unwrap();
            cb.forget();
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
    EMPTY,
    PLAYING,
    WAITING
}


#[function_component(PlayButtonComponent)]
pub fn play_button() -> Html {
    let audio_streamer = use_state(|| Rc::new(RwLock::new(AudioStreamer::empty().unwrap())));
    let audio_streamer_state = use_state(|| StreamerState::EMPTY);
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
        StreamerState::EMPTY => {
            let play = move |_| {
                log!("Sending");
                worker_bridge.send(AudioStreamingWorkerInput::SendInstrument(
                    instruments.as_ref().clone().into()
                ));
                audio_streamer.write().unwrap().play().unwrap();
                audio_streamer_state.set(audio_streamer.read().unwrap().state());
            };
            html! {
                <button class={format!("bg-transparent text-white font-semibold py-0 px-1 border border-gray-500 rounded h-7 w-7 hover:bg-gray-500 hover:border-transparent")} onclick={play}> {"⏵"} </button>
            }
        },
        StreamerState::PLAYING => {
            let stop = move |_| {
                audio_streamer.write().unwrap().stop().unwrap();
                audio_streamer_state.set(audio_streamer.read().unwrap().state());
            };
            html! {
                <button class="bg-transparent hover:bg-gray-500 text-sm text-white font-semibold py-0 px-1 border border-gray-500 hover:border-transparent rounded h-7 w-7" onclick={stop}> {"■"} </button>
            }
        },
        StreamerState::WAITING => {
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