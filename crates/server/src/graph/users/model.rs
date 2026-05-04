use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct User {
    pub id: i32,
    pub display_name: Option<String>,
    pub slug: String,
    pub admin: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<playerbrainz_entities::user::Model> for User {
    fn from(m: playerbrainz_entities::user::Model) -> Self {
        Self {
            id: m.id,
            display_name: m.display_name,
            slug: m.slug,
            admin: m.admin,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}
