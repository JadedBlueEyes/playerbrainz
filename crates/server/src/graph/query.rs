use async_graphql::{Context, Object, Result};

use playerbrainz_entities::{session, user};
use sea_orm::{DatabaseConnection, EntityTrait};

use super::User;

#[derive(Default)]
pub struct UtilQuery;

#[Object]
impl UtilQuery {
    async fn hello<'ctx>(&self, ctx: &Context<'ctx>) -> String {
        ctx.append_http_header("Meow", "mrrp mrrp");
        "Hello :3".to_string()
    }

    async fn whoami<'ctx>(&self, ctx: &Context<'ctx>) -> Result<User> {
        if let Ok(session) = ctx.data::<session::Model>() {
            let db = ctx.data::<DatabaseConnection>()?;
            let user_model = user::Entity::find_by_id(session.user_id)
                .one(db)
                .await?
                .ok_or_else(|| async_graphql::Error::new("User not found"))?;

            Ok(User {
                id: user_model.id,
                display_name: user_model.display_name,
                slug: user_model.slug,
                admin: user_model.admin,
                created_at: user_model.created_at,
                updated_at: user_model.updated_at,
            })
        } else {
            Err("Not logged in".into())
        }
    }
}
