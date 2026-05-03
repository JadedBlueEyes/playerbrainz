use async_graphql::dataloader::Loader;
use playerbrainz_entities::filesystem_libraries;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;

pub struct FsLibraryByIdLoader {
    pub db: DatabaseConnection,
}

impl Loader<i32> for FsLibraryByIdLoader {
    type Value = filesystem_libraries::Model;
    type Error = sea_orm::DbErr;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let models = filesystem_libraries::Entity::find()
            .filter(filesystem_libraries::Column::Id.is_in(keys.iter().copied()))
            .all(&self.db)
            .await?;

        Ok(models.into_iter().map(|m| (m.id, m)).collect())
    }
}
