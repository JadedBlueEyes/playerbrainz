use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "server_keypairs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(indexed)]
    pub server_href: String,

    pub algorithm: String,
    pub private_key: Vec<u8>,

    pub created_at: DateTimeWithTimeZone,

    #[sea_orm(nullable)]
    pub valid_until: Option<DateTimeWithTimeZone>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(DeriveIntoActiveModel)]
#[sea_orm(
    active_model = "ActiveModel",
    set(created_at = "chrono::Utc::now().fixed_offset()")
)]
pub struct NewServerKeypair {
    pub server_href: String,

    pub algorithm: String,
    pub private_key: Vec<u8>,
}
