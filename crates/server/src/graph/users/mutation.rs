use crate::graph::auth::AdminGuard;
use crate::graph::users::model::User;
use async_graphql::{Context, InputObject, Object, Result};
use playerbrainz_entities::user;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};

#[derive(Default)]
pub struct UserManagementMutation;

#[derive(InputObject)]
pub struct CreateUserInput {
    pub display_name: String,
    pub slug: String,
    pub admin: bool,
}

#[derive(InputObject)]
pub struct UpdateUserInput {
    pub id: i32,
    pub display_name: Option<String>,
    pub slug: Option<String>,
    pub admin: Option<bool>,
}

#[derive(InputObject)]
pub struct DeleteUserInput {
    pub id: i32,
}

#[Object]
impl UserManagementMutation {
    #[graphql(guard = "AdminGuard")]
    pub async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<User> {
        let db = ctx.data::<DatabaseConnection>()?;
        let new_user = user::ActiveModel {
            display_name: Set(Some(input.display_name)),
            slug: Set(input.slug),
            admin: Set(input.admin),
            ..Default::default()
        };

        let new_user = new_user.insert(db).await?;

        Ok(User {
            id: new_user.id,
            display_name: new_user.display_name,
            slug: new_user.slug,
            admin: new_user.admin,
            created_at: new_user.created_at,
            updated_at: new_user.updated_at,
        })
    }

    #[graphql(guard = "AdminGuard")]
    pub async fn update_user(&self, ctx: &Context<'_>, input: UpdateUserInput) -> Result<User> {
        let db = ctx.data::<DatabaseConnection>()?;
        let mut user = user::Entity::find_by_id(input.id)
            .one(db)
            .await?
            .ok_or("User not found")?
            .into_active_model();

        if let Some(display_name) = input.display_name {
            user.display_name = Set(Some(display_name));
        }

        if let Some(slug) = input.slug {
            user.slug = Set(slug);
        }

        if let Some(admin) = input.admin {
            user.admin = Set(admin);
        }

        let updated_user = user.update(db).await?;

        Ok(User {
            id: updated_user.id,
            display_name: updated_user.display_name,
            slug: updated_user.slug,
            admin: updated_user.admin,
            created_at: updated_user.created_at,
            updated_at: updated_user.updated_at,
        })
    }

    #[graphql(guard = "AdminGuard")]
    pub async fn delete_user(&self, ctx: &Context<'_>, input: DeleteUserInput) -> Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;
        let result = user::Entity::delete_by_id(input.id).exec(db).await?;
        Ok(result.rows_affected > 0)
    }
}
