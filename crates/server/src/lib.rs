pub mod discovery_endpoint;
pub mod federation_request;
pub mod graph;
pub mod http_signatures;
pub mod indexer;
pub mod remote_server_sync;

pub mod config;
pub mod shutdown;
pub mod startup;

#[cynic::schema("playerbrainz")]
mod schema {}
