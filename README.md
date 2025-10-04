# Chess

Application d’échecs complète avec un backend Rust (GraphQL) et un frontend Svelte. Jouez contre Stockfish avec différents niveaux de difficulté, suivez vos stats et un classement simple.

• Démo en ligne: https://chess.rdevs.fr/

## Aperçu
- Backend: Rust, Axum, Async-GraphQL, SQLx (SQLite), Stockfish (moteur UCI)
- Frontend: SvelteKit + Vite, client GraphQL (graphql-request)
- Base de données: SQLite (fichier `backend/chess.db`), migrations auto au démarrage
- API: GraphQL sur `/graphql` (+ IDE GraphiQL sur `/` en dev/containers)
- Conteneurs: Docker + docker-compose (Nginx pour servir le frontend)

## Lancer avec Docker (recommandé)
1) Créez les variables d’environnement (une fois) pour le build:

- `backend/.env.production` (copié comme `.env` dans l’image):
```
DATABASE_URL=sqlite:chess.db
# Si vous utilisez le frontend servi par Nginx sur 8081
CORS_ORIGIN=http://localhost:8081
```

- `frontend/.env.production` (copié comme `.env` dans l’image):
```
# L’URL vue par le navigateur
VITE_GRAPHQL_API_URL=http://localhost:8080/graphql
```

2) Démarrez les services:
```
docker compose up -d --build
```

3) Accédez à:
- Frontend: http://localhost:8081
- API GraphQL: http://localhost:8080/graphql
- GraphiQL (IDE): http://localhost:8080/

Arrêt:
```
docker compose down
```

Notes:
- Le fichier SQLite est monté: `./backend/chess.db -> /app/chess.db`
- L’image backend installe Stockfish automatiquement

## Lancer en développement (sans Docker)
Prérequis:
- Rust (toolchain récente)
- Node.js 20+ et npm
- Stockfish installé et accessible via PATH (Linux: `sudo apt install stockfish`)

1) Backend
```
cd backend
# Créez un fichier .env si besoin
# DATABASE_URL par défaut: sqlite:chess.db
cat > .env <<'EOF'
DATABASE_URL=sqlite:chess.db
CORS_ORIGIN=http://localhost:5173
EOF

sqlx migrate run if DB does not exist

cargo run
```
Le backend écoute sur http://localhost:8080, exécute les migrations SQLx et expose:
- GraphQL: `/graphql`
- GraphiQL: `/`

2) Frontend
```
cd frontend
cat > .env <<'EOF'
VITE_GRAPHQL_API_URL=http://localhost:8080/graphql
EOF
npm install
npm run dev -- --open
```
Le frontend écoute par défaut sur http://localhost:5173.

## API GraphQL (aperçu)
Mutations/queries principales disponibles côté frontend:
- `createUser`, `createGame`, `makeMove`
- `getUserProfile`, `getLeaderboard`

Vous pouvez explorer le schéma et tester des requêtes via GraphiQL sur http://localhost:8080/.

## Dépannage
- Erreur CORS: vérifiez que `CORS_ORIGIN` (backend) correspond exactement à l’URL du frontend (5173 en dev, 8081 en Docker).
- Stockfish introuvable: installez-le localement ou utilisez Docker. Le backend tente plusieurs chemins (`stockfish`, `/usr/games/stockfish`, `/usr/local/bin/stockfish`).
- Base SQLite: le fichier `backend/chess.db` est créé/validé au démarrage; les migrations dans `backend/migrations` s’appliquent automatiquement.

## Structure
- `backend/`: serveur Axum + GraphQL, services (chess/stockfish), migrations SQLx
- `frontend/`: SvelteKit + Vite, client GraphQL, composants UI
- `docker-compose.yml`: orchestration backend + frontend (Nginx)

---
Ce projet est accessible en ligne sur https://chess.rdevs.fr/