use async_graphql::{Context, Object, Result, Schema, EmptySubscription};
use crate::models::User;
use crate::database::Database;

/// Les queries (lectures)
pub struct Query;

#[Object]
impl Query {
    /// Récupère un utilisateur par son nom
    async fn user(&self, ctx: &Context<'_>, username: String) -> Result<Option<User>> {
        let db = ctx.data::<Database>()?;
        let user = db.find_user_by_username(&username).await?;
        Ok(user)
    }
    
    /// Test simple
    async fn hello(&self) -> &str {
        "Hello from GraphQL!"
    }
}

/// Les mutations (écritures)
pub struct Mutation;

#[Object]
impl Mutation {
    /// Crée un nouvel utilisateur
    async fn create_user(&self, ctx: &Context<'_>, username: String) -> Result<User> {
        let db = ctx.data::<Database>()?;
        
        // Check si l'utilisateur existe déjà
        if let Some(existing_user) = db.find_user_by_username(&username).await? {
            return Ok(existing_user);
        }
        
        // Sinon créer un nouveau
        let user = db.create_user(&username).await?;
        Ok(user)
    }
}

/// Fonction pour créer le schéma complet
pub fn create_schema() -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription).finish()
}