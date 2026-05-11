use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "remote_servers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub href: String,

    pub first_seen: DateTimeWithTimeZone,

    #[sea_orm(has_many)]
    pub keys: HasMany<super::remote_server_keys::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
