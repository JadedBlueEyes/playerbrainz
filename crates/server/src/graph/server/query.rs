use crate::{config::Config, graph::server::model};
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct ServerQuery;

#[Object]
impl ServerQuery {
    pub async fn server_info(&self, ctx: &Context<'_>) -> Result<model::Server> {
        let config = ctx.data::<Config>()?;

        Ok(model::Server {
            href: config.href.to_string(),
        })
    }
}
