use std::{
    fs::File,
    path::{Path, PathBuf},
};

use clap::Parser;
use snafu::{ResultExt, Snafu};
use symphonia::core::{
    formats::{FormatOptions, probe::Hint},
    io::MediaSourceStream,
    meta::MetadataOptions,
};

use playerbrainz_scanner::read::get_musicbrainz_recording_id_from_raw_tag;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum Error {
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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Metadata {
    #[arg(value_name = "FILE")]
    pub file_path: String,
}

fn main() -> Result<()> {
    let cli = Metadata::parse();

    let mut hint = Hint::new();
    let path = Path::new(&cli.file_path);

    let source = Box::new(File::open(path).context(OpenFileSnafu {
        path: path.to_path_buf(),
    })?);

    // Provide the file extension as a hint.
    if let Some(extension) = path.extension()
        && let Some(extension_str) = extension.to_str()
    {
        hint.with_extension(extension_str);
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
                path: path.to_path_buf(),
                feature,
            }
            .fail();
        }
        Err(source) => {
            return Err(source).context(SymphoniaProbeSnafu {
                path: path.to_path_buf(),
            });
        }
    };

    let format = probed.format_info();
    println!(
        "{}: {} ({})",
        format.format, format.short_name, format.long_name
    );

    let mut metadata = probed.metadata();

    if let Some(metadata) = metadata.skip_to_latest() {
        println!("{:?}", metadata.info);

        println!();
        println!("Visuals: {}", metadata.media.visuals.len());

        metadata.media.tags.iter().for_each(|tag| {
            if let Some(std) = tag.std.as_ref() {
                println!("{std:?}")
            } else if let Some(id) = get_musicbrainz_recording_id_from_raw_tag(tag) {
                println!("MusicBrainzRecordingId: {}", id);
            } else {
                println!(
                    "{}: {}, {:?}",
                    tag.raw.key, tag.raw.value, tag.raw.sub_fields
                );
            }
        });

        metadata.per_track.iter().for_each(|track| {
            println!();
            println!("Track {}:", track.track_id);
            track.metadata.tags.iter().for_each(|tag| {
                if let Some(std) = tag.std.as_ref() {
                    println!("{std:?}")
                } else {
                    println!(
                        "{}: {}, {:?}",
                        tag.raw.key, tag.raw.value, tag.raw.sub_fields
                    );
                }
            });
            println!("Visuals: {}", track.metadata.visuals.len());
        });
    }

    Ok(())
}
