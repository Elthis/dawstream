use std::collections::HashMap;

use serde::{Serialize, Deserialize};

dawmacros::generate_keys!();

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentPayloadDto {
    pub instruments: Vec<InstrumentDto>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentDto {
    pub name: String,
    pub notes: HashMap<usize, Vec<MidiKey>>
}