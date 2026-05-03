use async_graphql::{Context, Object, Result, dataloader::DataLoader};
use playerbrainz_entities::filesystem_libraries;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::graph::auth::SessionGuard;

use super::{loader::FsLibraryByIdLoader, model::FilesystemLibrary};

/// Query fields for filesystem libraries.
#[derive(Default)]
pub struct FsLibraryQuery;

#[Object]
impl FsLibraryQuery {
    #[graphql(guard = "SessionGuard")]
    pub async fn filesystem_libraries<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<Vec<FilesystemLibrary>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let libs = filesystem_libraries::Entity::find().all(db).await?;
        Ok(libs.into_iter().map(Into::into).collect())
    }

    #[graphql(guard = "SessionGuard")]
    pub async fn filesystem_library_by_id<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        id: i32,
    ) -> Result<Option<FilesystemLibrary>> {
        let loader = ctx.data::<DataLoader<FsLibraryByIdLoader>>()?;
        let lib: Option<filesystem_libraries::Model> = loader.load_one(id).await?;
        Ok(lib.map(Into::into))
    }

    #[graphql(guard = "SessionGuard")]
    pub async fn filesystem_library_by_path<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        path: String,
    ) -> Result<Option<FilesystemLibrary>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let lib = filesystem_libraries::Entity::find()
            .filter(filesystem_libraries::Column::Path.eq(path))
            .one(db)
            .await?;

        Ok(lib.map(Into::into))
    }
}
