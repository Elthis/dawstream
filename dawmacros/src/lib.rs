extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::Ident;
use quote::quote;

// A0 = 21

fn get_key(index: usize) -> (bool, syn::Ident) {
    let (is_step_key, name) = match (index - 21) % 12 {
        0 => (false, "A"),
        1 => (true, "Bb"),
        2 => (false, "B"),
        3 => (false, "C"),
        4 => (true, "Db"),
        5 => (false, "D"),
        6 => (true, "Eb"),
        7 => (false, "E"),
        8 => (false, "F"),
        9 => (true, "Gb"),
        10 => (false, "G"),
        11 => (true, "Ab"),
        _ => panic!("Out of scope")
    };

    let octave = (index - 21) / 12;

    (is_step_key, syn::Ident::new(&format!("{name}{octave}"), Span::call_site()))
}

struct MidiKey {
    ident: Ident,
    frequency: f32,
    is_step_key: bool,
}

#[proc_macro]
pub fn generate_keys(_: TokenStream) -> TokenStream {
    let keys = (21..108).map(|key| {
        let (is_step_key, ident) = get_key(key);
        let frequency = (2.0f64.powf((key as f64- 69.0f64) / 12.0f64) * (440.0f64)) as f32;
        MidiKey {
            ident,
            frequency,
            is_step_key
        }
    }).collect::<Vec<MidiKey>>();

    let idents = keys.iter()
        .map(|key| key.ident.clone())
        .collect::<Vec<Ident>>();

    let mappings = keys.iter()
        .map(|key| {
            let MidiKey {ident, frequency, is_step_key } = key;
            quote! {
                MidiKey::#ident => #frequency,
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();
    
    let names = keys.iter()
        .map(|key| {
            let MidiKey {ident, frequency, is_step_key } = key;
            let name = ident.to_string();
            quote! {
                MidiKey::#ident => #name,
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let step_keys = keys.iter()
        .map(|key| {
            let MidiKey {ident, frequency, is_step_key } = key;
            quote! {
                MidiKey::#ident => #is_step_key,
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, serde::Serialize, serde::Deserialize)]
        pub enum MidiKey {
            #(#idents),*,
        }

        impl MidiKey {
            pub fn frequency(&self) -> f32 {
                match self {
                    #(#mappings)*
                }
            }

            pub fn name(&self) -> &str {
                match self {
                    #(#names)*
                }
            }

            pub fn is_step_key(&self) -> bool {
                match self {
                    #(#step_keys)*
                }
            }
        }
    }.into()
}