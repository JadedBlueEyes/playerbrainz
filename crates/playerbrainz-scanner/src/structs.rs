use std::{path::PathBuf, time::SystemTime};

use symphonia::core::{
    meta::{ColorMode, StandardVisualKey},
    units::{Duration, TimeBase},
};
use uuid::Uuid;

#[derive(Debug)]
pub enum ScanItem {
    /// The contents of the directory have been sent, finish
    DirComplete,
    /// Lyric file
    LyricFile(PathBuf),
    /// Art file
    ArtFile(PathBuf),
    /// Music file
    MasterRecordingFile(PathBuf, Box<MasterRecordingMetadata>), // boxing because this is significantly larger than other variants
}

type UuidVec = Vec<Uuid>;

#[derive(Default, Debug)]
pub struct MasterRecordingMetadata {
    /// MusicBrainz Recording ID - primary identifier
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
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub file_size: Option<u64>, // in bytes

    pub track_duration: Option<Duration>,
    pub track_time_base: Option<TimeBase>,
    // Audio format
    pub format_short_name: Option<&'static str>,

    // ReplayGain data
    pub track_gain_db: Option<f32>,
    pub track_peak: Option<f32>,
    pub album_gain_db: Option<f32>,
    pub album_peak: Option<f32>,

    /// Embedded media (e.g., cover art)
    pub media: Vec<ScannedMedia>,
}

#[derive(Debug, Clone)]
pub struct ScannedMedia {
    pub usage: Option<StandardVisualKey>,
    pub media_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub color_mode: Option<ColorMode>,
    pub visual_index: u32,
}
