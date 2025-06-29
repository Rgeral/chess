use async_graphql::{Context, Object, Result, Schema, EmptySubscription};
use crate::models::{User, Game, NewGameInput, MakeMoveInput, GameMoveResult};
use crate::database::Database;
use crate::services::GameService;

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
    
    /// Récupère une partie par son ID
    async fn game(&self, ctx: &Context<'_>, game_id: String) -> Result<Option<Game>> {
        let db = ctx.data::<Database>()?;
        let game = GameService::get_game(db, game_id).await?;
        Ok(game)
    }
    
    /// Récupère les parties d'un utilisateur
    async fn user_games(&self, ctx: &Context<'_>, user_id: String) -> Result<Vec<Game>> {
        let db = ctx.data::<Database>()?;
        let games = GameService::get_user_games(db, user_id).await?;
        Ok(games)
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
    
    /// Crée une nouvelle partie
    async fn create_game(&self, ctx: &Context<'_>, input: NewGameInput) -> Result<Game> {
        let db = ctx.data::<Database>()?;
        let game = GameService::create_game(db, input).await?;
        Ok(game)
    }
    
    /// Fait un coup dans une partie
    async fn make_move(&self, ctx: &Context<'_>, input: MakeMoveInput) -> Result<GameMoveResult> {
        let db = ctx.data::<Database>()?;
        let result = GameService::make_move(db, input).await?;
        Ok(result)
    }
}

/// Fonction pour créer le schéma complet
pub fn create_schema() -> async_graphql::SchemaBuilder<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
}