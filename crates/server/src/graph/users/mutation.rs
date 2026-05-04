use crate::graph::auth::AdminGuard;
use crate::graph::users::model::User;
use async_graphql::{Context, InputObject, Object, Result};
use playerbrainz_entities::user;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};

#[derive(Default)]
pub struct UserManagementMutation;

#[derive(InputObject)]
pub struct CreateUserInput {
    pub display_name: Option<String>,
    pub slug: String,
    pub admin: bool,
    pub password: String,
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

        let new_user = user::NewUser {
            slug: input.slug,
            password: password_auth::generate_hash(input.password),
            display_name: input.display_name,
            admin: input.admin,
        }
        .into_active_model();

        let new_user = new_user.insert(db).await?;

        Ok(new_user.into())
    }

    #[graphql(guard = "AdminGuard")]
    pub async fn update_user(&self, ctx: &Context<'_>, input: UpdateUserInput) -> Result<User> {
        let db = ctx.data::<DatabaseConnection>()?;
        let existing = user::Entity::find_by_id(input.id)
            .one(db)
            .await?
            .ok_or("User not found")?;

        let update = user::UpdateUser {
            id: existing.id,
            display_name: Some(input.display_name),
            slug: input.slug,
            admin: input.admin,
            password: None,
        }
        .into_active_model();

        let updated_user = update.update(db).await?;

        Ok(updated_user.into())
    }

    #[graphql(guard = "AdminGuard")]
    pub async fn delete_user(&self, ctx: &Context<'_>, input: DeleteUserInput) -> Result<bool> {
        let session = ctx
            .data::<playerbrainz_entities::session::Model>()
            .expect("to be logged in");

        if session.user_id == input.id {
            return Err(async_graphql::Error::new("You cannot delete your own user"));
        }

        let db = ctx.data::<DatabaseConnection>()?;
        let result = user::Entity::delete_by_id(input.id).exec(db).await?;
        Ok(result.rows_affected > 0)
    }
}
