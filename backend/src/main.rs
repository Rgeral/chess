mod database;
mod graphql;
mod models;
mod services;

use axum::{
    extract::Extension,
    http::{HeaderValue, Method},
    response::Html,
    routing::{get, post},
    Router,
};
use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use dotenv::dotenv;
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use std::env;

use graphql::{QueryRoot, MutationRoot};

/// Main application entry point
/// Sets up the GraphQL server with SQLite database and CORS support
#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:chess.db".to_string());
    
    // Connect to database
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create GraphQL schema
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool.clone())
        .finish();

    // Configure CORS for frontend access
    let cors = CorsLayer::new()
        .allow_origin(cors_origin.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // Build the application routes
    let app = Router::new()
        .route("/", get(graphiql))
        .route("/graphql", post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🚀 Chess GraphQL API ready at http://localhost:8080");
    println!("📊 GraphiQL IDE available at http://localhost:8080");
    println!("🎯 Ready to serve chess games!");
    
    axum::serve(listener, app).await.unwrap();
}

/// Serves the GraphiQL IDE for testing GraphQL queries
async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// Handles GraphQL requests
async fn graphql_handler(
    schema: Extension<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}