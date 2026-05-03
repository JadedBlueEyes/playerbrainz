use async_graphql::{Context, Error, Object, Result};
use playerbrainz_entities::filesystem_libraries;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use tracing::info;

use crate::graph::auth::AdminGuard;

use super::model::FilesystemLibrary;

#[derive(async_graphql::InputObject, Debug)]
pub struct CreateFilesystemLibraryInput {
    /// Filesystem path for the library. Must be unique.
    pub path: String,
    /// Optional human-friendly name.
    pub display_name: Option<String>,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct UpdateFilesystemLibraryInput {
    pub id: i32,
    pub path: Option<String>,
    pub display_name: Option<String>,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct DeleteFilesystemLibraryInput {
    pub id: i32,
}

/// Mutation fields for filesystem libraries.
///
/// Auth:
/// - admin only
pub struct FsLibraryMutation;

#[Object]
impl FsLibraryMutation {
    #[graphql(guard = "AdminGuard")]
    pub async fn create_filesystem_library<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: CreateFilesystemLibraryInput,
    ) -> Result<FilesystemLibrary> {
        let db = ctx.data::<DatabaseConnection>()?;

        // Use helper that sets created_at/updated_at, then override nullable fields.
        let active: filesystem_libraries::ActiveModel = filesystem_libraries::NewFsLibrary {
            path: input.path,
            display_name: input.display_name,
        }
        .into_active_model();

        let created = active.insert(db).await?;
        Ok(created.into())
    }

    #[graphql(guard = "AdminGuard")]
    pub async fn update_filesystem_library<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: UpdateFilesystemLibraryInput,
    ) -> Result<FilesystemLibrary> {
        let db = ctx.data::<DatabaseConnection>()?;
        info!(?input);

        let existing = filesystem_libraries::Entity::find_by_id(input.id)
            .one(db)
            .await?
            .ok_or_else(|| Error::new("Library not found"))?;

        let active: filesystem_libraries::ActiveModel = filesystem_libraries::UpdateFsLibrary {
            id: existing.id,
            path: input.path,
            display_name: Some(input.display_name),
        }
        .into_active_model();

        let updated = active.update(db).await?;
        Ok(updated.into())
    }

    #[graphql(guard = "AdminGuard")]
    pub async fn delete_filesystem_library<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        input: DeleteFilesystemLibraryInput,
    ) -> Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;

        let res = filesystem_libraries::Entity::delete_by_id(input.id)
            .exec(db)
            .await?;

        Ok(res.rows_affected > 0)
    }
}
