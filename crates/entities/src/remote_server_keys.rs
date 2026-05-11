use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "remote_server_keys")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub server_href: String,

    #[sea_orm(primary_key, auto_increment = false)]
    pub server_key_id: String,

    pub public_key_bytes: Vec<u8>,

    pub first_seen: DateTimeWithTimeZone,
    pub valid_until: DateTimeWithTimeZone,

    #[sea_orm(belongs_to, from = "server_href", to = "href")]
    pub remote_server: HasOne<super::remote_servers::Entity>,

    #[sea_orm(has_many)]
    pub library_authorizations: HasMany<super::remote_server_key_library_authorizations::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
