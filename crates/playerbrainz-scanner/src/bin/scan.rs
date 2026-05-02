use clap::Parser;
use playerbrainz_scanner::{ScanItem, read_directory, watch_directory};
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
            watch_directory(Path::new(&cli.dir), tx, stopping);
        } else {
            read_directory(Path::new(&cli.dir), &tx)
        }
    });

    while let Some(track) = rx.recv().await {
        println!("{track:?}")
    }

    scan_handle.await.unwrap();
}
