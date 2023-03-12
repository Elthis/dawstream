use axum::{http::StatusCode, Json, extract::State};
use dawlib::InstrumentPayloadDto;

use crate::{AppState, dal::track::TrackRepository};


pub async fn store(State(state): State<AppState>, Json(payload): Json<InstrumentPayloadDto>) -> (StatusCode, Json<InstrumentPayloadDto>) {
    TrackRepository::save(&state.database_connection, "default", &payload).await.unwrap();
    (StatusCode::OK, Json(payload))
}

pub async fn restore(State(state): State<AppState>) -> (StatusCode, Json<InstrumentPayloadDto>) {
    let model = TrackRepository::find_by_name(&state.database_connection, "default").await.unwrap();

    match model.map(|model| serde_json::from_value(model.data)) {
        Some(Ok(model)) => {
            (StatusCode::OK, Json(model))
        },
        _ => {
            (StatusCode::NOT_FOUND, Json(InstrumentPayloadDto { instruments: vec![] }))
        },
    } 
}