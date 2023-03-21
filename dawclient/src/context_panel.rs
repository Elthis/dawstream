use std::rc::Rc;

use web_sys::Element;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::document::hooks::{use_pointer_up_document_callback, use_pointer_move_document_callback};

#[derive(Debug, Clone, PartialEq, Store, Default)]
pub struct ContextPanelStore {
    pub title: AttrValue,
    pub content: Option<Rc<Html>>
}

#[function_component(ContextPanel)]
pub fn context_panel() -> Html {
    let state = use_store_value::<ContextPanelStore>();
    let resizing = use_state(|| None);
    let node_ref = use_node_ref();
    let on_pointer_down = {
        let node_ref = node_ref.clone();
        let resizing = resizing.clone();
        move |event: PointerEvent| {
            if let Some(node) = node_ref.cast::<Element>() {
                resizing.set(Some((event.client_y(), node.client_height())))
            } 
        }
    };

    {
        let resizing = resizing.clone();
        use_pointer_up_document_callback(move |_| {
            resizing.set(None);
        });
    }

    {
        let node_ref = node_ref.clone();
        use_pointer_move_document_callback(move |event: PointerEvent| {
            if resizing.is_none() {
                return;
            }

            let (start_pos, start_height) = resizing.unwrap();

            if let Some(node) = node_ref.cast::<Element>() {
                let height = start_pos + (start_height - event.client_y());
                node.set_attribute("style", &format!("height: {height}px;")).unwrap();
            }
        });
    }

    let on_close_click = Dispatch::<ContextPanelStore>::new().reduce_mut_callback(|state| {
        state.content = None;
    });
    

    if let Some(content) = &state.content {
        html! {
            <div ref={node_ref} class={format!("relative h-64 block sticky bottom-0 z-10 bg-gray-700 border-t border-gray-600 w-full")}>
                <div class="flex flex-col h-full">
                    <div class="flex h-min py-1 pl-4 bg-gray-800 text-teal-100 text-sm sticky top-0">
                        <div class="grow cursor-n-resize" onpointerdown={on_pointer_down}> { state.title.as_ref() } </div>
                        <div class="mr-2 cursor-pointer hover:text-white font-semibold text-md" onclick={on_close_click}> {"×"} </div>
                    </div>
                    <div class="grow overflow-auto">
                        {content.as_ref().clone()}
                    </div>
                </div>
            </div>
        }
    } else {
        html! {}
    }
}