use std::collections::HashMap;

use gloo_console::log;
use yew::prelude::*;
use yewdux::{store::Store, prelude::use_store};

use dawlib::{MidiKey, InstrumentPayloadDto, InstrumentDto};

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct InstrumentState {
    pub entries: HashMap<String, HashMap<usize, Vec<MidiKey>>>,
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

        InstrumentPayloadDto { instruments }
    }
}

impl From<InstrumentPayloadDto> for InstrumentState {
    fn from(payload: InstrumentPayloadDto) -> Self {
        let entries = payload.instruments.into_iter()
        .map(|instrument| {
            (instrument.name, instrument.notes)
        }).collect();

        InstrumentState {
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
            <SawToothInstrumentComponent/>
            <SquareInstrumentComponent/>
            <SineInstrumentComponent/>
        </div>
    }
}


#[function_component(SawToothInstrumentComponent)]
pub fn saw_tooth_instrument() -> Html {
    html! {
        <div class="flex border-t border-gray-600 text-white bg-gray-700">
            <div class="shrink p-4 w-36">
                {"Sawtooth ΛΛΛ"}
            </div>   
            <div class="grow border-l-8 border-black">
                <PianoRollComponent instrument_name={"sawtooth"}/>
            </div> 
        </div>
    }
}

#[function_component(SquareInstrumentComponent)]
pub fn square_instrument() -> Html {
    html! {
        <div class="flex border-t border-gray-600 text-white bg-gray-700">
            <div class="shrink p-4 w-36">
                {"Square ⎍"}
            </div>   
            <div class="grow border-l-8 border-black">
                <PianoRollComponent instrument_name={"square"}/>
            </div> 
        </div>
    }
}

#[function_component(SineInstrumentComponent)]
pub fn sine_instrument() -> Html {
    html! {
        <div class="flex border-t border-gray-600 text-white bg-gray-700">
            <div class="shrink p-4 w-36">
                {"Sine ∿"}
            </div>   
            <div class="grow border-l-8 border-black">
                <PianoRollComponent instrument_name={"sine"} />
            </div> 
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Properties)]
pub struct PianoRollComponentProperties {
    pub instrument_name: &'static str
}

#[function_component(PianoRollComponent)]
pub fn piano_roll(props: &PianoRollComponentProperties) -> Html {
    html! {
        <div class="grid grid-cols-1">
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::F4}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::E4}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::Eb4}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::D4}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::Db4}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::C4}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::B3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::Bb3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::A3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::Ab3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::G3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::Gb3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::F3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::E3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::Eb3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::D3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::Db3}/>
            <PianoRollKeyComponent instrument_name={props.instrument_name} midi_key={MidiKey::C3}/>
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