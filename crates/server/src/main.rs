mod graph;

use async_graphql::{EmptyMutation, EmptySubscription, Schema, http::GraphiQLSource};
use async_graphql_axum::GraphQL;
use axum::{Router, response::IntoResponse, routing::get};
use playerbrainz_entities::{User, user};
use sea_orm::{Database, EntityTrait, IntoActiveModel, SqlErr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::graph::Query;

async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://music.db?mode=rwc".to_string());

    let db = Database::connect(database_url).await?;
    // synchronizes database schema with entity definitions
    db.get_schema_registry("playerbrainz_entities::*")
        .sync(&db)
        .await?;

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(db.clone())
        .finish();
    let db = &db;

    if let Err(e) = User::insert(user::ActiveModel {
        admin: sea_orm::ActiveValue::Set(true),
        ..user::NewUser {
            id: 0,
            slug: "admin",
            password: "$argon2i$v=19$m=65536,t=1,p=1$c29tZXNhbHQAAAAAAAAAAA$+r0d29hqEB0yasKr55ZgICsQGSkl0v0kgwhd+U3wyRo".to_string(),
        }.into_active_model()
    }).exec(db).await && e.sql_err().filter(|e| matches!(e, SqlErr::UniqueConstraintViolation(_))).is_none() {
            Err(e)?;
        }

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/graphql", get(graphiql).post_service(GraphQL::new(schema)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_index() -> &'static str {
    r"Hey there! Looks like you've reached the API server.
If you just want to play music, you probably shouldn't be here!

API docs at /graphql
"
}
