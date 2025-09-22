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
#[tokio::main]
async fn main() {
    println!("🟡 Starting Chess Backend...");

    // Charger les variables d'environnement
    dotenv().ok();
    println!("✅ .env chargé (ou ignoré si absent)");

    // Récupération des variables
    let cors_origin = env::var("CORS_ORIGIN")
        .unwrap_or_else(|_| {
            println!("⚠️  CORS_ORIGIN non défini, utilisation de la valeur par défaut");
            "http://localhost:5173".to_string()
        });
    println!("🌍 CORS_ORIGIN = {}", cors_origin);

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            println!("⚠️  DATABASE_URL non défini, utilisation de la valeur par défaut");
            "sqlite:chess.db".to_string()
        });
    println!("🗄️ DATABASE_URL = {}", database_url);

    // Connexion à la base
    println!("⏳ Connexion à la base de données...");
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("❌ Échec de la connexion à la base");
    println!("✅ Connecté à la base de données");

    // Exécution des migrations
    println!("⏳ Lancement des migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("❌ Échec des migrations");
    println!("✅ Migrations exécutées avec succès");

    // Création du schéma GraphQL
    println!("⏳ Construction du schéma GraphQL...");
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool.clone())
        .finish();
    println!("✅ Schéma GraphQL prêt");

    // Config CORS
    println!("⏳ Configuration CORS...");
    let cors = CorsLayer::new()
        .allow_origin(cors_origin.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);
    println!("✅ CORS configuré");

    // Définition des routes
    println!("⏳ Construction des routes...");
    let app = Router::new()
        .route("/", get(graphiql))
        .route("/graphql", post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors);
    println!("✅ Routes prêtes");

    // Démarrage du serveur
    println!("🚀 Lancement du serveur sur 0.0.0.0:8080...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("📡 Serveur en écoute sur http://localhost:8080");
    println!("📊 GraphiQL IDE dispo sur http://localhost:8080");
    println!("🎯 Backend prêt à gérer des parties d'échecs !");
    
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
