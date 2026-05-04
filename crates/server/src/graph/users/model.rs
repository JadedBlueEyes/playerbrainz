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
