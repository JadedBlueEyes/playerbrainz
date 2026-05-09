use async_graphql::{Context, Error, Guard, InputObject, Object, Result, SimpleObject};
use password_auth::VerifyError;
use playerbrainz_entities::{session, user};
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tracing::error;

/// Guard that requires any logged-in user.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SessionGuard;

impl Guard for SessionGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        ctx.data::<session::Model>()
            .map(|_| ())
            .map_err(|_| Error::new("Not logged in"))
    }
}

/// Guard that requires an admin user.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AdminGuard;

impl Guard for AdminGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let user = ctx
            .data::<user::Model>()
            .map_err(|_| Error::new("Not logged in"))?;

        if user.admin {
            return Ok(());
        }

        Err(Error::new("Admin only"))
    }
}

#[derive(Default)]
pub struct AuthMutation;

#[derive(InputObject, Debug)]
pub struct LoginInput {
    pub slug: String,
    pub password: String,
}

#[derive(SimpleObject, Debug)]
pub struct LoginPayload {
    pub token: String,
}

#[Object]
impl AuthMutation {
    /// Log in with slug and password.
    ///
    /// Returns a bearer token that can be used as `Authorization: Bearer <token>`.
    pub async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginPayload> {
        let db = ctx.data::<DatabaseConnection>()?;

        let user_model = user::Entity::find()
            .filter(user::Column::Slug.eq(&input.slug))
            .one(db)
            .await?
            .ok_or_else(|| Error::new("Invalid username or password"))?;

        if let Err(e) = password_auth::verify_password(&input.password, &user_model.password) {
            if e != VerifyError::PasswordInvalid {
                error!(?e, "Error checking password hash");
            }

            return Err(Error::new("Invalid username or password"));
        }

        let mut token_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut token_bytes);

        use base64::Engine;
        let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token_bytes);

        let session = session::ActiveModel {
            id: ActiveValue::Set(token.clone()),
            user_id: ActiveValue::Set(user_model.id),
            created_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
            expires_at: ActiveValue::Set(
                (chrono::Utc::now() + chrono::TimeDelta::try_days(30).unwrap()).fixed_offset(),
            ),
        };

        session.insert(db).await?;

        Ok(LoginPayload { token })
    }
}
