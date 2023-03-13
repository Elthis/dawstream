use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::Track;

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_INSTRUMENT_TRACK_ID: &str = "fk__instrument__track_id__track__id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Instrument::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Instrument::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Instrument::Name).string().not_null())
                    .col(ColumnDef::new(Instrument::Volume).float().not_null())
                    .col(ColumnDef::new(Instrument::Midi).json().not_null())
                    .col(ColumnDef::new(Instrument::TrackId).integer().not_null())
                    .foreign_key(
                        sea_query::ForeignKey::create()
                            .name(FK_INSTRUMENT_TRACK_ID)
                            .from(Instrument::Table, Instrument::TrackId)
                            .to(Track::Table, Track::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Instrument::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Instrument {
    Table,
    Id,
    Name,
    Volume,
    Midi,
    TrackId
}
