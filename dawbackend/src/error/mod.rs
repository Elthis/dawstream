use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse};
use axum_macros::FromRequest;
use sea_orm::DbErr;
use serde_json::json;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct JsonInput<T>(pub T);

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let payload = json!({
            "message": self.message
        });

        (self.status, axum::Json(payload)).into_response()
    }
}

impl From<DbErr> for ApiError {
    fn from(error: DbErr) -> Self {
        tracing::error!("Database Error: {}.", error);
        ApiError { 
            status: StatusCode::INTERNAL_SERVER_ERROR, 
            message: "Unexpected error occured.".to_string() 
        }
    }
}