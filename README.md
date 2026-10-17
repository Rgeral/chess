# Chess

A full-stack chess web application: play against the Stockfish engine at ten difficulty levels, track your statistics and compare yourself on a simple Elo leaderboard.

Live instance: https://chess.rdevs.fr/

## Purpose of this project

The application itself is deliberately simple. **The real goal of this repository is to serve as a test project for publishing an application on my self-hosted Proxmox server.**

It is a realistic but small "guinea pig" that exercises everything a real service needs:

- a compiled backend (Rust) that depends on an external binary (Stockfish)
- a relational database with migrations and persistent storage (PostgreSQL)
- a statically built frontend served by a web server (Nginx)
- multi-container orchestration (Docker Compose, health checks, start-up ordering)
- secrets and configuration management
- a CI/CD pipeline that deploys to a machine that is **not** reachable from the public internet (GitHub Actions → WireGuard VPN → SSH)
- a public domain name pointing to the self-hosted service

If the whole chain works for this project, the same recipe can be reused for future self-hosted applications.

## Features

- Create a player profile (username only, no password; the session is kept in the browser's local storage)
- Play as White against Stockfish, with 10 selectable difficulty levels from beginner to expert
- Interactive board with legal-move highlighting, last-move highlighting, pawn promotion dialog, captured pieces and a game timer
- Resign a game, with an end-of-game summary (result, time, number of moves)
- Per-player statistics: games played, win rate, Elo estimate, per-level results and personal records
- Global leaderboard ranked by estimated Elo
- Responsive dashboard with a sidebar (Home, Statistics, Leaderboard) and a logout button

> Some dashboard widgets (recent games, achievements, learning section) currently display mock data defined in `frontend/src/lib/mock.ts`. They are UI placeholders for future features.

## Tech stack

| Layer | Technology |
| --- | --- |
| Backend language | Rust (edition 2021) |
| Web framework | Axum 0.8 on Tokio |
| API | GraphQL via `async-graphql` (+ GraphiQL IDE in dev/containers) |
| Chess logic | `chess` crate (move generation and validation) |
| AI engine | Stockfish, driven over the UCI protocol |
| Database | PostgreSQL 16 |
| DB access | SQLx 0.8 (compile-time checked queries, embedded migrations) |
| Frontend framework | SvelteKit 2 with Svelte 5, TypeScript, Vite 6 |
| Styling | Tailwind CSS 4 |
| Board UI | Chessground (Lichess board) and chess.js |
| API client | `graphql-request` |
| Static build | `@sveltejs/adapter-static` |
| Web server | Nginx (serves the frontend and proxies `/graphql` to the backend) |
| Containers | Docker and Docker Compose |
| CI/CD | GitHub Actions |
| Tooling | ESLint, Prettier, svelte-check, Vitest |

## Architecture

```
Browser
   │  HTTPS
   ▼
Reverse proxy / public domain (chess.rdevs.fr)
   │  HTTP
   ▼
┌───────────────────── Docker Compose (VM on Proxmox) ─────────────────────┐
│                                                                           │
│  frontend (Nginx :80) ──/graphql──▶ backend (Axum :8080) ──▶ postgres     │
│   static SvelteKit build            GraphQL + Stockfish      (volume      │
│                                      process (UCI)            pg_data)    │
└───────────────────────────────────────────────────────────────────────────┘
```

- The **frontend** is a static SvelteKit build served by Nginx. API calls go to the relative path `/graphql`, which Nginx forwards to the backend, so the browser only ever talks to one origin.
- The **backend** exposes a single GraphQL endpoint. It validates moves with the `chess` crate, asks Stockfish for the AI reply, and persists users, games and statistics in PostgreSQL. Migrations in `backend/migrations` are applied automatically at start-up.
- **PostgreSQL** stores its data in the named volume `pg_data`, so data survives container rebuilds.

## Deployment on Proxmox

The production instance runs in a VM on a self-hosted Proxmox server. Deployments are fully automated by [`.github/workflows/deploy.yml`](.github/workflows/deploy.yml) on every push to `main`:

1. **Connect to the private network.** The GitHub runner brings up a WireGuard tunnel (config stored in a GitHub secret), because the VM is only reachable from the home LAN.
2. **Copy the sources.** `scp` uploads the backend sources, migrations, frontend and Compose files to the VM.
3. **Prepare the environment.** On first deployment only, a `.env` file is generated on the VM from GitHub secrets. Later deployments keep the existing file.
4. **Build.** Fresh Docker images are built on the VM.
5. **Start PostgreSQL** and wait until its health check passes.
6. **Run migrations** with the new backend image before starting the application.
7. **Start the backend** and wait until it reports healthy (a GraphQL `{__typename}` query).
8. **Start the frontend** and wait until Nginx answers.
9. **Smoke tests** through Nginx: the static `/_app/version.json` file and a GraphQL request. Any failure fails the pipeline and prints the container logs.

Required GitHub secrets: `WIREGUARD_CONFIG`, `SERVER_USER`, `SERVER_SSH_KEY`, `POSTGRES_PASSWORD`.

## Running with Docker (recommended)

1. Create the `.env` file at the repository root (git-ignored) from the template and set a real password:

   ```bash
   cp .env.example .env
   # edit POSTGRES_PASSWORD
   ```

2. Build and start everything:

   ```bash
   docker compose up -d --build
   ```

3. Open the app. The compose file publishes the frontend on port 80; the backend is only reachable through Nginx at `/graphql`. To reach the backend directly (for example to use GraphiQL), expose its port 8080 locally.

Stop the stack:

```bash
docker compose down
```

Postgres data lives in the `pg_data` volume: `down` keeps it, `down -v` deletes it. The backend image installs Stockfish automatically.

## Local development (without Docker for the app)

Prerequisites: a recent Rust toolchain, Node.js 20+, and Stockfish on your `PATH` (e.g. `sudo apt install stockfish`). The backend tries `stockfish`, `/usr/games/stockfish` and `/usr/local/bin/stockfish`.

**Backend**

```bash
# From the repository root: start only Postgres
docker compose up -d postgres

cd backend
cp .env.example .env   # then adjust DATABASE_URL (password, port)
cargo run              # applies migrations, listens on http://localhost:8080
```

Endpoints: GraphQL on `/graphql`, GraphiQL on `/`.

**Frontend**

```bash
cd frontend
echo 'VITE_GRAPHQL_API_URL=http://localhost:8080/graphql' > .env
npm install
npm run dev -- --open   # http://localhost:5173
```

Useful scripts: `npm run check` (types), `npm run lint`, `npm test`.

## GraphQL API overview

- Mutations: `createUser`, `createGame`, `makeMove`
- Queries: `getUserProfile`, `getLeaderboard`

Explore the full schema with GraphiQL.

## Backup and restore (PostgreSQL)

```bash
# Backup
docker compose exec -T postgres sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" --clean --if-exists' > chess_$(date +%F).sql

# Restore
docker compose exec -T postgres sh -c 'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB"' < chess_YYYY-MM-DD.sql
```

Never commit dumps or the `.env` file.

## Troubleshooting

- **CORS errors:** `CORS_ORIGIN` (backend) must exactly match the frontend URL (`http://localhost:5173` in dev, your public URL in production).
- **Stockfish not found:** install it locally or use Docker.
- **Port 5432 already in use:** set `POSTGRES_PORT` in `.env`.
- **After editing a `sqlx::query!` query:** regenerate the offline metadata used by the Docker build with `cd backend && cargo sqlx prepare` (a migrated database is required), then commit `backend/.sqlx/`.

## Repository layout

```
backend/              Rust server: Axum + GraphQL, services (chess, Stockfish), SQLx migrations
frontend/             SvelteKit app, GraphQL client, UI components, Nginx config
docker-compose.yml    Production stack: postgres, backend, frontend
.github/workflows/    CI/CD pipeline deploying to the Proxmox VM
```
