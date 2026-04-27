//! SeaORM entity definitions
//!
//! This is factored into a separate crate for compile time and reusability reasons.
pub mod session;
pub mod user;

pub use session::Entity as Session;
pub use user::Entity as User;
