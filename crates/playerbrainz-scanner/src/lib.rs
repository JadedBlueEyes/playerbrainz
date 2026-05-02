use std::{
    collections::HashSet, ffi::OsStr, os::unix::ffi::OsStrExt, path::Path,
    sync::atomic::AtomicBool, time::Duration,
};

use notify_debouncer_full::new_debouncer;
use tokio::sync::mpsc;
use tracing::{error, info};
use walkdir::{DirEntry, WalkDir};

use crate::read::{ScanError, try_read_mastering};
pub use crate::structs::ScanItem;

pub const SCANNER_VERSION: &str = "0";

mod read;
mod structs;

pub fn watch_directory(
    dir: impl AsRef<Path>,
    tx: mpsc::UnboundedSender<ScanItem>,
    stopping: AtomicBool,
) {
    let dir = dir.as_ref();

    let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();
    let mut watcher = new_debouncer(Duration::from_secs(5), None, watcher_tx).unwrap();
    watcher
        .watch(dir, notify_debouncer_full::notify::RecursiveMode::Recursive)
        .unwrap();

    read_directory(dir, &tx);

    loop {
        match watcher_rx.recv_timeout(Duration::from_secs(15)) {
            Ok(Ok(batch)) => {
                let mut scan_roots: HashSet<&Path> = HashSet::new();

                for evt in &batch {
                    for p in &evt.paths {
                        if p.is_file() && p.file_name().is_some_and(|p| !is_ignored_file(p)) {
                            scan_roots.insert(p.parent().unwrap_or(dir));
                        } else {
                            scan_roots.insert(p);
                        }
                    }
                }

                let scan_roots = minimize_folders(scan_roots);

                for d in scan_roots {
                    info!(dir = %d.display(), "Rescanning directory due to filesystem changes");
                    read_directory(d, &tx);
                }
            }
            Ok(Err(e)) => error!(?e, "Watcher error"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(e) => error!(%e, "Watcher channel error"),
        }

        if stopping.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
    }
}

/// Coalesce child paths into their parent paths
fn minimize_folders<'a>(roots: HashSet<&'a Path>) -> Vec<&'a Path> {
    let mut v: Vec<&'a Path> = roots.into_iter().collect();
    // Sort so parents come before children (lexicographical ordering for paths).
    v.sort();

    let mut minimized: Vec<&'a Path> = Vec::new();
    'outer: for p in v {
        for kept in &minimized {
            if p.starts_with(kept) {
                continue 'outer;
            }
        }
        minimized.push(p);
    }
    minimized
}

pub fn read_directory(dir: impl AsRef<Path>, tx: &mpsc::UnboundedSender<ScanItem>) {
    let dir = dir.as_ref();
    if !dir.exists() {
        tx.send(ScanItem::DirComplete(dir.into()))
            .expect("Channel to send");
        return;
    }

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
            tx.send(ScanItem::DirComplete(entry.into_path()))
                .expect("Channel to send");
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

fn is_ignored_file(name: &OsStr) -> bool {
    #[cfg(target_os = "macos")]
    if name.as_bytes() == b".DS_Store" {
        return true;
    };

    false
}
