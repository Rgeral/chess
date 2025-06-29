mod models;
mod services;
mod database;
mod graphql;

use axum::{
    routing::{get, post},
    Router,
    response::Html,
    extract::Extension,
};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use tower_http::cors::CorsLayer;
use database::Database;
use graphql::{Query, Mutation, create_schema};

/// Endpoint GraphQL principal
async fn graphql_handler(
    Extension(schema): Extension<async_graphql::Schema<Query, Mutation, async_graphql::EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// Interface web pour tester GraphQL
async fn graphql_playground() -> Html<String> {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[tokio::main]
async fn main() {
    println!("🚀 Chess Backend Starting...");
    
    // Connexion base de données
    let db = Database::new("sqlite:chess.db").await.unwrap();
    println!("✅ Database connected!");
    
    // Création du schéma GraphQL
    let schema = create_schema()
        .data(db)
        .finish();
    println!("📊 GraphQL schema created!");
    
    // Configuration du serveur Axum
    let app = Router::new()
        .route("/", get(|| async { "🏁 Chess Backend API is running!" }))
        .route("/graphql", post(graphql_handler))
        .route("/playground", get(graphql_playground))
        .layer(Extension(schema))
        .layer(CorsLayer::permissive());
    
    // Lancement du serveur
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🎯 Server running on:");
    println!("   API: http://localhost:8080/graphql");
    println!("   Playground: http://localhost:8080/playground");
    
    axum::serve(listener, app).await.unwrap();
}