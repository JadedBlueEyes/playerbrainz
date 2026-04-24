use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};

use tokio::sync::mpsc;
use tracing::error;
use walkdir::{DirEntry, WalkDir};

use crate::read::{ScanError, try_read_mastering};
pub use crate::structs::ScanItem;

pub const SCANNER_VERSION: &str = "0";

mod read;
mod structs;

pub fn read_directory(dir: impl AsRef<Path>, tx: mpsc::UnboundedSender<ScanItem>) {
    // Walk the tree depth first. This means we can push up completion notifications,
    // that can be used to remove deleted files and clean up unused references
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .contents_first(true)
        .sort_by(dir_sort)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            tx.send(ScanItem::DirComplete).expect("Channel to send");
            continue;
        }

        if is_lyric_file(entry.file_name()) {
            tx.send(ScanItem::LyricFile(entry.into_path()))
                .expect("Channel to send");
            continue;
        }

        if is_cover_file(entry.file_name()) {
            tx.send(ScanItem::ArtFile(entry.into_path()))
                .expect("Channel to send");
            continue;
        }

        // TODO: parallelize reads? Perhaps dump it as the outer process's responsibility so it can read a directory at a time without having to do a threadpool inside spawn_blocking.
        match try_read_mastering(entry.path()) {
            Ok(r) => {
                tx.send(ScanItem::MasterRecordingFile(
                    entry.into_path(),
                    Box::new(r),
                ))
                .expect("Channel to send");
                continue;
            }
            Err(e) => {
                if let ScanError::SymphoniaUnsupported(_f) = e {
                    // Ignore this, expected
                    continue;
                }
                error!(%e, "Unexpected error while scanning");
                continue;
            }
        }
    }
}

fn dir_sort(a: &DirEntry, b: &DirEntry) -> core::cmp::Ordering {
    // We have to look at subdirectories at the same time (ie without the contents interleaved)
    match (a.file_type().is_dir(), b.file_type().is_dir()) {
        (true, true) => return a.file_name().cmp(b.file_name()),
        (true, false) => return core::cmp::Ordering::Less,
        (false, true) => return core::cmp::Ordering::Greater,
        (false, false) => (),
    };

    a.file_name().cmp(b.file_name())
}

fn is_lyric_file(name: &OsStr) -> bool {
    let lyric_needles = [
        OsStr::new(".lrc"),
        OsStr::new(".txt"), // elrc, ttml?
    ];
    for needle in lyric_needles {
        if name.as_bytes().ends_with(needle.as_bytes()) {
            return true;
        }
    }
    false
}

fn is_cover_file(name: &OsStr) -> bool {
    let needle = OsStr::new("cover");
    name.as_bytes().starts_with(needle.as_bytes())
}
