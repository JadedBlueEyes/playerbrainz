use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::Duration,
};

use clap::Parser;
use notify_debouncer_full::{new_debouncer, notify};
use snafu::{ResultExt, Snafu};
use tokio::sync::mpsc;

use playerbrainz_scanner::{RecursiveMode, ScanItem, read_directory, watch};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Unable to create filesystem debouncer: {}", source))]
    CreateDebouncer { source: notify::Error },

    #[snafu(display("Unable to watch directory '{}': {}", dir.display(), source))]
    WatchDirectory { dir: PathBuf, source: notify::Error },

    #[snafu(display("Background scan task failed: {}", source))]
    JoinScanTask { source: tokio::task::JoinError },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Scan {
    #[arg(value_name = "DIR")]
    pub dir: String,
    #[arg(short, long)]
    pub watch: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Scan::parse();

    let (tx, mut rx) = mpsc::unbounded_channel::<ScanItem>();

    let dir = cli.dir;
    let watch_enabled = cli.watch;

    let scan_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        let stopping = AtomicBool::new(false);

        if watch_enabled {
            let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();

            let mut watcher = new_debouncer(Duration::from_secs(5), None, watcher_tx)
                .context(CreateDebouncerSnafu)?;

            watcher
                .watch(&dir, RecursiveMode::Recursive)
                .context(WatchDirectorySnafu {
                    dir: PathBuf::from(&dir),
                })?;

            read_directory(&dir, &tx);
            watch(tx, &stopping, watcher_rx);
        } else {
            read_directory(Path::new(&dir), &tx)
        }

        Ok(())
    });

    while let Some(track) = rx.recv().await {
        println!("{track:?}")
    }

    scan_handle.await.context(JoinScanTaskSnafu)??;

    Ok(())
}
