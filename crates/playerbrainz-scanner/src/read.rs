use std::{
    fs::File,
    path::{Path, PathBuf},
};

use snafu::{ResultExt, Snafu};
use symphonia::core::{
    formats::{FormatOptions, probe::Hint},
    io::MediaSourceStream,
    meta::{MetadataOptions, RawValue, StandardTag, Tag},
};
use tracing::warn;
use uuid::Uuid;

use crate::structs::{MasterRecordingMetadata, ScannedMedia};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can occur during scanning / metadata probing.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to open file '{}': {}", path.display(), source))]
    OpenFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "Unsupported audio format/feature '{}' for file '{}'",
        feature,
        path.display()
    ))]
    SymphoniaUnsupported {
        path: PathBuf,
        feature: &'static str,
    },

    #[snafu(display(
        "Unable to probe audio metadata for file '{}': {}",
        path.display(),
        source
    ))]
    SymphoniaProbe {
        path: PathBuf,
        source: symphonia::core::errors::Error,
    },
}

/// This is a workaround for the recording ID being encoded weirdly in MP3s.
pub fn get_musicbrainz_recording_id_from_raw_tag(tag: &Tag) -> Option<Uuid> {
    if tag.raw.key == "UFID"
        && let Some(sub_fields) = &tag.raw.sub_fields
        && sub_fields.iter().any(|f| {
            if f.field == "OWNER"
                && let RawValue::String(ref uri) = f.value
                && uri.as_bytes() == b"http://musicbrainz.org"
            {
                return true;
            }
            false
        })
        && let RawValue::Binary(bytes) = &tag.raw.value
        && let Ok(id) = Uuid::try_parse_ascii(bytes.iter().as_slice())
    {
        return Some(id);
    }
    None
}

pub(crate) fn try_read_mastering(path: &Path) -> Result<MasterRecordingMetadata> {
    let path_buf = path.to_path_buf();

    let mut hint = Hint::new();

    let source = Box::new(File::open(path).context(OpenFileSnafu {
        path: path_buf.clone(),
    })?);

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
        res.file_size = Some(file_meta.len());
    }

    let mss = MediaSourceStream::new(source, Default::default());

    let format_opts = FormatOptions::default();

    let metadata_opts: MetadataOptions = Default::default();

    let probe_result =
        symphonia::default::get_probe().probe(&hint, mss, format_opts, metadata_opts);

    let mut probed = match probe_result {
        Ok(probed) => probed,
        Err(symphonia::core::errors::Error::Unsupported(feature)) => {
            return SymphoniaUnsupportedSnafu {
                path: path_buf,
                feature,
            }
            .fail();
        }
        Err(source) => {
            return Err(source).context(SymphoniaProbeSnafu { path: path_buf });
        }
    };

    res.format_short_name = Some(probed.format_info().short_name);

    let mut metadata = probed.metadata();

    if let Some(metadata) = metadata.skip_to_latest() {
        for tag in &metadata.media.tags {
            let Some(std) = tag.std.as_ref() else {
                if let Some(id) = get_musicbrainz_recording_id_from_raw_tag(tag) {
                    res.recording_id = Some(id);
                }
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
                StandardTag::MusicBrainzRecordingId(id) => {
                    res.recording_id = Uuid::parse_str(id)
                        .inspect_err(|e| warn!(%e, %id, "Parse error"))
                        .ok()
                } // Haven't seen this set
                StandardTag::MusicBrainzTrackId(id) => {
                    res.recording_id = Uuid::parse_str(id)
                        .inspect_err(|e| warn!(%e, %id, "Parse error"))
                        .ok()
                } // Actually the recording ID per picard? Symphonia bug? (in flac files)
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

    if let Some(track) = probed.default_track(symphonia::core::formats::TrackType::Audio) {
        if let Some(codec) = &track.codec_params {
            let _audio_codec = codec.audio().expect("audio track");
            // Extract bitrate, sample frequency, bit depth
        }
        res.track_duration = track.duration;
        res.track_time_base = track.time_base;
    }

    Ok(res)
}

fn parse_gain_value(s: &str) -> Result<f32, std::num::ParseFloatError> {
    let trimmed = s.trim().trim_end_matches("dB").trim();
    trimmed.parse()
}
