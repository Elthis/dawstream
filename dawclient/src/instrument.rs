use std::collections::HashMap;

use gloo_console::log;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlElement};
use yew::prelude::*;
use yew_hooks::prelude::*;
use yewdux::prelude::*;
use std::rc::Rc;

use dawlib::{MidiKey, InstrumentPayloadDto, InstrumentDto};

use crate::{context_panel::ContextPanelStore, document::hooks::*};

const BEAT_COUNT: usize = 300;

#[derive(Debug, Clone, PartialEq)]
pub struct MidiFragment {
    id: usize,
    length: MidiLength,
    notes: HashMap<usize, PlayedMidiKey>
}

#[derive(Debug, Clone, PartialEq)]
struct PlayedMidiKey {
    length: MidiLength,
    key: MidiKey
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MidiLength {
    beat_count: usize,
    quarter_beat_count: usize
}


#[derive(Debug, Clone, PartialEq, Properties)]
pub struct MidiFragmentContextComponentProperties {
    fragment: MidiFragment,
    name: AttrValue
}

#[function_component(MidiFragmentContextComponent)]
pub fn midi_fragment_context_component(props: &MidiFragmentContextComponentProperties) -> Html {
    

    html! {
        <div class="flex text-teal-100 h-full">
            <div class={format!("shrink w-52 pl-4")}>
                <span class="inline-block ml-1 pt-1 mr-1 select-none"> {"Length"} </span>
                <div class="inline-block ml-1 bg-transparent text-xs text-white py-0 px-1 border border-gray-500 rounded text-center"> 
                    <span class="inline-block w-5 bg-transparent text-xs"> {format!("{}.{}",props.fragment.length.beat_count, props.fragment.length.quarter_beat_count)} </span>
                </div>
            </div>
            <div class={format!("grow border-l-4 border-black overflow-y-auto h-full")}>
                    <PianoRollComponent instrument_name={props.name.clone()}/>
            </div>
        </div>
    }
}

#[function_component(ProjectComponent)]
pub fn project_component() -> Html {
    let node = use_node_ref();
    let state = use_list((1..BEAT_COUNT).into_iter().collect());
    let scroll_offset = use_state(|| 0);
    let starting_state: UseStateHandle<Option<(i32, i32)>> = use_state(|| None);

    let on_pointer_down = {
        let starting_state = starting_state.clone();
        let scroll_offset = scroll_offset.clone();
        move |event: PointerEvent| starting_state.set(Some((*scroll_offset, event.x())))
    };

    {
        let starting_state = starting_state.clone(); 
        use_pointer_up_document_callback(move |_| starting_state.set(None));
    }

    {
        let node = node.clone();
        use_pointer_move_document_callback(move |event: PointerEvent| {
            
            if let Some((starting_offset, starting_x)) = *starting_state {
                if let Some(node) = node.get() {
                    let element: &web_sys::HtmlElement = node.unchecked_ref();
                    let offset = (starting_offset - (event.x() - starting_x))
                        .min(element.scroll_width() - element.client_width())
                        .max(0);
                    scroll_offset.set(offset);

                    element.set_scroll_left(offset);
                    let html_elements = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_elements_by_class_name("instrument-timeline-scroll");
                    for index in 0..html_elements.length() {
                        if let Some(element) = html_elements.get_with_index(index) {
                            if let Some(element) = element.dyn_ref::<HtmlElement>() {
                                element.set_scroll_left(offset);
                            }
                        }
                    }
                }
            }
        });
    }

    let beat_handle = html! {
        <div onpointerdown={on_pointer_down} class="h-full">
        {
            for state.current().iter().map(|element| {
                html! { <div class="inline-block w-32 text-right h-full pr-1 py-1 border-r border-gray-600">{ element }</div> }
            })
        }
        </div>
    };

    html! {
        <div class="grid grid-cols-1 bg-gray-700 text-xs text-teal-100">
            <div class="flex font-semibold bg-gray-900 sticky top-0 absolute">
                <div class="shrink text-xs font-semibold text-teal-100"> 
                    <p class="py-1 pl-4 w-36 instrument border-gray-600 border-r"> {"Instruments"} </p>
                </div>
                <div ref={node} class="block select-none cursor-grab overflow-auto h-7 h-full whitespace-nowrap scrollbar-hide">
                    {beat_handle}
                </div>
            </div>
        </div>
    }
}



#[derive(Debug, Clone, PartialEq, Store)]
pub struct TrackState {
    pub tempo: usize,
    pub entries: HashMap<String, InstrumentData>,
}


#[derive(Debug, Default, Clone, PartialEq, Store)]
pub struct InstrumentData {
    pub gain: f32,
    pub notes: HashMap<usize, Vec<MidiKey>>
}

impl Default for TrackState {
    fn default() -> Self {
        Self { tempo: 60, entries: HashMap::new() }
    }
}

impl From<TrackState> for InstrumentPayloadDto {
    fn from(state: TrackState) -> Self {
        let instruments = state.entries.into_iter()
        .map(|(instrument, data)| {
            InstrumentDto {
                name: instrument,
                gain: data.gain,
                notes: data.notes
            }
        }).collect();

        InstrumentPayloadDto { tempo: state.tempo, instruments }
    }
}

impl From<InstrumentPayloadDto> for TrackState {
    fn from(payload: InstrumentPayloadDto) -> Self {
        let entries = payload.instruments.into_iter()
        .map(|instrument| {
            (instrument.name, InstrumentData {
                gain: instrument.gain,
                notes: instrument.notes
            })
        }).collect();

        TrackState {
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
            <InstrumentComponent name={"kick"}/>
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
pub fn instrument_component(props: &InstrumentComponentProperties) -> Html {
    let track_state = use_store_value::<TrackState>();

    let timeline = (1..BEAT_COUNT).map(|element| {
        let panel_dispatch = Dispatch::<ContextPanelStore>::new();
        let instrument = props.name;
        let on_click = panel_dispatch.reduce_mut_callback(move |store| {
            store.title = instrument.into();

            let fragment = MidiFragment {
                id: 0,
                length: MidiLength {
                    beat_count: 4,
                    quarter_beat_count: 0
                },
                notes: Default::default()
            };
            store.content = Some(Rc::new(html! {
                <MidiFragmentContextComponent fragment={fragment} name={instrument} />
            }))
        });
        if let Some(notes) = track_state.entries.get(props.name).and_then(|instrument_entry| instrument_entry.notes.get(&element)) {
            let notes = notes.iter().map(|note| format!("{} ", note.name())).collect::<String>();
            html! { 
                <div class="inline-block w-32 pr-1 text-xs border-r border-gray-600 h-full overflow-hidden hover:bg-color-gray-600" onclick={on_click}>
                    <span class="h-full"> {notes} </span>
                </div> 
            }
        } else {
            html! { <div class="inline-block w-32 pr-1 text-xs border-r border-gray-600 h-full overflow-hidden" onclick={on_click}>  </div> }
        }
    }).collect::<Html>();
    html! {
        <>
            <div class="flex content-box border-t border-gray-600 text-white bg-gray-700 h-full">
                <div class="pl-4 w-36 border-r border-gray-600 instrument">
                    <p class="text-xs"> {props.name.capitalize()} </p>
                    <GainComponent instrument_name={props.name}/>
                </div>
                <div class="grow box-border text-white border-box bg-gray-700 instrument-timeline-scroll h-full overflow-x-scroll overflow-y-hidden  whitespace-nowrap scrollbar-hide">
                    {timeline}
                </div>
            </div>
        </>
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Properties)]
pub struct VolumeComponentProperties {
    pub instrument_name: &'static str
}


#[function_component(GainComponent)]
pub fn gain_component(props: &VolumeComponentProperties) -> Html{
    let component_id = format!("{}GainSlider", props.instrument_name);
    let (track_state, track_dispatch) = use_store::<TrackState>();
    
    let on_input = {
        let instrument_name = props.instrument_name;
        track_dispatch.reduce_mut_callback_with(
            move |state, event: yew::events::InputEvent| {
                let input = event.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
    
                if let Some(input) = input {
                    state.entries.entry(instrument_name.to_string()).or_default().gain = (input.value().parse::<f32>().unwrap() - 300.0f32) / 10.0f32;
                }
            }
        )
    };

    let gain_value = track_state.entries.get(props.instrument_name).map(|data| data.gain).unwrap_or_default();
    let input_value = (gain_value * 10.0f32 + 300.0f32) as usize;
    html! {
        <div>
            <label for={component_id.clone()} class="inline-block text-neutral-200 text-xs">{"Gain"}</label>
            <div class="flex h-7">
                <input type="range" min="0" max="360" oninput={on_input} value={input_value.to_string()} class="outline-0 flex-grow h-7 accent-blue-400 transparent h-1.5 w-full cursor-pointer rounded-lg border-none" id={component_id} />
                <div class="bg-transparent ml-1 text-xs text-white py-0 px-1 rounded h-7 text-center w-[68px]">  
                    <span class="inline-block pt-1 select-none"> { gain_value } </span>
                </div>
            </div>     
        </div>
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct PianoRollComponentProperties {
    pub instrument_name: AttrValue
}

#[function_component(PianoRollComponent)]
pub fn piano_roll(props: &PianoRollComponentProperties) -> Html {
    let keys = MidiKey::VALUES.iter().rev()
        .copied()
        .map(|midi_key| {
            html! {
                <PianoRollKeyComponent instrument_name={props.instrument_name.clone()} midi_key={midi_key}/>
            }
        }).collect::<Html>();
    html! {
        <div class="grid grid-cols-1 text-xs">
            {keys}
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct PianoRollKeyComponentProperties {
    pub instrument_name: AttrValue,
    pub midi_key: MidiKey
}

#[function_component(PianoRollKeyComponent)]
pub fn piano_roll_key(props: &PianoRollKeyComponentProperties) -> Html {
    let key_class = if props.midi_key.is_step_key() {
        "text-white bg-black"
    } else {
        "text-black bg-white"
    };

  
    let midi_key = props.midi_key;
    let instrument_state = {
        let instrument_name = props.instrument_name.clone();
        use_selector::<'_, TrackState, _, _>(move |value| {
            value.entries.get(instrument_name.as_str()).cloned().unwrap_or_default()
        })
    };

    let piano_roll_entries = (0..24).map(|index| {
        let midi_key = midi_key;
        let dispatch = Dispatch::<TrackState>::new();
        let instrument_name = props.instrument_name.clone();
        let on_click = dispatch.reduce_mut_callback(move |state| {
            let instrument = state.entries.entry(instrument_name.to_string()).or_default();
            let keys = instrument.notes.entry(index).or_default();
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

        let is_set = instrument_state.notes.get(&index)
            .filter(|keys| keys.contains(&midi_key))
            .is_some();



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
