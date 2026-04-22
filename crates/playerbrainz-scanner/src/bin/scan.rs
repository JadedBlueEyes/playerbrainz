use clap::Parser;
use playerbrainz_scanner::{ScanItem, read_directory};
use std::path::Path;
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Scan {
    #[arg(value_name = "DIR")]
    pub dir: String,
}

#[tokio::main]
async fn main() {
    let cli = Scan::parse();

    let (tx, mut rx) = mpsc::unbounded_channel::<ScanItem>();
    let scan_handle = tokio::task::spawn_blocking(move || read_directory(Path::new(&cli.dir), tx));

    while let Some(track) = rx.recv().await {
        println!("{track:?}")
    }

    scan_handle.await.unwrap();
}
