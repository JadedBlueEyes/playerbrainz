//! SeaORM entity definitions
//!
//! This is factored into a separate crate for compile time and reusability reasons.
pub mod filesystem_libraries;
pub mod filesystem_mastering;
pub mod session;
pub mod user;

pub use filesystem_libraries::Entity as FsLibrary;
pub use session::Entity as Session;
pub use user::Entity as User;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    sea_orm::entity::prelude::FromJsonQueryResult,
)]
pub struct UuidVec(pub Vec<uuid::Uuid>);

impl std::ops::Deref for UuidVec {
    type Target = Vec<uuid::Uuid>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
