use async_graphql::*;

pub struct Query;

#[Object]
impl Query {
    async fn hello(&self) -> String {
        "Hello :3".to_string()
    }
}

pub fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}
