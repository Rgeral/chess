export const CREATE_USER = `
    mutation CreateUser($username: String!) {
        createUser(username: $username) {
            id
            username
            totalGames
            gamesWon
            createdAt
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
                moves
                status
                endTime
            }
            stockfishMove
            gameOver
            winner
        }
    }
`;

export const GET_GAME = `
    query GetGame($gameId: String!) {
        game(gameId: $gameId) {
            id
            userId
            difficulty
            fen
            moves
            status
            startTime
            endTime
        }
    }
`;

export const GET_USER = `
    query GetUser($username: String!) {
        user(username: $username) {
            id
            username
            totalGames
            gamesWon
            gamesLost
            maxDifficultyBeaten
        }
    }
`;