use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "filesystem_library")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(nullable)]
    pub display_name: Option<String>,
    #[sea_orm(unique, indexed)]
    pub path: String,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,

    #[sea_orm(has_many)]
    pub remote_server_key_authorizations:
        HasMany<super::remote_server_key_library_authorizations::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(DeriveIntoActiveModel)]
#[sea_orm(
    active_model = "ActiveModel",
    set(updated_at = "chrono::Utc::now().fixed_offset()"),
    set(created_at = "chrono::Utc::now().fixed_offset()")
)]
pub struct NewFsLibrary {
    pub path: String,
    pub display_name: Option<String>,
}

#[derive(DeriveIntoActiveModel)]
#[sea_orm(
    active_model = "ActiveModel",
    set(updated_at = "chrono::Utc::now().fixed_offset()")
)]
pub struct UpdateFsLibrary {
    pub id: i32,
    pub path: Option<String>,
    pub display_name: Option<Option<String>>,
}
