use dawlib::InstrumentPayloadDto;
use sea_orm::*;

use super::entity::{track, track::Entity as Track};

pub struct TrackRepository;

impl TrackRepository {
    pub async fn save(db: &DbConn, name: &str, data: &InstrumentPayloadDto) -> Result<(), DbErr>{
        let track = Self::find_by_name(db, name).await?;

        match track {
            Some(track) => {
                let mut active_track = track.into_active_model();
                active_track.data = Set(serde_json::to_value(data).unwrap());
                active_track.update(db).await?;
            }
            None => {
                let active_track = track::ActiveModel {
                    name: Set(name.to_owned()),
                    data: Set(serde_json::to_value(data).unwrap()),
                    ..Default::default()
                };
                active_track.insert(db).await?;
            }
        }
        Ok(())
    }
    pub async fn find_by_name(db: &DbConn, name: &str) -> Result<Option<track::Model>, DbErr>{
        Track::find()
            .filter(track::Column::Name.eq(name))
            .one(db)
            .await
    }
}