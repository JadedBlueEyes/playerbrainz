mod discovery_endpoint;
mod graph;
mod indexer;

mod config;
mod shutdown;
mod startup;

use std::process::exit;

use async_graphql::{EmptySubscription, Schema, dataloader::DataLoader, http::GraphiQLSource};
use axum::{
    Router,
    extract::{Extension, State},
    response::IntoResponse,
    routing::get,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
    typed_header::TypedHeaderRejection,
};
use sea_orm::{Database, DatabaseConnection, EntityTrait};
use snafu::{ResultExt, Snafu};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use playerbrainz_entities::{session, user};

use crate::{
    config::{ConfigError, config},
    discovery_endpoint::discovery,
    startup::StartupError,
};
use crate::{
    graph::{Mutation, Query},
    shutdown::shutdown_signal,
};

async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn graphql_handler(
    State(db): State<DatabaseConnection>,
    Extension(schema): Extension<Schema<Query, Mutation, EmptySubscription>>,
    auth: std::result::Result<TypedHeader<Authorization<Bearer>>, TypedHeaderRejection>,
    // jar: CookieJar,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    let token = auth.as_ref().ok().map(|h| h.token());
    // .or_else(|| jar.get("pb_token").map(|c| c.value()));
    let mut req = req.into_inner();

    if let Some(token) = token
        && let Ok(Some(session)) = session::Entity::find_by_id(token).one(&db).await
        && let Ok(Some(user)) = user::Entity::find_by_id(session.user_id).one(&db).await
    {
        req = req.data(session).data(user);
    }

    schema.execute(req).await.into()
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
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
}

#[tokio::main]
async fn main() {
    let Err(e) = async_main().await else {
        return;
    };
    eprint!("{e}");
    exit(1)
}

async fn async_main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = dbg!(config().context(ConfigSnafu)?);

    let database_url = config.database_url.clone();

    let db = Database::connect(database_url.clone())
        .await
        .context(DatabaseConnectSnafu {
            database_url: database_url.clone(),
        })?;
    // synchronizes database schema with entity definitions
    db.get_schema_registry("playerbrainz_entities::*")
        .sync(&db)
        .await
        .context(SyncDatabaseSchemaSnafu)?;

    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db.clone())
        .data(DataLoader::new(
            crate::graph::fs_libraries::FsLibraryByIdLoader { db: db.clone() },
            tokio::spawn,
        ))
        .finish();
    let db = &db;

    startup::seed_admin_user(db).await.context(StartUpSnafu)?;

    let _key = startup::ensure_server_key(db, &config)
        .await
        .context(StartUpSnafu)?;

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/.well-known/playerbrainz/client", get(discovery))
        .route("/.well-known/playerbrainz/server", get(discovery))
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::AllowOrigin::mirror_request())
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                ])
                .allow_credentials(true),
        )
        .with_state(db.clone());

    let addr = config.listen_addr.clone();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(BindTcpListenerSnafu { addr })?;

    let indexer = indexer::indexer_task(db, config.clone());
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    let (indexer_result, server_result) = tokio::join!(indexer, server);
    if let Err(e) = indexer_result {
        tracing::error!("Indexer task failed: {}", e);
    }
    if let Err(e) = server_result {
        tracing::error!("Server task failed: {}", e);
    }
    Ok(())
}

async fn serve_index() -> &'static str {
    r"Hey there! Looks like you've reached the API server.
If you just want to play music, you probably shouldn't be here!

API docs at /graphql
"
}
