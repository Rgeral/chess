# Chess

Application d’échecs complète avec un backend Rust (GraphQL) et un frontend Svelte. Jouez contre Stockfish avec différents niveaux de difficulté, suivez vos stats et un classement simple.

• Démo en ligne: https://chess.rdevs.fr/

## Aperçu
- Backend: Rust, Axum, Async-GraphQL, SQLx (PostgreSQL), Stockfish (moteur UCI)
- Frontend: SvelteKit + Vite, client GraphQL (graphql-request)
- Base de données: PostgreSQL 16 (volume Docker `pg_data`), migrations auto au démarrage
- API: GraphQL sur `/graphql` (+ IDE GraphiQL sur `/` en dev/containers)
- Conteneurs: Docker + docker-compose (Nginx pour servir le frontend)

## Lancer avec Docker (recommandé)
1) Créez le fichier `.env` à la racine (ignoré par git) à partir du modèle, puis changez le mot de passe:
```
cp .env.example .env
# éditez POSTGRES_PASSWORD (et DATABASE_URL si vous lancez aussi le backend hors Docker)
```
Le service `postgres` a un healthcheck (`pg_isready`) et le backend ne démarre qu’une fois la base saine. Dans Docker, `DATABASE_URL` est construit par `docker-compose.yml` à partir de `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB`.

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
- Les données Postgres vivent dans le volume nommé `pg_data` (`docker compose down` les conserve, `down -v` les supprime)
- L’image backend installe Stockfish automatiquement

## Lancer en développement (sans Docker)
Prérequis:
- Rust (toolchain récente)
- Node.js 20+ et npm
- Stockfish installé et accessible via PATH (Linux: `sudo apt install stockfish`)

1) Backend
```
cd backend
# Démarrer Postgres seul (depuis la racine du repo, .env créé comme ci-dessus)
docker compose up -d postgres

# Une seule fois : fichier de config local (ignoré par git)
# cp .env.example .env   (dans backend/, puis éditez DATABASE_URL: mot de passe et port)
cargo run   # applique les migrations au démarrage
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

## Sauvegarde / restauration (Postgres)
```
# Sauvegarde (service postgres du compose)
docker compose exec -T postgres sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" --clean --if-exists' > chess_$(date +%F).sql

# Restauration
docker compose exec -T postgres sh -c 'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB"' < chess_AAAA-MM-JJ.sql
```
Ne committez jamais les dumps ni le fichier `.env`.

## Dépannage
- Erreur CORS: vérifiez que `CORS_ORIGIN` (backend) correspond exactement à l’URL du frontend (5173 en dev, 8081 en Docker).
- Stockfish introuvable: installez-le localement ou utilisez Docker. Le backend tente plusieurs chemins (`stockfish`, `/usr/games/stockfish`, `/usr/local/bin/stockfish`).
- Base Postgres: les migrations dans `backend/migrations` s’appliquent automatiquement au démarrage. Si le port 5432 est déjà pris, définissez `POSTGRES_PORT` dans `.env`.
- Après modification d’une requête `sqlx::query!`, régénérez les métadonnées offline (utilisées par le build Docker): `cd backend && cargo sqlx prepare` (base migrée requise) puis committez `backend/.sqlx/`.

## Structure
- `backend/`: serveur Axum + GraphQL, services (chess/stockfish), migrations SQLx
- `frontend/`: SvelteKit + Vite, client GraphQL, composants UI
- `docker-compose.yml`: orchestration backend + frontend (Nginx)

---
Ce projet est accessible en ligne sur https://chess.rdevs.fr/