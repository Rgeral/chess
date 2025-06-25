use crate::models::User;
use uuid::Uuid;
use chrono::Utc;

/// Service for managing user operations
pub struct UserService;

impl UserService {
    /// Create a new user or return existing one by username
    pub async fn get_or_create_user(username: String) -> Result<User, String> {
        // TODO: Check if user exists in database
        // For now, create a mock user
        let user = User {
            id: Uuid::new_v4().to_string(),
            username,
            created_at: Utc::now(),
            total_games: 0,
            games_won: 0,
            games_lost: 0,
            games_draw: 0,
            max_difficulty_beaten: 0,
            last_played: None,
        };
        
        println!("Created/found user: {}", user.username);
        Ok(user)
    }
    
    /// Get user by ID
    pub async fn get_user_by_id(user_id: String) -> Result<Option<User>, String> {
        // TODO: Query database
        println!("Looking for user: {}", user_id);
        Ok(None)
    }
}