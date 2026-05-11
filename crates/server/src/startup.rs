use std::str::FromStr;

use playerbrainz_signatures::KeyAlgorithm;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, SqlErr,
};
use snafu::{ResultExt, Snafu};

use ed25519_compact::*;
use playerbrainz_entities::{ServerKeypair, User, server_keypairs, user};
use strum::VariantArray;

use crate::config::Config;

type Result<T, E = StartupError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum StartupError {
    #[snafu(display("Unable to seed admin user: {}", source))]
    SeedAdminUser { source: sea_orm::DbErr },
    #[snafu(display("Unable to ensure key exists: {}", source))]
    EnsureKey { source: sea_orm::DbErr },
}

pub async fn seed_admin_user(db: &DatabaseConnection) -> Result<()> {
    if let Err(e) = User::insert(user::ActiveModel {
        id: sea_orm::ActiveValue::Set(0),
        ..user::NewUser {
            display_name: None,
            admin: true,
            slug: "admin".to_string(),
            password: password_auth::generate_hash("password"),
        }
        .into_active_model()
    })
    .exec(db)
    .await
        && e.sql_err()
            .filter(|e| matches!(e, SqlErr::UniqueConstraintViolation(_)))
            .is_none()
    {
        return Err(e).context(SeedAdminUserSnafu);
    }
    Ok(())
}

pub async fn ensure_server_key(db: &DatabaseConnection, config: &Config) -> Result<KeyPair> {
    let key = ServerKeypair::find()
        .filter(
            Condition::any()
                .add(server_keypairs::Column::ValidUntil.is_null())
                .add(server_keypairs::Column::ValidUntil.lt(chrono::Utc::now().fixed_offset())),
        )
        .filter(server_keypairs::Column::CreatedAt.lte(chrono::Utc::now().fixed_offset()))
        .filter(
            server_keypairs::Column::Algorithm.is_in(
                KeyAlgorithm::VARIANTS
                    .iter()
                    .map(|k| k.as_ref())
                    .collect::<Vec<&str>>(),
            ),
        )
        .one(db)
        .await
        .context(EnsureKeySnafu)?;
    let key = match key {
        Some(key) => match KeyAlgorithm::from_str(&key.algorithm)
            .expect("Returned result to have selected algorithm")
        {
            KeyAlgorithm::Ed25519 => {
                let mut seed = [0u8; ed25519_compact::Seed::BYTES];
                seed.copy_from_slice(&key.private_key);
                ed25519_compact::KeyPair::from_seed(ed25519_compact::Seed::new(seed))
            }
        },
        None => {
            let key = ed25519_compact::KeyPair::generate();
            ServerKeypair::insert(
                server_keypairs::NewServerKeypair {
                    server_href: config.href.to_string(),
                    algorithm: KeyAlgorithm::Ed25519.to_string(),
                    private_key: key.sk.seed().to_vec(),
                }
                .into_active_model(),
            )
            .exec(db)
            .await
            .context(EnsureKeySnafu)?;
            key
        }
    };

    Ok(key)
}
