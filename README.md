# Kadyn Pearce Portfolio

A high-performance portfolio website built with Rust, Actix-web, and SurrealDB.

## Features

- **Tech Stack**: Rust, Actix-web, SurrealDB (embedded), Tera templates
- **Admin Dashboard**: Manage projects, blog posts, skills, and experience via `/admin`
- **GitHub Authentication**: Login with GitHub to access admin features
- **Contact Form**: Email integration via Resend
- **Responsive Design**: Modern, mobile-friendly UI

## Quick Start

### Local Development

```bash
# Clone the repository
git clone https://github.com/kadynjaipearce/new_portfolio.git
cd new_portfolio

# Copy environment variables
cp .env.example .env

# Run the server
cargo run
```

Visit `http://localhost:8080`

### Docker Deployment

```bash
# Build and run with Docker Compose
docker-compose up -d
```

Visit `http://localhost:8080`

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | Path to SurrealDB file | Yes |
| `DATABASE_NS` | Database namespace | Yes |
| `DATABASE_DB` | Database name | Yes |
| `GITHUB_CLIENT_ID` | GitHub OAuth app client ID | Yes |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth app client secret | Yes |
| `GITHUB_REDIRECT_URI` | OAuth callback URL | Yes |
| `ADMIN_GITHUB_USERNAME` | GitHub username for admin access | Yes |
| `SESSION_SECRET` | Secret for session encryption | Yes |
| `RESEND_API_KEY` | Resend API key for emails | No |
| `RESEND_FROM_EMAIL` | Email sender address | No |
| `CONTACT_EMAIL` | Email address for contact form | Yes |

## GitHub OAuth Setup

1. Go to GitHub Settings > Developer settings > OAuth Apps
2. Create a new OAuth App
3. Set Homepage URL to your domain
4. Set Authorization callback URL to `https://yourdomain.com/auth/github/callback`
5. Copy Client ID and Client Secret to your environment variables

## Fly.io Deployment

```bash
# Install flyctl
brew install flyctl

# Login
flyctl auth login

# Launch app (creates fly.toml)
flyctl launch

# Set secrets
flyctl secrets set GITHUB_CLIENT_ID=xxx GITHUB_CLIENT_SECRET=xxx SESSION_SECRET=xxx DATABASE_URL=file://data/portfolio.db DATABASE_NS=portfolio DATABASE_DB=main ADMIN_GITHUB_USERNAME=yourusername CONTACT_EMAIL=you@example.com

# Deploy
flyctl deploy
```

## Managing Content

1. Visit `/admin` and login with GitHub
2. Add your projects, experience, skills, and blog posts
3. Content is stored in the database and persists across deployments

## Project Structure

```
portfolio/
├── src/
│   ├── main.rs          # Entry point
│   ├── config.rs        # Configuration
│   ├── db/              # Database setup
│   ├── models/          # Data models
│   ├── routes/          # HTTP handlers
│   ├── services/        # Email, GitHub services
│   └── middleware/      # Auth middleware
├── templates/           # Tera templates
├── static/              # CSS, JS, images
├── Dockerfile           # Docker image
├── docker-compose.yml   # Docker Compose config
└── .env.example         # Environment template
```

## License

MIT
