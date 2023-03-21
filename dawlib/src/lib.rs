use std::collections::HashMap;

use itertools::Itertools;
use reqwest::Response;
use serde::{Serialize, Deserialize};

dawmacros::generate_keys!();

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentPayloadDto {
    pub tempo: usize,
    pub instruments: Vec<InstrumentDto>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentDto {
    pub name: String,
    pub gain: f32,
    pub notes: HashMap<usize, Vec<MidiKey>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum SoundOutputPacket {
    End {
        length: u16,
        channel_data: Option<ChannelData>
    },
    Data {
        channel_data: ChannelData
    },
}

impl From<SoundOutputPacket> for Vec<u8> {
    fn from(packet: SoundOutputPacket) -> Vec<u8> {
        match packet {
            SoundOutputPacket::End { length, channel_data } => {
                let bytes = std::iter::once(0x00).chain(length.to_le_bytes());

                if let Some(channel_data) = channel_data {
                    bytes.chain(Into::<Vec<u8>>::into(channel_data)).collect()
                } else {
                    bytes.collect()
                }
            },
            SoundOutputPacket::Data {channel_data } => {
                std::iter::once(0x01).chain(Into::<Vec<u8>>::into(channel_data)).collect()    
            }
        }
    }
}

impl TryFrom<(Vec<u8>, usize)> for SoundOutputPacket {
    type Error = String;

    fn try_from((data, chunk_size): (Vec<u8>, usize)) -> Result<Self, Self::Error> {
        let mut data_iterator = data.into_iter();
        let tag_byte = data_iterator.next().ok_or_else(|| "Missing tag byte.".to_string())?;
        match tag_byte {
            0x00 => {
                let length = data_iterator.next_tuple()
                .map(|(b0, b1)| u16::from_le_bytes([b0, b1]))
                .unwrap();

                let channel_data = if length != 0 {
                    Some(ChannelData::try_from((data_iterator, length as usize))?)
                } else {
                    None
                };

                Ok(SoundOutputPacket::End { length, channel_data })
            },
            0x01 => {
                Ok(SoundOutputPacket::Data { channel_data: ChannelData::try_from((data_iterator, chunk_size))? })
            }
            _ => Err(format!("Unexpected tag byte value {tag_byte}."))
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum ChannelData {
    Mono(Vec<f32>),
    Stereo(Vec<f32>, Vec<f32>)
}

impl From<ChannelData> for Vec<u8> {
    fn from(data: ChannelData) -> Self {
        match data {
            ChannelData::Mono(channel_data) => {   
                std::iter::once(0x01).chain(channel_data.into_iter()
                .flat_map(|sample| sample.to_le_bytes()))
                .collect()
            },
            ChannelData::Stereo(first_channel, second_channel) => {
                std::iter::once(0x02)
                .chain(first_channel.into_iter().flat_map(|sample| sample.to_le_bytes()))
                .chain(second_channel.into_iter().flat_map(|sample| sample.to_le_bytes()))
                .collect()
            },
        }
    }
}

impl <T> TryFrom<(T, usize)> for ChannelData where T: IntoIterator<Item = u8> {
    type Error = String;

    fn try_from((data, chunk_size): (T, usize)) -> Result<Self, Self::Error> {
        let mut data_iterator = data.into_iter();
        let tag_byte = data_iterator.next().ok_or_else(|| "Missing tag byte.".to_string())?;
        match tag_byte {
            0x01 => {
                let channel_data = data_iterator.tuples()
                .map(|(b0, b1, b2, b3)| f32::from_le_bytes([b0, b1, b2, b3]))
                .collect();
                Ok(ChannelData::Mono(channel_data))
            }
            0x02 => {
                let channel_chunks = data_iterator.chunks(chunk_size * 4);
                let mut channel_iterator = channel_chunks.into_iter()
                .map(|channel| {
                    channel.tuples()
                    .map(|(b0, b1, b2, b3)| f32::from_le_bytes([b0, b1, b2, b3]))
                    .collect::<Vec<f32>>()
                });
                let first_data = channel_iterator.next().ok_or_else(|| "Missing First Channel Data.".to_string())?;
                let second_data = channel_iterator.next().ok_or_else(|| "Missing Second Cannel Data".to_string())?;
                Ok(ChannelData::Stereo(first_data, second_data))
            }
            _ => Err(format!("Unexpected tag byte value {tag_byte}."))
        }
    }
}

static HOST: &str = "http://localhost:3000";
static TRACKS_ENDPOINT: &str = "/tracks";

pub struct DawstreamBackendClient {
    client: reqwest::Client
}

impl Default for DawstreamBackendClient {
    fn default() -> DawstreamBackendClient {
        let client = reqwest::ClientBuilder::new().build().unwrap();
        DawstreamBackendClient {
            client
        }
    }
}

impl DawstreamBackendClient {
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



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_channel_data_mono() -> Result<(), String> {
        // given
        let channel_data = ChannelData::Mono(vec![13.0; 44100]);

        // when
        let bytes = Vec::<u8>::from(channel_data.clone());

        // then
        assert_eq!(ChannelData::try_from((bytes, 44100))?, channel_data);
        Ok(())
    }


    #[test]
    fn test_channel_data_stereo() -> Result<(), String> {
        // given
        let channel_data = ChannelData::Stereo(vec![13.0; 44100], vec![15.0; 44100]);

        // when
        let bytes = Vec::<u8>::from(channel_data.clone());

        // then
        assert_eq!(ChannelData::try_from((bytes, 44100))?, channel_data);
        Ok(())
    }
}