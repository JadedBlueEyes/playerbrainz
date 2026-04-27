mod error;
mod graph;
mod login;

use async_graphql::{EmptyMutation, EmptySubscription, Schema, http::GraphiQLSource};
use axum::{
    Router,
    extract::{Extension, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{Database, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, SqlErr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use playerbrainz_entities::{User, session, user};

use crate::graph::Query;
use crate::login::login;

async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn graphql_handler(
    State(db): State<DatabaseConnection>,
    Extension(schema): Extension<Schema<Query, EmptyMutation, EmptySubscription>>,
    headers: HeaderMap,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    let mut req = req.into_inner();

    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
        && let Ok(Some(session)) = session::Entity::find_by_id(token).one(&db).await
    {
        req = req.data(session);
    }

    schema.execute(req).await.into()
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
            slug: "admin".to_string(),
            password: "$argon2i$v=19$m=65536,t=1,p=1$c29tZXNhbHQAAAAAAAAAAA$+r0d29hqEB0yasKr55ZgICsQGSkl0v0kgwhd+U3wyRo".to_string(),
        }.into_active_model()
    }).exec(db).await && e.sql_err().filter(|e| matches!(e, SqlErr::UniqueConstraintViolation(_))).is_none() {
            Err(e)?;
        }

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/login", post(login))
        .layer(Extension(schema))
        .with_state(db.clone());

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
