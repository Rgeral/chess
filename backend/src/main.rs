mod models;
mod services;
mod database;
mod graphql;

use database::Database;
use graphql::create_schema;

#[tokio::main]
async fn main() {
    println!("🚀 Chess Backend Starting...");
    
    let db = Database::new("sqlite:chess.db").await.unwrap();
    println!("✅ Database connected!");
    
    let schema = create_schema();
    println!("📊 GraphQL schema created!");
    
    // TODO: Setup Axum server with GraphQL endpoint
}