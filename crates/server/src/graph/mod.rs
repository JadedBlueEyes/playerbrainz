pub mod auth;
pub mod fs_libraries;
pub mod query;
pub mod user;

pub use query::UtilQuery;
pub use user::User;

use async_graphql::MergedObject;

use crate::graph::fs_libraries::FsLibraryQuery;

#[derive(MergedObject, Default)]
pub struct Query(pub FsLibraryQuery, pub UtilQuery);

// #[derive(MergedObject, Default)]
// pub struct Mutation(FsLibraryMutation);
