use std::collections::HashMap;

use reqwest::Response;
use serde::{Serialize, Deserialize};

dawmacros::generate_keys!();

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentPayloadDto {
    pub instruments: Vec<InstrumentDto>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentDto {
    pub name: String,
    pub notes: HashMap<usize, Vec<MidiKey>>
}


static HOST: &str = "http://localhost:3000";
static TRACKS_ENDPOINT: &str = "/tracks";

pub struct DawstreamBackendClient {
    client: reqwest::Client
}

impl DawstreamBackendClient {
    pub fn new() -> DawstreamBackendClient {
        let client = reqwest::ClientBuilder::new().build().unwrap();
        DawstreamBackendClient {
            client
        }
    }

    pub async fn store_state(&self, state: &InstrumentPayloadDto) -> Result<(), DawstreamBackendClientError> {
        let request = self.client.post(format!("{HOST}{TRACKS_ENDPOINT}"))
        .header("content-type", "application/json")
        .body(serde_json::to_string(&state).unwrap())
        .build()?;

        self.client.execute(request).await.and_then(Response::error_for_status)?;

        Ok(())
    }

    pub async fn restore_state(&self) -> Result<InstrumentPayloadDto, DawstreamBackendClientError> {
        let request = self.client.get(format!("{HOST}{TRACKS_ENDPOINT}"))
        .header("accept", "application/json")
        .build()?;

        let response = self.client.execute(request).await.and_then(Response::error_for_status)?;

        let body = response.bytes().await?;

        Ok(serde_json::from_slice(&body)?)
    }
}

#[derive(Debug)]
pub enum DawstreamBackendClientError {
    Serde(serde_json::Error),
    Reqwest(reqwest::Error)
}

impl From<serde_json::Error> for DawstreamBackendClientError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error)
    }
}

impl From<reqwest::Error> for DawstreamBackendClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

