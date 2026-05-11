use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "remote_server_key_library_authorizations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub server_href: String,

    #[sea_orm(primary_key, auto_increment = false)]
    pub server_key_id: String,

    #[sea_orm(primary_key, auto_increment = false)]
    pub library_id: i32,

    // Composite FK -> remote_server_keys(server_href, server_key_id)
    #[sea_orm(
        belongs_to,
        from = "(server_href, server_key_id)",
        to = "(server_href, server_key_id)"
    )]
    pub remote_server_key: HasOne<super::remote_server_keys::Entity>,

    #[sea_orm(belongs_to, from = "library_id", to = "id")]
    pub filesystem_library: HasOne<super::filesystem_libraries::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
