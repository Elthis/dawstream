use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::instrument::InstrumentState;

#[function_component(TempoComponent)]
pub fn tempo_component() -> Html {
    let (track_state, track_dispatcher) = use_store::<InstrumentState>();

    let on_input = |event: InputEvent| {
        if let Some(input) = event.target().as_ref().and_then(|target| target.dyn_ref::<HtmlInputElement>()) {
            if let Ok(value) = input.value().parse::<f32>() {
                if value > 220.0 {
                    input.set_value(&220.to_string()); 
                } else if value < 0.0 {
                    input.set_value(&20.to_string()); 
                } else {
                    input.set_value(&(value as usize).to_string());
                }
            } else {
                let value = input.value().chars().filter(|char| *char != '-')
                .collect::<String>();

            
                input.set_value(&value);
            }
        }
    };

    let on_change = track_dispatcher.reduce_mut_callback_with(move |state, event: Event| {
        if let Some(input) = event.target().as_ref().and_then(|target| target.dyn_ref::<HtmlInputElement>()) {
            if let Ok(value) = input.value().parse::<usize>() {
                if value >= 20 {
                    state.tempo = value;
                } else {
                    state.tempo = 20;
                }
            } else {
                state.tempo = 20;
            }
        } 
    });

    html! {
        <div class="block bg-transparent text-xs text-white py-0 px-1 border border-gray-500 rounded h-7 text-center">  
            <input type="number" class="ml-1 outline-0 inline-block mt-1 w-5 bg-transparent text-xs" oninput={on_input} onchange={on_change} value={track_state.tempo.to_string()} />
            <span class="inline-block ml-1 pt-1 mr-1 select-none"> {"BPM"} </span>
        </div>
    }
}