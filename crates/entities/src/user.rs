use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "local_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub display_name: String,
    /// crypt(3) formatted password hash as per `password-hash`
    pub password: String,

    pub admin: bool,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(DeriveIntoActiveModel)]
#[sea_orm(
    active_model = "ActiveModel",
    set(updated_at = "chrono::Utc::now().fixed_offset()"),
    set(created_at = "chrono::Utc::now().fixed_offset()")
)]
pub struct NewUser {
    pub id: i32,
    pub password: String,
    pub admin: bool,
}
