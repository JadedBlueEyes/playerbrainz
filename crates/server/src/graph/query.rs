use async_graphql::{Context, Object};

#[derive(Default)]
pub struct UtilQuery;

#[Object]
impl UtilQuery {
    async fn hello<'ctx>(&self, ctx: &Context<'ctx>) -> String {
        ctx.append_http_header("Meow", "mrrp mrrp");
        "Hello :3".to_string()
    }
}
