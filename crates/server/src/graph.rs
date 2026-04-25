use async_graphql::*;

pub struct Query;

#[Object]
impl Query {
    async fn hello<'ctx>(&self, ctx: &Context<'ctx>) -> String {
        ctx.append_http_header("Meow", "mrrp mrrp");
        "Hello :3".to_string()
    }
}
