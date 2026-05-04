use clap::Parser;
use std::{fs::File, path::Path};
use symphonia::core::{
    formats::{FormatOptions, probe::Hint},
    io::MediaSourceStream,
    meta::MetadataOptions,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Metadata {
    #[arg(value_name = "FILE")]
    pub file_path: String,
}

fn main() {
    let cli = Metadata::parse();

    let mut hint = Hint::new();
    let path = Path::new(&cli.file_path);

    let source = Box::new(File::open(path).unwrap());

    // Provide the file extension as a hint.
    if let Some(extension) = path.extension()
        && let Some(extension_str) = extension.to_str()
    {
        hint.with_extension(extension_str);
    }
    let mss = MediaSourceStream::new(source, Default::default());

    let format_opts = FormatOptions::default();

    let metadata_opts: MetadataOptions = Default::default();

    let mut probed = symphonia::default::get_probe()
        .probe(&hint, mss, format_opts, metadata_opts)
        .unwrap();
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
            } else if tag.raw.key == "UFID" {
                if let Some(sub_fields) = &tag.raw.sub_fields {
                    if sub_fields.iter().any(|f| f.field == "OWNER") {
                        let value_str = format!("{:?}", tag.raw.value);
                        let bytes: Vec<u8> = value_str
                            .replace("Binary([", "")
                            .replace("])", "")
                            .split(", ")
                            .map(|s| s.parse().unwrap())
                            .collect();
                        println!(
                            "MusicBrainzRecordingId: {}",
                            String::from_utf8_lossy(&bytes)
                        );
                    }
                }
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
}
