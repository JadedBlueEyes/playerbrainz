use async_graphql::{Context, Object, Result, SimpleObject};
use base64::Engine;
use playerbrainz_entities::{ServerKeypair, server_keypairs};
use playerbrainz_signatures::KeyAlgorithm;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use std::str::FromStr;

#[derive(SimpleObject)]
pub struct ServerPublicKey {
    pub id: String,
    pub content: String,
}

pub struct Server {
    pub href: String,
}

#[Object]
impl Server {
    pub async fn href(&self) -> &str {
        &self.href
    }

    pub async fn public_keys(&self, ctx: &Context<'_>) -> Result<Vec<ServerPublicKey>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let keys = ServerKeypair::find()
            .filter(
                Condition::any()
                    .add(server_keypairs::Column::ValidUntil.is_null())
                    .add(server_keypairs::Column::ValidUntil.gt(chrono::Utc::now().fixed_offset())),
            )
            .filter(server_keypairs::Column::CreatedAt.lte(chrono::Utc::now().fixed_offset()))
            .filter(server_keypairs::Column::ServerHref.eq(&self.href))
            .all(db)
            .await?;

        let mut public_keys = Vec::new();
        for key in keys {
            if let Ok(algo) = KeyAlgorithm::from_str(&key.algorithm)
                && algo == KeyAlgorithm::Ed25519
            {
                let mut seed = [0u8; ed25519_compact::Seed::BYTES];
                seed.copy_from_slice(&key.private_key);
                let kp = ed25519_compact::KeyPair::from_seed(ed25519_compact::Seed::new(seed));
                public_keys.push(ServerPublicKey {
                    id: playerbrainz_signatures::key::PublicKey::Ed25519(kp.pk).key_id(),
                    content: base64::engine::general_purpose::STANDARD.encode(kp.pk.as_ref()),
                });
            }
        }

        Ok(public_keys)
    }
}
