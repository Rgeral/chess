export const CREATE_USER = `
    mutation CreateUser($username: String!) {
        createUser(username: $username) {
            id
            username
            totalGames
            gamesWon
            estimatedElo
            currentStreak
            bestStreak
        }
    }
`;

export const CREATE_GAME = `
    mutation CreateGame($input: NewGameInput!) {
        createGame(input: $input) {
            id
            userId
            difficulty
            fen
            status
            movesCount
            startTime
        }
    }
`;

export const MAKE_MOVE = `
    mutation MakeMove($input: MakeMoveInput!) {
        makeMove(input: $input) {
            game {
                id
                fen
                status
                result
                movesCount
                durationSeconds
            }
            stockfishMove
            gameOver
            winner
            totalTimeSeconds
            lastMove {
                from
                to
                piece
                color
            }
        }
    }
`;

export const GET_USER_PROFILE = `
    query GetUserProfile($userId: String!) {
        getUserProfile(userId: $userId) {
            user {
                id
                username
                totalGames
                gamesWon
                totalPlayTimeSeconds
                currentStreak
                bestStreak
                estimatedElo
            }
            records {
                difficulty
                bestTimeSeconds
                movesCount
                achievedAt
            }
            levelStats {
                difficulty
                gamesPlayed
                gamesWon
                averageTimeSeconds
                averageMoves
            }
        }
    }
`;

export const GET_LEADERBOARD = `
    query GetLeaderboard($limit: Int) {
        getLeaderboard(limit: $limit) {
            username
            estimatedElo
            totalGames
            gamesWon
            currentStreak
        }
    }
`;
