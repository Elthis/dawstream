use dawlib::DawstreamBackendClient;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::{play::PlayButtonComponent, instrument::{InstrumentsComponent, TrackState, ProjectComponent}, tempo::TempoComponent, context_panel::ContextPanel};

pub mod play;
pub mod worker;
pub mod tempo;
pub mod context_panel;
pub mod document;

mod instrument;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>      
            <TopNav/>
            <div class="flex flex-col grow">
                <main class="bg-gray-600 grow">
                    <InstrumentsComponent/>
                </main>
            </div>
            <ContextPanel/>
        </>
    }
}


#[function_component(TopNav)]
pub fn top_nav() -> Html {
    let (instrument_state, instrument_state_dispatch) = use_store::<TrackState>();
    let menu_toggle = use_state(|| false);

    let onclick = {
        let menu_toggle = menu_toggle.clone();
        Callback::from(move |_| menu_toggle.set(!*menu_toggle))
    };

    let menu_state = if *menu_toggle {
        ""
    } else {
        "hidden"
    };

    let on_store = move |_|  {
        let instrument_state = instrument_state.clone();
        spawn_local(async move {
            let client =  DawstreamBackendClient::default();
            
            client.store_state(&instrument_state.as_ref().clone().into()).await.unwrap();
        });
    };

    let on_restore = move |_| { 
        let instrument_state_dispatch = instrument_state_dispatch.clone();
        spawn_local(async move {
            let client =  DawstreamBackendClient::default();
            let restored_state = client.restore_state().await.unwrap();
            instrument_state_dispatch.set(restored_state.into());
        })
    };


    html! {
        <nav class="flex items-center justify-between flex-wrap bg-gray-900 sticky top-0 z-10">
            <div class="flex items-center flex-shrink-0 text-white mr-3 p-3">
                <div class="w-36">
                    <span class="text-xl tracking-tight mr-2" width="54" height="54">{ "🎹" }</span>
                    <span class="font-semibold tracking-tight">{ "Dawstream" }</span>
                </div>
                <PlayButtonComponent/>
                <span class="mx-1"/>
                <TempoComponent/>
            </div>
            <div class="inline-block lg:hidden py-3">
                <button class="flex items-center px-3 py-2 border rounded text-teal-200 border-teal-400 hover:text-white hover:border-white py-3" onclick={onclick}>
                    <svg class="fill-current h-3 w-3" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><title> { "Menu" }</title><path d="M0 3h20v2H0V3zm0 6h20v2H0V9zm0 6h20v2H0v-2z"/></svg>
                </button>
            </div>
            <div class={format!("w-full block flex-grow lg:flex lg:items-center lg:w-auto {menu_state} lg:visible py-3")}>
                <div class="text-sm flex-col lg:flex-grow">
                    <a onclick={on_store} class="block mt-4 lg:inline-block lg:mt-0 text-teal-100 hover:text-white mr-4 cursor-pointer">
                        {"Store"}
                    </a>
                    <a onclick={on_restore} class="block mt-4 lg:inline-block lg:mt-0 text-teal-100 hover:text-white mr-4 cursor-pointer">
                        {"Restore"}
                    </a>
                    <a class="block mt-4 lg:inline-block lg:mt-0 text-gray-400 cursor-not-allowed">
                        {"Help"}
                    </a>
                </div>
            </div>
            <ProjectComponent/>
        </nav>
    }
}

