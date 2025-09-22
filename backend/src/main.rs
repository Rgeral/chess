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
use tracing::{error, info};
use tracing_subscriber;
use graphql::{QueryRoot, MutationRoot};

/// Main application entry point
#[tokio::main]
async fn main() {
    // Init logs
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🔧 Starting Chess Backend...");
    dotenv().ok();

    // Load env vars
    let cors_origin = env::var("CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    info!("🌍 CORS_ORIGIN = {}", cors_origin);

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:chess.db".to_string());
    info!("🗄️ DATABASE_URL = {}", database_url);

    // Connect to database
    info!("🔌 Connecting to database...");
    let pool = match SqlitePool::connect(&database_url).await {
        Ok(pool) => {
            info!("✅ Connected to database");
            pool
        }
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Run migrations
    info!("📦 Running migrations...");
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        error!("❌ Failed to run migrations: {}", e);
        std::process::exit(1);
    }
    info!("✅ Migrations applied");

    // Create GraphQL schema
    info!("🔧 Building GraphQL schema...");
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool.clone())
        .finish();
    info!("✅ GraphQL schema ready");

    // Configure CORS
    info!("🔧 Configuring CORS...");
    let cors = CorsLayer::new()
        .allow_origin(cors_origin.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);
    info!("✅ CORS configured");

    // Build routes
    let app = Router::new()
        .route("/", get(graphiql))
        .route("/graphql", post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors);

    // Start server
    info!("🚀 Binding server on 0.0.0.0:8080 ...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("❌ Failed to bind port 8080");

    info!("✅ Server bound successfully!");
    info!("🚀 Chess GraphQL API ready at http://0.0.0.0:8080/graphql");
    info!("📊 GraphiQL IDE available at http://0.0.0.0:8080");

    if let Err(e) = axum::serve(listener, app).await {
        error!("❌ Server crashed: {}", e);
    }
}

/// Serves the GraphiQL IDE
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
