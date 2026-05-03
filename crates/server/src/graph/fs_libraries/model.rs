use async_graphql::SimpleObject;
use playerbrainz_entities::filesystem_libraries;

#[derive(SimpleObject, Clone)]
pub struct FilesystemLibrary {
    pub id: i32,
    pub display_name: Option<String>,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<filesystem_libraries::Model> for FilesystemLibrary {
    fn from(m: filesystem_libraries::Model) -> Self {
        Self {
            id: m.id,
            display_name: m.display_name,
            path: m.path,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}
