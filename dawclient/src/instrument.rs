use std::collections::HashMap;

use gloo_console::log;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::{store::Store, prelude::use_store};

use dawlib::{MidiKey, InstrumentPayloadDto, InstrumentDto};

#[derive(Debug, Clone, PartialEq, Eq, Store)]
pub struct InstrumentState {
    pub tempo: usize,
    pub entries: HashMap<String, HashMap<usize, Vec<MidiKey>>>,
}

impl Default for InstrumentState {
    fn default() -> Self {
        Self { tempo: 60, entries: HashMap::new() }
    }
}

impl From<InstrumentState> for InstrumentPayloadDto {
    fn from(state: InstrumentState) -> Self {
        let instruments = state.entries.into_iter()
        .map(|(instrument, notes)| {
            InstrumentDto {
                name: instrument,
                notes
            }
        }).collect();

        InstrumentPayloadDto { tempo: state.tempo, instruments }
    }
}

impl From<InstrumentPayloadDto> for InstrumentState {
    fn from(payload: InstrumentPayloadDto) -> Self {
        let entries = payload.instruments.into_iter()
        .map(|instrument| {
            (instrument.name, instrument.notes)
        }).collect();

        InstrumentState {
            tempo: payload.tempo,
            entries
        }
    }
}

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct NoteData {
    key: MidiKey,
    position: usize
}

#[function_component(InstrumentsComponent) ]
pub fn instruments() -> Html {
    html! {
        <div class="grid grid-cols-1">
            <InstrumentComponent name={"sawtooth"}/>
            <InstrumentComponent name={"square"}/>
            <InstrumentComponent name={"sine"}/>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Properties)]
pub struct InstrumentComponentProperties {
    name: &'static str
}

trait Capitalize {
    fn capitalize(&self) -> String;
}

impl <T: AsRef<str>> Capitalize for T {
    fn capitalize(&self) -> String {
        let mut chars = self.as_ref().chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }    
    }
}

#[function_component(InstrumentComponent)]
pub fn instrument(props: &InstrumentComponentProperties) -> Html {
    let is_piano_roll_visible = use_state(|| false);

    let piano_roll = if *is_piano_roll_visible {
        html! {
            <div class={format!("grow border-l-4 border-black")}>
                <PianoRollComponent instrument_name={props.name}/>
            </div> 
        }
    } else {
        html! {
            <div class={format!("grow")}>
            </div> 
        }
    };

    let notes_prefix = if *is_piano_roll_visible {
        "Hide"
    } else {
        "Show"
    };

    let on_visibility_click = {
        move |_| {
            is_piano_roll_visible.set(!*is_piano_roll_visible)
        }   
    };

    html! {
        <>
            <div class="flex border-t border-gray-600 text-white bg-gray-800">
                <div class="shrink p-1 pl-4 w-36">
                    <p class="text-xs"> {"Instrument"} </p>
                    <p class="text-ss"> {props.name.capitalize()} </p>
                </div>   
                <div class="flex grow border-l border-gray-600 bg-gray-800 text-teal-100 hover:text-white text-xs pt-1">
                    <div class="shrink p-1 pl-2 cursor-pointer text-center" onclick={on_visibility_click}> 
                        <p> {notes_prefix} </p>
                        <p> {"Piano Roll"} </p>
                    </div>
                    <div class="grow"></div>
                </div> 
            </div>
            <div class="flex text-white bg-gray-700">
                <div class="shrink p-4 w-36">
                    <VolumeComponent instrument_name={props.name}/>
                </div>   
                {piano_roll}
            </div>
        </>
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Properties)]
pub struct VolumeComponentProperties {
    pub instrument_name: &'static str
}


#[function_component(VolumeComponent)]
pub fn volume_component(props: &VolumeComponentProperties) -> Html{
    let component_id = format!("{}VolumeRangeSlider", props.instrument_name);
    let volume_value = use_state(|| 100.0);
    let on_change = {
        let volume_value = volume_value.clone();
        move |event: yew::events::Event| {
            let input = event.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                volume_value.set(input.value().parse().unwrap());
            }
        }
    };
    html! {
        <div>
            <label for={component_id.clone()} class="mb-2 inline-block text-neutral-200 text-sm">{format!("Volume: {}%", *volume_value)}</label>
            <input type="range" onchange={on_change} value={volume_value.to_string()} class="transparent h-1.5 w-full cursor-pointer appearance-none rounded-lg border-transparent bg-neutral-200" id={component_id} />
        </div>
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Properties)]
pub struct PianoRollComponentProperties {
    pub instrument_name: &'static str
}

#[function_component(PianoRollComponent)]
pub fn piano_roll(props: &PianoRollComponentProperties) -> Html {
    let keys = MidiKey::VALUES.iter().rev()
        .copied()
        .map(|midi_key| {
            html! {
                <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={midi_key}/>
            }
        }).collect::<Html>();
    html! {
        <div class="grid grid-cols-1">
            {keys}
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Properties)]
pub struct PianoRollKeyComponentProperties {
    pub instrument_name: &'static str,
    pub midi_key: MidiKey
}

#[function_component(PianoRollKeyComponent)]
pub fn piano_roll_key(props: &PianoRollKeyComponentProperties) -> Html {
    let key_class = if props.midi_key.is_step_key() {
        "text-white bg-black"
    } else {
        "text-black bg-white"
    };

    let (state, dispatch) = use_store::<InstrumentState>();
    let instrument_name = props.instrument_name;
    let midi_key = props.midi_key;
    let piano_roll_entries = (0..24).map(|index| {
        let midi_key = midi_key;
        let state = state.clone();
        let dispatch = dispatch.clone();
        let on_click = dispatch.reduce_mut_callback(move |state| {
            let instrument = state.entries.entry(instrument_name.to_string()).or_default();
            let keys = instrument.entry(index).or_default();
            if keys.contains(&midi_key) {
                let mut filtered_keys = keys.iter()
                    .filter(|key| **key != midi_key)
                    .copied()
                    .collect::<Vec<MidiKey>>();
                keys.clear();
                keys.append(&mut filtered_keys);   
            } else {
                keys.push(midi_key);
                log!("Hello");
            }
        });
        let is_set = state.entries.get(instrument_name).and_then(|keys| keys.get(&index)).filter(|keys| keys.contains(&midi_key)).is_some();



        html! {
            <PianoRollKeyEntryComponent is_set={is_set} onclick={on_click}/>
        }
    }).collect::<Html>();
    html! {
        <div class="flex border-t border-gray-600">
            <div class={format!("shrink px-2 w-12 {key_class}")}> {props.midi_key.name()} </div>
            <div class="grow grid grid-cols-24">
                {piano_roll_entries}
            </div>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct PianoRollKeyEntryComponentProperties {
    is_set: bool,
    onclick: Callback<MouseEvent, ()>
}

#[function_component(PianoRollKeyEntryComponent)]
pub fn piano_roll_key_entry(props: &PianoRollKeyEntryComponentProperties) -> Html {
    let background = if props.is_set {
        "bg-blue-500"
    } else {
        "bg-transparent"
    };

    html! {
        <div class={format!("{background} cursor-pointer hover:bg-gray-500 text-sm text-white font-semibold py-0 px-1 border-l border-gray-500")} onclick={props.onclick.clone()}/>
    }
}
