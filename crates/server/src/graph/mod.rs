pub mod auth;
pub mod fs_libraries;
pub mod query;
pub mod users;

pub use query::UtilQuery;

use async_graphql::MergedObject;

use crate::graph::{
    fs_libraries::{FsLibraryMutation, FsLibraryQuery},
    users::{mutation::UserManagementMutation, query::UserManagementQuery},
};

#[derive(MergedObject, Default)]
pub struct Query(pub FsLibraryQuery, pub UtilQuery, pub UserManagementQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(pub FsLibraryMutation, pub UserManagementMutation);
