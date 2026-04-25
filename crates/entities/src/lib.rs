//! SeaORM entity definitions
//!
//! This is factored into a separate crate for compile time and reusability reasons.
pub mod user;

pub use user::Entity as User;
