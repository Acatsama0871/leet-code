# LeetCode Tracker - Production Edition

A professional full-stack application for tracking LeetCode progress across multiple question lists. Built with Rust (backend), React + TypeScript (frontend), and DuckDB for persistent storage.

## Features

- **GitHub OAuth Authentication** - Secure login with GitHub
- **Multiple Question Lists** - Track progress across NeetCode 150, Meta questions, and more
- **Progress Tracking** - Mark questions as done, set difficulty levels
- **Custom Tags** - Create and assign custom tags to questions
- **List Intersections** - View common questions between different lists
- **Beautiful Dark Theme** - Catppuccin Mocha color scheme
- **One-Click Deployment** - Docker Compose for easy deployment

## Tech Stack

### Backend (Rust)
- **Axum** - Modern async web framework
- **DuckDB** - Embedded SQL database
- **OAuth2** - GitHub OAuth authentication
- **Tower HTTP** - Middleware and static file serving

### Frontend (React + TypeScript)
- **Vite** - Fast build tool
- **React Query** - Server state management
- **TanStack Table** - Advanced table features
- **Tailwind CSS** - Utility-first styling
- **Catppuccin Mocha** - Beautiful dark theme

## Prerequisites

- **Docker & Docker Compose** (for deployment)
  - **Important**: Docker needs **at least 8GB RAM** allocated
  - On Docker Desktop: Settings → Resources → Memory → 8GB
  - Building DuckDB from source requires significant memory on ARM64/Apple Silicon
- **OR** Manual setup:
  - Rust 1.75+
  - Node.js 20+
  - npm or yarn

## Quick Start (Docker)

### 1. Clone the Repository

```bash
git clone <your-repo-url>
cd leet-code
```

### 2. Set Up GitHub OAuth

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click **"New OAuth App"**
3. Fill in the details:
   - **Application name**: LeetCode Tracker
   - **Homepage URL**: `http://localhost:3000`
   - **Authorization callback URL**: `http://localhost:3000/api/auth/callback`
4. Click **"Register application"**
5. Copy the **Client ID** and generate a new **Client Secret**

### 3. Configure Environment Variables

```bash
# Copy the example env file
cp .env.example .env

# Edit .env and add your GitHub OAuth credentials
nano .env
```

Update the following values in `.env`:
```bash
GITHUB_CLIENT_ID=your_actual_client_id
GITHUB_CLIENT_SECRET=your_actual_client_secret
SESSION_SECRET=generate_a_random_secret_key_here
```

To generate a secure session secret:
```bash
openssl rand -hex 32
```

### 4. Migrate Existing Data (Optional)

If you have existing data from the Streamlit app:

```bash
# The migration will look for data/02_state/leetcode.duckdb
# and create a new data/leetcode.duckdb with migrated data
cd backend
cargo run --bin migrate
cd ..
```

### 5. Prepare Database

You have two options:

**Option A: Use migration script (if you have old data)**
```bash
# The migration script will copy everything from old database
cd backend
cargo run --bin migrate
cd ..
```

**Option B: Use existing database**
```bash
# If you already have a working data/leetcode.duckdb, you're all set!
# Just make sure it's in the data/ directory
```

### 6. Start the Application

```bash
# One command to build and run everything!
docker compose up -d

# View logs
docker compose logs -f

# Stop the application
docker compose down
```

The app will be available at **http://localhost:3000**

## Manual Setup (Without Docker)

### Backend Setup

```bash
cd backend

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Copy environment file
cp .env.example .env
# Edit .env with your GitHub OAuth credentials

# Run migrations (if you have old data)
cargo run --bin migrate

# Run backend (serves both API and frontend)
cargo run --release
```

The backend serves on `http://localhost:3000`

### Frontend Development

For development with hot reload:

```bash
cd frontend

# Install dependencies
npm install

# Run development server
npm run dev
```

Frontend dev server runs on `http://localhost:5173` and proxies API calls to backend.

To build for production:
```bash
npm run build
# Outputs to ../backend/static/
```

## Project Structure

```
leet-code/
├── backend/               # Rust backend
│   ├── src/
│   │   ├── main.rs       # Server entry point
│   │   ├── config.rs     # Configuration
│   │   ├── models/       # Data models
│   │   ├── db/           # Database layer
│   │   ├── api/          # REST API endpoints
│   │   ├── auth/         # GitHub OAuth
│   │   └── bin/
│   │       └── migrate.rs # Migration script
│   ├── Cargo.toml
│   └── .env.example
│
├── frontend/             # React frontend
│   ├── src/
│   │   ├── components/   # Reusable UI components
│   │   ├── pages/        # Page components
│   │   ├── api/          # API client
│   │   ├── types/        # TypeScript types
│   │   ├── lib/          # Utilities
│   │   └── App.tsx       # Main app component
│   ├── package.json
│   └── vite.config.ts
│
├── data/                 # Data directory
│   ├── 01_raw/          # CSV files
│   └── leetcode.duckdb  # DuckDB database
│
├── Dockerfile           # Multi-stage build
├── docker-compose.yml   # One-click deployment
└── .env.example        # Environment template
```

## API Endpoints

### Authentication
- `GET /api/auth/github` - Initiate GitHub OAuth
- `GET /api/auth/callback` - OAuth callback
- `GET /api/auth/me` - Get current user
- `POST /api/auth/logout` - Logout

### Lists & Questions
- `GET /api/lists` - Get all lists
- `GET /api/lists/:name` - Get questions for a list
- `PUT /api/questions/:number` - Update question (done, difficulty)
- `GET /api/metrics/:list` - Get progress metrics

### Intersections
- `GET /api/intersections` - Get available intersections
- `GET /api/intersections/:id` - Get intersection questions

### Tags
- `GET /api/tags` - Get all tags
- `POST /api/tags` - Create new tag
- `DELETE /api/tags/:name` - Delete tag
- `GET /api/questions/:number/tags` - Get question tags
- `PUT /api/questions/:number/tags` - Update question tags

## Database Schema

### Tables

- **Question Lists** (5 tables): `neetcode_150`, `neetcode_meta_list`, `leetcode_meta_3mo`, `adv_algo_questions`, `pinterest`
- **question_status**: Tracks completion and difficulty
- **tags**: Available tags
- **question_tags**: Many-to-many relationship (questions ↔ tags)
- **user_sessions**: GitHub OAuth sessions

## Production Deployment

### Docker (Recommended)

1. Update `.env` with production values:
   ```bash
   FRONTEND_URL=https://your-domain.com
   BACKEND_URL=https://your-domain.com
   ```

2. Update GitHub OAuth callback URL to match your domain

3. Deploy:
   ```bash
   docker compose up -d
   ```

### Reverse Proxy (Nginx)

Example Nginx configuration:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Development

### Backend Development

```bash
cd backend

# Run in development mode (with logging)
RUST_LOG=debug cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

### Frontend Development

```bash
cd frontend

# Install dependencies
npm install

# Run dev server with hot reload
npm run dev

# Build for production
npm run build

# Lint
npm run lint
```

## Troubleshooting

### Port Already in Use

If port 3000 is already in use, change it in `.env`:
```bash
PORT=8080
```

And update `docker-compose.yml` port mapping.

### Database Locked

If you get "database is locked" errors, make sure:
1. Only one instance is running
2. The old Streamlit app is stopped
3. No other processes are accessing the database

### GitHub OAuth Errors

1. Double-check your `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET`
2. Verify the callback URL matches in both `.env` and GitHub OAuth settings
3. Make sure `FRONTEND_URL` and `BACKEND_URL` are correct

### Migration Issues

If migration fails:
```bash
# Backup your old database first
cp data/02_state/leetcode.duckdb data/02_state/leetcode.duckdb.backup

# Delete the new database and retry
rm data/leetcode.duckdb
cd backend
cargo run --bin migrate
```

## Contributing

This is a personal project, but feel free to fork and customize for your needs!

## License

MIT License - feel free to use this for your own LeetCode tracking!

---

## Differences from Streamlit Version

### Improvements
- **Authentication**: Secure GitHub OAuth (no open access)
- **Performance**: Rust backend is significantly faster
- **Professional UI**: Modern React with Catppuccin theme
- **Type Safety**: Full TypeScript coverage
- **Scalability**: Ready for production deployment
- **Docker**: One-click deployment

### Migration Notes
- All data from the Streamlit app can be migrated using the migration script
- The database schema is compatible (same DuckDB format)
- Question lists, progress, tags, and relationships are preserved

---

**Built with ❤️ using Rust, React, and DuckDB**
