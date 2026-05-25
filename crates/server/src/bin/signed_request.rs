use axum::http;
use httpsig_hyper::ContentDigest;
use sea_orm::Database;
use snafu::{ResultExt, Snafu};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use playerbrainz_server::{
    config::{ConfigError, config},
    remote_server_sync::{RemoteServerSyncError, get_graphql_endpoint},
    startup::StartupError,
};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[allow(dead_code)]
enum Error {
    #[snafu(display("Unable to connect to database at '{}': {}", database_url, source))]
    DatabaseConnect {
        database_url: String,
        source: sea_orm::DbErr,
    },

    #[snafu(display("Unable to sync database schema: {}", source))]
    SyncDatabaseSchema { source: sea_orm::DbErr },

    #[snafu(display("Unable to bind server listener on '{}': {}", addr, source))]
    BindTcpListener {
        addr: String,
        source: std::io::Error,
    },

    #[snafu(display("Bad configuration: {}", source))]
    Config { source: ConfigError },
    #[snafu(display("Failed to start up: {}", source))]
    StartUp { source: StartupError },
    #[snafu(display("Failed to communicate with remote server: {}", source))]
    RemoteServer { source: RemoteServerSyncError },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = dbg!(config().context(crate::ConfigSnafu)?);

    let database_url = config.database_url.clone();

    let db = Database::connect(database_url.clone())
        .await
        .context(DatabaseConnectSnafu {
            database_url: database_url.clone(),
        })?;

    let key = playerbrainz_server::startup::ensure_server_key(&db, &config)
        .await
        .context(StartUpSnafu)?;

    // Authorising the keys just a lil bit
    playerbrainz_server::remote_server_sync::discover_fetch_and_persist_remote_server(
        &db,
        config.href.clone(),
    )
    .await
    .context(RemoteServerSnafu)?;

    let client = reqwest::Client::new();
    let graph_endpoint =
        get_graphql_endpoint("http://localhost:3030/".try_into().unwrap(), &client)
            .await
            .context(RemoteServerSnafu)?;

    let req = playerbrainz_server::federation_request::build_request(
        key.clone(),
        graph_endpoint,
        &config.href,
    )
    .await;
    println!("{req:?}");

    // println!("{:?}", req.extract_signatures().unwrap());

    let (parts, body) = req.into_parts();
    let body = body.into_bytes().await.unwrap();
    let req = http::Request::from_parts(parts, body);
    let req = reqwest::Request::try_from(req).unwrap();
    let res = client.execute(req).await.unwrap();
    println!("{res:?}");
    let res_text = res.text().await.unwrap();

    println!("{res_text:?}");

    // let res_json = res.json::<cynic::GraphQlResponse<ServerInfoQuery>>().await.unwrap();
    // println!("{res_json:?}");

    Ok(())
}
