use axum::Json;
use axum::{http::StatusCode, extract::State};
use dawlib::InstrumentPayloadDto;
use crate::error::{JsonInput, ApiError};

use crate::{AppState, dal::track::TrackRepository};


pub async fn store(State(state): State<AppState>, JsonInput(payload): JsonInput<InstrumentPayloadDto>) -> Result<(StatusCode, Json<InstrumentPayloadDto>), ApiError> {
    TrackRepository::save(&state.database_connection, "default", &payload).await.unwrap();
    Ok((StatusCode::OK, Json(payload)))
}

pub async fn restore(State(state): State<AppState>) -> Result<(StatusCode, Json<InstrumentPayloadDto>), ApiError> {
    let model = TrackRepository::find_by_name(&state.database_connection, "default").await.unwrap();

    match model.map(|model| serde_json::from_value(model.data)) {
        Some(Ok(model)) => {
            Ok((StatusCode::OK, Json(model)))
        },
        _ => {
            Err(ApiError {
                status: StatusCode::NOT_FOUND,
                message: "Track not found.".to_string(),
            })
        },
    } 
}