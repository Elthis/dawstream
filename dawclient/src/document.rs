pub mod hooks {
    use yew::prelude::*;
    use wasm_bindgen::JsCast;
    
    #[hook]
    pub fn use_pointer_up_document_callback<F, R>(callback: F)
    where
        F: wasm_bindgen::closure::IntoWasmClosure<(dyn Fn(PointerEvent) -> R + 'static)> + 'static,
        R: wasm_bindgen::convert::IntoWasmAbi + 'static
    {
        use_effect(move || {
            
            let callback = wasm_bindgen::closure::Closure::new(callback);
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .set_onpointerup(Some(callback.as_ref().unchecked_ref()));

            || {
                web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .remove_event_listener_with_callback(
                        "pointerup",
                        callback.as_ref().unchecked_ref(),
                    ).unwrap();
                callback.forget();
            }
        });
    }

    #[hook]
    pub fn use_pointer_move_document_callback<F, R>(callback: F)
    where
        F: wasm_bindgen::closure::IntoWasmClosure<(dyn Fn(PointerEvent) -> R + 'static)> + 'static,
        R: wasm_bindgen::convert::IntoWasmAbi + 'static
    {
        use_effect(move || {
            let callback = wasm_bindgen::closure::Closure::new(callback);
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .set_onpointermove(Some(callback.as_ref().unchecked_ref()));

            || {
                web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .remove_event_listener_with_callback(
                        "pointermove",
                        callback.as_ref().unchecked_ref(),
                    ).unwrap();
                callback.forget();
            }
        });
    }
}