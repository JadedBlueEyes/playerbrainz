use async_graphql::{Context, Error, Guard, Result};
use playerbrainz_entities::{session, user};

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
