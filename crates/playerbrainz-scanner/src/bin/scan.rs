use clap::Parser;
use notify_debouncer_full::new_debouncer;
use playerbrainz_scanner::{ScanItem, read_directory, watch};
use std::time::Duration;
use std::{path::Path, sync::atomic::AtomicBool};
use tokio::sync::mpsc;

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
async fn main() {
    let cli = Scan::parse();

    let (tx, mut rx) = mpsc::unbounded_channel::<ScanItem>();
    let stopping = AtomicBool::new(false);
    let scan_handle = tokio::task::spawn_blocking(move || {
        if cli.watch {
            let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();
            let mut watcher = new_debouncer(Duration::from_secs(5), None, watcher_tx).unwrap();

            watcher
                .watch(
                    &cli.dir,
                    notify_debouncer_full::notify::RecursiveMode::Recursive,
                )
                .unwrap();

            read_directory(&cli.dir, &tx);
            watch(tx, &stopping, watcher_rx);
        } else {
            read_directory(Path::new(&cli.dir), &tx)
        }
    });

    while let Some(track) = rx.recv().await {
        println!("{track:?}")
    }

    scan_handle.await.unwrap();
}
