use crate::models::User;
use uuid::Uuid;
use chrono::Utc;

/// Service responsible for managing user operations
pub struct UserService;

impl UserService {
    /// Creates a new user with default statistics
    /// 
    /// # Arguments
    /// * `username` - Display name for the user
    /// 
    /// # Returns
    /// User instance with initialized default values
    pub fn create_user(username: String) -> User {
        User {
            id: Uuid::new_v4().to_string(),
            username,
            total_games: 0,
            games_won: 0,
            created_at: Utc::now(),
            total_play_time_seconds: Some(0),
            current_streak: Some(0),
            best_streak: Some(0),
            estimated_elo: Some(800),
        }
    }
}