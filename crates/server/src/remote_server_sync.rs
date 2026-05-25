use base64::Engine;
use chrono::{FixedOffset, TimeDelta};
use cynic::{
    QueryBuilder,
    http::{CynicReqwestError, ReqwestExt},
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use snafu::{ResultExt, Snafu};
use url::Url;

use playerbrainz_entities::{RemoteServer, RemoteServerKey, remote_server_keys, remote_servers};

pub type Result<T, E = RemoteServerSyncError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum RemoteServerSyncError {
    #[snafu(display("Request error: {source}"))]
    Request { source: reqwest::Error },

    #[snafu(display("GraphQL request error: {source}"))]
    GraphQlRequest { source: CynicReqwestError },

    #[snafu(display("Discovery response was invalid"))]
    BadDiscoveryJson { source: serde_json::Error },

    #[snafu(display("Remote server returned GraphQL errors: {errors:?}"))]
    RemoteGraphQlErrors { errors: Vec<cynic::GraphQlError> },

    #[snafu(display("Remote server returned empty GraphQL data"))]
    MissingGraphQlData,

    #[snafu(display(
        "Remote server returned invalid base64 public key for id '{key_id}': {source}"
    ))]
    BadPublicKeyBase64 {
        key_id: String,
        source: base64::DecodeError,
    },

    #[snafu(display("Database error: {source}"))]
    Database { source: sea_orm::DbErr },

    #[snafu(display("Remote server href '{href}' cannot be a base URL"))]
    HrefCannotBeBase { href: Url },

    #[snafu(display("Remote server returned invalid graph endpoint '{endpoint}'"))]
    InvalidGraphEndpoint {
        endpoint: String,
        source: url::ParseError,
    },
}

#[derive(Debug, Clone)]
pub struct SyncRemoteServerResult {
    pub server_href: String,
    pub keys: Vec<RemoteKeyRecord>,
}

#[derive(Debug, Clone)]
pub struct RemoteKeyRecord {
    pub algorithm: String,
    pub key_id: String,
    pub public_key_bytes: Vec<u8>,
}

#[derive(serde::Deserialize, Debug)]
struct DiscoveryResponse {
    graph_endpoint: String,
}

pub async fn get_graphql_endpoint(remote_href: Url, client: &reqwest::Client) -> Result<Url> {
    if remote_href.cannot_be_a_base() {
        return Err(RemoteServerSyncError::HrefCannotBeBase { href: remote_href });
    }

    let mut discovery_endpoint = remote_href.clone();
    discovery_endpoint.set_path("/.well-known/playerbrainz/server");
    discovery_endpoint.set_query(None);
    discovery_endpoint.set_fragment(None);

    let discovery_text = client
        .get(discovery_endpoint)
        .send()
        .await
        .context(RequestSnafu)?
        .error_for_status()
        .context(RequestSnafu)?
        .text()
        .await
        .context(RequestSnafu)?;

    let discovery: DiscoveryResponse =
        serde_json::from_str(&discovery_text).context(BadDiscoveryJsonSnafu)?;

    let graph_endpoint = Url::parse(&discovery.graph_endpoint)
        .or_else(|_| remote_href.join(&discovery.graph_endpoint))
        .context(InvalidGraphEndpointSnafu {
            endpoint: discovery.graph_endpoint.clone(),
        })?;
    Ok(graph_endpoint)
}

/// Discover a remote server's GraphQL endpoint, fetch `serverInfo { href publicKeys { id algorithm content } }`,
/// and persist the server + keys in the local database.
pub async fn discover_fetch_and_persist_remote_server(
    db: &DatabaseConnection,
    base_href: Url,
) -> Result<SyncRemoteServerResult> {
    let client = reqwest::Client::new();
    let graph_endpoint = get_graphql_endpoint(base_href, &client).await?;

    let operation = gql::ServerInfoQuery::build(());

    let gql_response: cynic::GraphQlResponse<gql::ServerInfoQuery> = client
        .post(graph_endpoint)
        .run_graphql(operation)
        .await
        .context(GraphQlRequestSnafu)?;

    if let Some(errors) = &gql_response.errors {
        return Err(RemoteGraphQlErrorsSnafu {
            errors: errors.clone(),
        }
        .build());
    }

    let data = gql_response
        .data
        .ok_or(RemoteServerSyncError::MissingGraphQlData)?;

    let server_href = data.server_info.href;

    let mut keys = Vec::new();
    for k in data.server_info.public_keys {
        let public_key_bytes = base64::engine::general_purpose::STANDARD
            .decode(&k.content)
            .context(BadPublicKeyBase64Snafu {
                key_id: k.id.clone(),
            })?;

        keys.push(RemoteKeyRecord {
            algorithm: k.algorithm,
            key_id: k.id,
            public_key_bytes,
        });
    }

    persist_remote_server_and_keys(db, &server_href, &keys).await?;

    Ok(SyncRemoteServerResult { server_href, keys })
}

async fn persist_remote_server_and_keys(
    db: &DatabaseConnection,
    server_href: &str,
    keys: &[RemoteKeyRecord],
) -> Result<()> {
    let now: chrono::DateTime<FixedOffset> = chrono::Utc::now().fixed_offset();

    // Insert remote server if new.
    let existing = RemoteServer::find_by_id(server_href.to_string())
        .one(db)
        .await
        .context(DatabaseSnafu)?;

    if existing.is_none() {
        let am = remote_servers::ActiveModel {
            href: ActiveValue::Set(server_href.to_string()),
            first_seen: ActiveValue::Set(now),
        };
        am.insert(db).await.context(DatabaseSnafu)?;
    }

    // Keys have no explicit expiry in the public API; keep them fresh for a window.
    // If we stop seeing a key, it will naturally expire.
    let valid_for = TimeDelta::try_days(1).unwrap();
    let valid_until: chrono::DateTime<FixedOffset> =
        (chrono::Utc::now() + valid_for).fixed_offset();

    for key in keys {
        // The schema uses (server_href, server_key) as the primary key. We use the remote server's
        // advertised key id as that stable identifier.
        let existing = RemoteServerKey::find_by_id((server_href.to_string(), key.key_id.clone()))
            .one(db)
            .await
            .context(DatabaseSnafu)?;

        match existing {
            Some(model) => {
                let mut am: remote_server_keys::ActiveModel = model.into();
                am.server_key_id = ActiveValue::Set(key.key_id.clone());
                am.algorithm = ActiveValue::Set(key.algorithm.clone());
                am.public_key_bytes = ActiveValue::Set(key.public_key_bytes.clone());
                am.valid_until = ActiveValue::Set(valid_until);
                am.update(db).await.context(DatabaseSnafu)?;
            }
            None => {
                let am = remote_server_keys::ActiveModel {
                    server_href: ActiveValue::Set(server_href.to_string()),
                    server_key_id: ActiveValue::Set(key.key_id.clone()),
                    algorithm: ActiveValue::Set(key.algorithm.clone()),
                    public_key_bytes: ActiveValue::Set(key.public_key_bytes.clone()),
                    first_seen: ActiveValue::Set(now),
                    valid_until: ActiveValue::Set(valid_until),
                };
                am.insert(db).await.context(DatabaseSnafu)?;
            }
        }
    }

    Ok(())
}

mod schema {
    cynic::use_schema!("../../packages/playerbrainz/schema.graphql");
}

// #[cynic::schema_for_derives(file = "../../packages/playerbrainz/schema.graphql", module = "schema")]
pub mod gql {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query")]
    pub struct ServerInfoQuery {
        #[cynic(rename = "serverInfo")]
        pub server_info: Server,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Server")]
    pub struct Server {
        pub href: String,

        #[cynic(rename = "publicKeys")]
        pub public_keys: Vec<ServerPublicKey>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "ServerPublicKey")]
    pub struct ServerPublicKey {
        pub id: String,
        pub algorithm: String,
        pub content: String,
    }
}
