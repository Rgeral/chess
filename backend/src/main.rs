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
use sqlx::PgPool;
use sqlx::postgres::PgConnectOptions;
use std::str::FromStr;
use tower_http::cors::{Any, CorsLayer};
use std::env;
use tracing::{error, info, warn};
use tracing_subscriber::{self, EnvFilter};
use graphql::{QueryRoot, MutationRoot};
use std::fs::{OpenOptions};
use std::io::Write;

/// Lightweight health probe
async fn healthz() -> &'static str { "ok" }

/// Main application entry point
#[tokio::main]
async fn main() {
    // Init logs le plus tôt possible
    // Respecte RUST_LOG si présent, sinon défaut à `info`
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(log_filter))
        .with_ansi(false)
        .init();

    // Messages très précoces au cas où le logger ne s’initialise pas
    eprintln!("[startup] chess-backend: démarrage main()…");
    println!("[startup] stdout prêt");

    // Trace persistante sur disque pour diagnostiquer les sorties silencieuses
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/app/data/boot.log")
    {
        let _ = writeln!(f, "{} - main() start", chrono::Utc::now());
        let _ = f.flush();
    }

    info!("🔧 Starting Chess Backend...");
    dotenv().ok();

    // Load env vars
    let cors_origin = env::var("CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    info!("🌍 CORS_ORIGIN = {}", cors_origin);

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://user:pass@localhost:5432/chess)");
    info!("🗄️ Database URL configured");

    // Connect to database
    info!("🔌 Connecting to database...");
    let connect_opts = match PgConnectOptions::from_str(&database_url) {
        Ok(opts) => opts,
        Err(e) => {
            error!("❌ Invalid DATABASE_URL: {}", e);
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open("/app/data/boot.log") {
                let _ = writeln!(f, "{} - Invalid DATABASE_URL: {}", chrono::Utc::now(), e);
                let _ = f.flush();
            }
            std::process::exit(1);
        }
    };

    let pool = match PgPool::connect_with(connect_opts).await {
        Ok(pool) => {
            info!("✅ Connected to database");
            pool
        }
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open("/app/data/boot.log") {
                let _ = writeln!(f, "{} - DB connect error: {}", chrono::Utc::now(), e);
                let _ = f.flush();
            }
            std::process::exit(1);
        }
    };

    // Run migrations
    info!("📦 Running migrations...");
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        error!("❌ Failed to run migrations: {}", e);
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open("/app/data/boot.log") {
            let _ = writeln!(f, "{} - Migration error: {}", chrono::Utc::now(), e);
            let _ = f.flush();
        }
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
        .route("/healthz", get(healthz))
        .layer(Extension(schema))
        .layer(cors);

    // Start server
    info!("🚀 Binding server on 0.0.0.0:8080 ...");
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap_or_else(|e| panic!("❌ Failed to bind port {}: {}", port, e));

    info!("✅ Server bound successfully!");
    info!("🚀 Chess GraphQL API ready at http://0.0.0.0:8080/graphql");
    info!("📊 GraphiQL IDE available at http://0.0.0.0:8080");

    // Démarre le serveur HTTP
    match axum::serve(listener, app).await {
        Ok(()) => {
            // En théorie ne se produit qu’en arrêt gracieux. Si ça arrive au lancement, on veut le voir.
            warn!("⚠️ Server exited gracefully (unexpected early exit)");
            // Évite une sortie silencieuse et garde le conteneur vivant pour inspection
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        }
        Err(e) => {
            error!("❌ Server crashed: {}", e);
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open("/app/data/boot.log") {
                let _ = writeln!(f, "{} - Server crashed: {}", chrono::Utc::now(), e);
                let _ = f.flush();
            }
            // Échec explicite pour éviter code de sortie 0
            std::process::exit(1);
        }
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
