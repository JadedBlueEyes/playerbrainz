use password_auth::VerifyError;
use playerbrainz_entities::{User, session, user};
use rand::Rng;
use tracing::error;

use crate::error::AppError;
use axum::{Json, extract::State};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    slug: String,
    password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    token: String,
}

pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = User::find()
        .filter(user::Column::Slug.eq(&req.slug))
        .one(&db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid username or password"))?;

    if let Err(e) = password_auth::verify_password(&req.password, &user.password) {
        if e != VerifyError::PasswordInvalid {
            error!(?e, "Error checking password hash")
        }
        return Err(anyhow::anyhow!("Invalid username or password").into());
    }

    let mut token_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut token_bytes);
    use base64::Engine;
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token_bytes);

    let session = session::ActiveModel {
        id: ActiveValue::Set(token.clone()),
        user_id: ActiveValue::Set(user.id),
        created_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        expires_at: ActiveValue::Set(
            (chrono::Utc::now() + chrono::TimeDelta::try_days(30).unwrap()).fixed_offset(),
        ),
    };

    session.insert(&db).await?;

    Ok(Json(LoginResponse { token }))
}
