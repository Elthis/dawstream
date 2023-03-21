use std::{rc::Rc, sync::{RwLock}};

use gloo_console::{__macro::JsValue, log};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{AudioBufferSourceNode, AudioNode, AudioContext};

use yew_agent::use_bridge;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{worker::{AudioStreamingWorker, AudioStreamingWorkerInput, AudioStreamingWorkerOutput}, instrument::TrackState};

pub enum Message {
    Click,
    WorkerMsg(AudioStreamingWorkerOutput),
}

#[derive(Debug, Clone, PartialEq)]
struct AudioStreamer {
    ctx: AudioContext,
    state: AudioStreamerState
}

#[derive(Debug, Clone, PartialEq)]
enum AudioStreamerState {
    Stopped, 
    Started,
    Playing { 
        next_offset: f64,  
        scheduled: Vec<AudioBufferSourceNode>,
    }
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
            state: AudioStreamerState::Stopped
        })
    }

    fn state(&self) -> StreamerState {
        match &self.state {
            AudioStreamerState::Stopped => StreamerState::Waiting,
            AudioStreamerState::Playing { .. } => StreamerState::Playing,
            AudioStreamerState::Started => StreamerState::Playing,
        }
    }

    fn attach_to_last(&mut self, callback: Callback<(), ()>) -> Result<(), JsValue> {
        match &mut self.state {
            AudioStreamerState::Stopped => Ok(()),
            AudioStreamerState::Started => Ok(()),
            AudioStreamerState::Playing { scheduled, .. } => {
                if let Some(last) = scheduled.last() {
                    let cb: Closure<dyn FnMut() -> Result<(), JsValue>> = Closure::new(move || {
                        callback.emit(());      
                        Ok(())
                    });
                    last
                        .add_event_listener_with_callback("ended", cb.as_ref().unchecked_ref())?;
                    cb.forget();
                    Ok(())
                } else {
                    Ok(())
                }
            },
        }
    }

    fn play(&mut self) {
        if matches!(self.state, AudioStreamerState::Stopped) {
            self.state = AudioStreamerState::Started;
        }
    }

    fn play_chunk(&mut self, chunk: Vec<Vec<f32>>) -> Result<(), JsValue> {
        match &mut self.state {
            AudioStreamerState::Playing { next_offset, scheduled } => {
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
    
                buffer_source.start_with_when(*next_offset as f64).unwrap();
                *next_offset += 1_f64;
    
                scheduled.push(buffer_source);

                Ok(())
            }
            AudioStreamerState::Started =>  {
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

                let current_offset = self.ctx.current_time();
                buffer_source.start_with_when(current_offset).unwrap();
    
                self.state = AudioStreamerState::Playing { next_offset: current_offset + 1_f64, scheduled: vec![buffer_source] };
                Ok(())
            },
            AudioStreamerState::Stopped => Ok(()),
        }
    }

    fn stop(&mut self) -> Result<(), JsValue> {
        match &self.state {
            AudioStreamerState::Stopped => { },
            AudioStreamerState::Playing { scheduled, .. } => {
                for node in scheduled {
                    node.stop()?;
                }
            },
            AudioStreamerState::Started => { },
        };
        self.state = AudioStreamerState::Stopped;
        Ok(())
    }
}


enum StreamerState {
    Playing,
    Waiting
}


#[function_component(PlayButtonComponent)]
pub fn play_button() -> Html {
    let audio_streamer_state = use_state(|| StreamerState::Waiting);
    let audio_streamer = use_state(|| Rc::new(RwLock::new(AudioStreamer::empty().unwrap())));

    let instruments = use_store_value::<TrackState>();
    let worker_bridge = {
        let audio_streamer = audio_streamer.clone();
        let audio_streamer_state = audio_streamer_state.clone();
        use_bridge::<AudioStreamingWorker, _>(move |response| {
            match response {
                AudioStreamingWorkerOutput::Chunk(chunk) => {
                    audio_streamer.write().unwrap().play_chunk(chunk).unwrap();
                    audio_streamer_state.set(audio_streamer.read().unwrap().state());
                }
                AudioStreamingWorkerOutput::End => {
                    let audio_streamer_state = audio_streamer_state.clone();
                    let audio_streamer_handle = audio_streamer.clone();
                    audio_streamer.write().unwrap().attach_to_last(Callback::from(move |_| {
                        audio_streamer_handle.write().unwrap().stop().unwrap();
                        audio_streamer_state.set(audio_streamer_handle.read().unwrap().state());
                    })).unwrap();
                }
            }
        })
    };

    match *audio_streamer_state {
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
                audio_streamer.write().unwrap().play();
                worker_bridge.send(AudioStreamingWorkerInput::SendInstrument(
                    instruments.as_ref().clone().into()
                ));
            };
            html! {
                <button class={format!("outline-0 bg-transparent text-white font-semibold py-0 px-1 border border-gray-500 rounded h-7 w-7 hover:bg-gray-500 hover:border-transparent")} onclick={play}> {"⏵"} </button>
            }
        },
    }
}