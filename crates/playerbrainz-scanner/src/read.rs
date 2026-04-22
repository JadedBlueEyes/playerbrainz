use std::{fs::File, path::Path};
use symphonia::core::{
    formats::{FormatOptions, probe::Hint},
    io::MediaSourceStream,
    meta::{MetadataOptions, StandardTag},
};
use thiserror::Error;
use tracing::warn;
use uuid::Uuid;

use crate::structs::{MasterRecordingMetadata, ScannedMedia};

/// Errors that can occur during scanning
#[derive(Debug, Error)]
pub enum ScanError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported feature: {0}")]
    SymphoniaUnsupported(&'static str),
}

impl From<symphonia::core::errors::Error> for ScanError {
    fn from(value: symphonia::core::errors::Error) -> Self {
        match value {
            symphonia::core::errors::Error::IoError(error) => error.into(),
            symphonia::core::errors::Error::DecodeError(_) => {
                unreachable!("Should not be decoding in scanner")
            }
            symphonia::core::errors::Error::SeekError(_) => {
                unreachable!("Should not be seeking in scanner")
            }
            symphonia::core::errors::Error::Unsupported(f) => Self::SymphoniaUnsupported(f),
            symphonia::core::errors::Error::LimitError(_) => todo!(),
            symphonia::core::errors::Error::ResetRequired => unreachable!(),
            _ => todo!(),
        }
    }
}

pub(crate) fn try_read_mastering(path: &Path) -> Result<MasterRecordingMetadata, ScanError> {
    let mut hint = Hint::new();

    let source = Box::new(File::open(path)?);

    // Provide the file extension as a hint.
    if let Some(extension) = path.extension()
        && let Some(extension_str) = extension.to_str()
    {
        hint.with_extension(extension_str);
    }

    let mut res = MasterRecordingMetadata::default();

    if let Ok(file_meta) = source.metadata() {
        res.modified = file_meta.modified().ok();
        res.created = file_meta.created().ok();
    }

    let mss = MediaSourceStream::new(source, Default::default());

    let format_opts = FormatOptions::default();

    let metadata_opts: MetadataOptions = Default::default();

    let mut probed =
        symphonia::default::get_probe().probe(&hint, mss, format_opts, metadata_opts)?;

    let mut metadata = probed.metadata();

    if let Some(metadata) = metadata.skip_to_latest() {
        for tag in &metadata.media.tags {
            let Some(std) = tag.std.as_ref() else {
                continue;
            };

            match std {
                // See https://picard-docs.musicbrainz.org/en/latest/appendices/tag_mapping.html and Symphonia source
                StandardTag::MusicBrainzArtistId(id) => {
                    if let Ok(id) =
                        Uuid::parse_str(id).inspect_err(|e| warn!(%e, %id, "Parse error"))
                    {
                        res.artist_ids.push(id)
                    }
                } // As expected
                StandardTag::MusicBrainzAlbumArtistId(id) => {
                    if let Ok(id) =
                        Uuid::parse_str(id).inspect_err(|e| warn!(%e, %id, "Parse error"))
                    {
                        res.release_artist_ids.push(id)
                    }
                } // As expected (release)
                StandardTag::MusicBrainzAlbumId(id) => {
                    if let Ok(id) =
                        Uuid::parse_str(id).inspect_err(|e| warn!(%e, %id, "Parse error"))
                    {
                        res.release_ids.push(id)
                    }
                } // As expected (release)

                StandardTag::MusicBrainzReleaseGroupId(id) => {
                    if let Ok(id) =
                        Uuid::parse_str(id).inspect_err(|e| warn!(%e, %id, "Parse error"))
                    {
                        res.release_group_ids.push(id)
                    }
                } // As expected
                // StandardTag::MusicBrainzRecordingId(_) => todo!(), // Haven't seen this set
                StandardTag::MusicBrainzTrackId(id) => {
                    res.recording_id = Uuid::parse_str(id)
                        .inspect_err(|e| warn!(%e, %id, "Parse error"))
                        .ok()
                } // Actually the recording ID per picard? Symphonia bug?
                StandardTag::MusicBrainzReleaseTrackId(id) => {
                    if let Ok(id) =
                        Uuid::parse_str(id).inspect_err(|e| warn!(%e, %id, "Parse error"))
                    {
                        res.track_ids.push(id)
                    }
                } // Actual ID for MB track
                StandardTag::MusicBrainzWorkId(id) => {
                    if let Ok(id) =
                        Uuid::parse_str(id).inspect_err(|e| warn!(%e, %id, "Parse error"))
                    {
                        res.work_ids.push(id)
                    }
                } // As expected

                // StandardTag::MusicBrainzDiscId(_) => todo!(), // Haven't seen this set
                // StandardTag::MusicBrainzLabelId(_) => todo!(), // Haven't seen this set
                // StandardTag::MusicBrainzOriginalAlbumId(_) => todo!(),
                // StandardTag::MusicBrainzOriginalArtistId(_) => todo!(),

                // MusicBrainzTrmId is obsolete (https://musicbrainz.org/doc/Fingerprinting#TRM)
                // MusicBrainzGenreId doesn't exist?
                // MusicBrainzReleaseStatus, MusicBrainzReleaseType should be pulled from source

                // StandardTag::AcoustIdFingerprint(_) => todo!(),
                // StandardTag::AcoustIdId(_) => todo!(),
                StandardTag::ReplayGainAlbumGain(n) => {
                    res.album_gain_db = parse_gain_value(n)
                        .inspect_err(|e| warn!(%e, %n, "Parse error"))
                        .ok()
                }
                StandardTag::ReplayGainAlbumPeak(n) => {
                    res.album_peak = n
                        .parse::<f32>()
                        .inspect_err(|e| warn!(%e, %n, "Parse error"))
                        .ok()
                }
                StandardTag::ReplayGainTrackGain(n) => {
                    res.track_gain_db = parse_gain_value(n)
                        .inspect_err(|e| warn!(%e, %n, "Parse error"))
                        .ok()
                }
                StandardTag::ReplayGainTrackPeak(n) => {
                    res.track_peak = n
                        .parse::<f32>()
                        .inspect_err(|e| warn!(%e, %n, "Parse error"))
                        .ok()
                }
                // StandardTag::ReplayGainTrackRange(_) => todo!(),
                // StandardTag::ReplayGainAlbumRange(_) => todo!(),
                // StandardTag::ReplayGainReferenceLoudness(_) => todo!(),

                // StandardTag::Lyrics(_) => todo!(),

                // StandardTag::Comment(_) => todo!(),
                StandardTag::TrackTitle(s) => res.title = Some(s.to_string()),
                StandardTag::Artist(s) => res.artist = Some(s.to_string()),
                StandardTag::Album(s) => res.album = Some(s.to_string()),
                StandardTag::AlbumArtist(s) => res.album_artist = Some(s.to_string()),
                StandardTag::OriginalReleaseDate(s) => {
                    res.original_release_date = Some(s.to_string())
                }
                StandardTag::TrackNumber(s) => res.track_number = Some(*s),
                StandardTag::Genre(s) => res.genre = Some(s.to_string()),

                _ => continue,
            }
        }

        for (idx, visual) in metadata.media.visuals.iter().enumerate() {
            res.media.push(ScannedMedia {
                usage: visual.usage,
                media_type: visual.media_type.clone(),
                width: visual.dimensions.map(|d| d.width),
                height: visual.dimensions.map(|d| d.height),
                color_mode: visual.color_mode,
                visual_index: idx as u32,
            });
        }

        // res.duration_secs = duration_secs;
    } else {
        // Concerning but legitimate AFAIK
    };

    Ok(res)
}

fn parse_gain_value(s: &str) -> Result<f32, std::num::ParseFloatError> {
    let trimmed = s.trim().trim_end_matches("dB").trim();
    trimmed.parse()
}
