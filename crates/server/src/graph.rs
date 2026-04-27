use async_graphql::*;
use playerbrainz_entities::session;

pub struct Query;

#[Object]
impl Query {
    async fn hello<'ctx>(&self, ctx: &Context<'ctx>) -> String {
        ctx.append_http_header("Meow", "mrrp mrrp");
        "Hello :3".to_string()
    }

    async fn whoami<'ctx>(&self, ctx: &Context<'ctx>) -> Result<String> {
        if let Ok(session) = ctx.data::<session::Model>() {
            Ok(format!("Logged in as user {}", session.user_id))
        } else {
            Err("Not logged in".into())
        }
    }
}
