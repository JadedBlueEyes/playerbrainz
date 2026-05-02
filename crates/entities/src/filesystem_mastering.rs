use std::ops::Deref;

use crate::UuidVec;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "filesystem_mastering")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Absolute path
    #[sea_orm(unique, indexed)]
    pub file_path: String,

    /// MusicBrainz Recording ID - primary identifier
    #[sea_orm(indexed)]
    pub recording_id: Option<Uuid>,

    // Additional MusicBrainz IDs
    pub release_ids: UuidVec,
    pub release_group_ids: UuidVec,
    pub track_ids: UuidVec,
    pub artist_ids: UuidVec,
    pub release_artist_ids: UuidVec,
    pub work_ids: UuidVec,

    // Fallback track metadata from audio tags
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub original_release_date: Option<String>,
    pub track_number: Option<u64>,
    pub genre: Option<String>,

    // Filesystem data
    pub created: Option<DateTimeUtc>,
    pub modified: Option<DateTimeUtc>,
    pub file_size: Option<u64>, // in bytes

    pub track_duration: Option<u64>,
    pub track_time_base_numerator: Option<u32>,
    pub track_time_base_denominator: Option<u32>,
    // Audio format
    pub format_short_name: Option<String>,

    // ReplayGain data
    pub track_gain_db: Option<f32>,
    pub track_peak: Option<f32>,
    pub album_gain_db: Option<f32>,
    pub album_peak: Option<f32>,
}

impl ActiveModelBehavior for ActiveModel {}
