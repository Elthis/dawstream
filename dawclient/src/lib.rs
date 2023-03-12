use yew::prelude::*;

use crate::{workerconsumer::{PlayButtonComponent}, instrument::InstrumentsComponent};

pub mod workerconsumer;
pub mod worker;

mod instrument;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>      
            <TopNav/>
            <div class="flex flex-col">
                <main class="bg-gray-600 flex-grow">
                    <InstrumentsComponent/>
                </main>
            </div>   
        </>
    }
}


#[function_component(TopNav)]
pub fn top_nav() -> Html {
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

    html! {
        <nav class="flex items-center justify-between flex-wrap bg-gray-800 p-3 sticky top-0 z-10">
            <div class="flex items-center flex-shrink-0 text-white mr-6">
                <span class="text-xl tracking-tight mr-2" width="54" height="54">{ "🎹" }</span>
                <span class="font-semibold tracking-tight mr-4">{ "Dawstream" }</span>
                <PlayButtonComponent/>
            </div>
            <div class="inline-block lg:hidden">
                <button class="flex items-center px-3 py-2 border rounded text-teal-200 border-teal-400 hover:text-white hover:border-white" onclick={onclick}>
                    <svg class="fill-current h-3 w-3" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><title> { "Menu" }</title><path d="M0 3h20v2H0V3zm0 6h20v2H0V9zm0 6h20v2H0v-2z"/></svg>
                </button>
            </div>
            <div class={format!("w-full block flex-grow lg:flex lg:items-center lg:w-auto {menu_state} lg:visible")}>
                <div class="text-sm flex-col lg:flex-grow">
                    <a href="#test" class="block mt-4 lg:inline-block lg:mt-0 text-teal-100 hover:text-white mr-4">
                        {"File"}
                    </a>
                    <a href="#test" class="block mt-4 lg:inline-block lg:mt-0 text-teal-100 hover:text-white mr-4">
                        {"Edit"}
                    </a>
                    <a class="block mt-4 lg:inline-block lg:mt-0 text-gray-400 cursor-not-allowed">
                        {"Help"}
                    </a>
                </div>
            </div>
        </nav>
    }
}

