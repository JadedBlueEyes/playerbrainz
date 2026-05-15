pub mod auth;
pub mod fs_libraries;
pub mod query;
pub mod server;
pub mod users;

pub use query::UtilQuery;

use async_graphql::MergedObject;

use crate::graph::{
    auth::AuthMutation,
    fs_libraries::{FsLibraryMutation, FsLibraryQuery},
    server::query::ServerQuery,
    users::{mutation::UserManagementMutation, query::UserManagementQuery},
};

#[derive(MergedObject, Default)]
pub struct Query(
    pub FsLibraryQuery,
    pub UtilQuery,
    pub UserManagementQuery,
    pub ServerQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    pub AuthMutation,
    pub FsLibraryMutation,
    pub UserManagementMutation,
);
