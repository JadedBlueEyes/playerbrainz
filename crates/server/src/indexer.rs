use playerbrainz_entities::{UuidVec, filesystem_libraries, filesystem_mastering};
use playerbrainz_scanner::{RecursiveMode, ScanItem, notify, read_directory, watch};
use sea_orm::{
    ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    TransactionTrait, sea_query,
};
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::shutdown;

pub async fn indexer_task(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::unbounded_channel::<ScanItem>();
    let libraries = filesystem_libraries::Entity::find().all(db).await?;

    let _scan_handle = tokio::task::spawn_blocking(move || {
        let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();
        let mut watcher = notify::new_debouncer(Duration::from_secs(5), None, watcher_tx).unwrap();

        for library in libraries {
            watcher
                .watch(&library.path, RecursiveMode::Recursive)
                .unwrap();
            if option_env!("SKIP_INITIAL_INDEX").is_none() {
                read_directory(&library.path, &tx);
            }
        }
        watch(tx, &shutdown::STOPPING, watcher_rx);
    })
    .await;
    indexer_reader(rx, db).await;
    Ok(())
}

async fn indexer_reader(mut rx: UnboundedReceiver<ScanItem>, db: &DatabaseConnection) {
    let mut masterings = Vec::new();
    while let Some(track) = rx.recv().await {
        match track {
            ScanItem::LyricFile(_) | ScanItem::ArtFile(_) => {} // Ignoring for now
            ScanItem::MasterRecordingFile(path, meta) => {
                masterings.push((path, meta));
            }
            ScanItem::DirComplete(path) => {
                let txn = db.begin().await.unwrap();
                let mut mastering_paths = Vec::new();
                for (path, meta) in masterings.drain(..) {
                    let file_path = path.to_str().unwrap().to_string();
                    mastering_paths.push(file_path.clone());
                    let mastering = filesystem_mastering::ActiveModel {
                        file_path: ActiveValue::Set(file_path),
                        recording_id: ActiveValue::Set(meta.recording_id),
                        release_ids: ActiveValue::Set(UuidVec(meta.release_ids.clone())),
                        release_group_ids: ActiveValue::Set(UuidVec(
                            meta.release_group_ids.clone(),
                        )),
                        track_ids: ActiveValue::Set(UuidVec(meta.track_ids.clone())),
                        artist_ids: ActiveValue::Set(UuidVec(meta.artist_ids.clone())),
                        release_artist_ids: ActiveValue::Set(UuidVec(
                            meta.release_artist_ids.clone(),
                        )),
                        work_ids: ActiveValue::Set(UuidVec(meta.work_ids.clone())),
                        title: ActiveValue::Set(meta.title),
                        artist: ActiveValue::Set(meta.artist),
                        album: ActiveValue::Set(meta.album),
                        album_artist: ActiveValue::Set(meta.album_artist),
                        original_release_date: ActiveValue::Set(meta.original_release_date),
                        track_number: ActiveValue::Set(meta.track_number),
                        genre: ActiveValue::Set(meta.genre),
                        created: ActiveValue::Set(meta.created.map(|c| c.into())),
                        modified: ActiveValue::Set(meta.modified.map(|m| m.into())),
                        file_size: ActiveValue::Set(meta.file_size),
                        track_duration: ActiveValue::Set(meta.track_duration.map(|d| d.get())),
                        track_time_base_numerator: ActiveValue::Set(
                            meta.track_time_base.map(|t| t.numer.get()),
                        ),
                        track_time_base_denominator: ActiveValue::Set(
                            meta.track_time_base.map(|t| t.denom.get()),
                        ),
                        format_short_name: ActiveValue::Set(
                            meta.format_short_name.map(|s| s.to_string()),
                        ),
                        track_gain_db: ActiveValue::Set(meta.track_gain_db),
                        track_peak: ActiveValue::Set(meta.track_peak),
                        album_gain_db: ActiveValue::Set(meta.album_gain_db),
                        album_peak: ActiveValue::Set(meta.album_peak),
                        ..Default::default()
                    };
                    filesystem_mastering::Entity::insert(mastering)
                        .on_conflict(
                            sea_query::OnConflict::column(filesystem_mastering::Column::FilePath)
                                .update_columns([
                                    filesystem_mastering::Column::RecordingId,
                                    filesystem_mastering::Column::ReleaseIds,
                                    filesystem_mastering::Column::ReleaseGroupIds,
                                    filesystem_mastering::Column::TrackIds,
                                    filesystem_mastering::Column::ArtistIds,
                                    filesystem_mastering::Column::ReleaseArtistIds,
                                    filesystem_mastering::Column::WorkIds,
                                    filesystem_mastering::Column::Title,
                                    filesystem_mastering::Column::Artist,
                                    filesystem_mastering::Column::Album,
                                    filesystem_mastering::Column::AlbumArtist,
                                    filesystem_mastering::Column::OriginalReleaseDate,
                                    filesystem_mastering::Column::TrackNumber,
                                    filesystem_mastering::Column::Genre,
                                    filesystem_mastering::Column::Created,
                                    filesystem_mastering::Column::Modified,
                                    filesystem_mastering::Column::FileSize,
                                    filesystem_mastering::Column::TrackDuration,
                                    filesystem_mastering::Column::TrackTimeBaseNumerator,
                                    filesystem_mastering::Column::TrackTimeBaseDenominator,
                                    filesystem_mastering::Column::FormatShortName,
                                    filesystem_mastering::Column::TrackGainDb,
                                    filesystem_mastering::Column::TrackPeak,
                                    filesystem_mastering::Column::AlbumGainDb,
                                    filesystem_mastering::Column::AlbumPeak,
                                ])
                                .to_owned(),
                        )
                        .exec(&txn)
                        .await
                        .unwrap();
                }

                let path_str = path.to_str().unwrap();
                let like_pattern = format!("{}/%", path_str);
                let not_like_pattern = format!("{}/%/%", path_str);

                filesystem_mastering::Entity::delete_many()
                    .filter(
                        Condition::all()
                            .add(filesystem_mastering::Column::FilePath.like(&like_pattern))
                            .add(filesystem_mastering::Column::FilePath.not_like(&not_like_pattern))
                            .add(filesystem_mastering::Column::FilePath.is_not_in(mastering_paths)),
                    )
                    .exec(&txn)
                    .await
                    .unwrap();
                txn.commit().await.unwrap();
            }
        }
    }
}
