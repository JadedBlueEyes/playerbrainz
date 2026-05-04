use crate::graph::users::model::User;
use async_graphql::{Context, Object, Result};
use sea_orm::{DatabaseConnection, EntityTrait};

use playerbrainz_entities::{session, user};

use crate::graph::auth::SessionGuard;

#[derive(Default)]
pub struct UserManagementQuery;

#[Object]
impl UserManagementQuery {
    #[graphql(guard = "SessionGuard")]
    pub async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let users: Vec<User> = user::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|user| User {
                id: user.id,
                display_name: user.display_name,
                slug: user.slug,
                admin: user.admin,
                created_at: user.created_at,
                updated_at: user.updated_at,
            })
            .collect();
        Ok(users)
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
