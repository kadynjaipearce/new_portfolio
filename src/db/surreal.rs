use crate::config::CONFIG;
use crate::error::AppError;
use crate::models::{Experience, Post, Project, Skill};
use once_cell::sync::OnceCell;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;
use tracing::info;

static DB: OnceCell<Surreal<Db>> = OnceCell::new();

pub async fn init_db() -> Result<(), AppError> {
    let db_path = CONFIG
        .database_url
        .strip_prefix("file://")
        .unwrap_or("data/portfolio.db");

    std::fs::create_dir_all(db_path).ok();

    let db = Surreal::new::<RocksDb>(db_path)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    db.use_ns(&CONFIG.database_ns)
        .use_db(&CONFIG.database_db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    DB.set(db)
        .map_err(|_| AppError::DatabaseError("Database already initialized".to_string()))?;

    run_migrations().await?;
    seed_data().await?;

    info!("Database initialized successfully");
    Ok(())
}

pub fn get_db() -> &'static Surreal<Db> {
    DB.get().expect("Database not initialized")
}

async fn run_migrations() -> Result<(), AppError> {
    let db = get_db();

    db.query(
        r#"
        DEFINE TABLE IF NOT EXISTS projects SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS title ON projects TYPE string;
        DEFINE FIELD IF NOT EXISTS slug ON projects TYPE string;
        DEFINE FIELD IF NOT EXISTS description ON projects TYPE string;
        DEFINE FIELD IF NOT EXISTS content ON projects TYPE string;
        DEFINE FIELD IF NOT EXISTS category ON projects TYPE string;
        DEFINE FIELD IF NOT EXISTS tech_stack ON projects TYPE array<string>;
        DEFINE FIELD IF NOT EXISTS github_url ON projects TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS live_url ON projects TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS image_url ON projects TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS featured ON projects TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS status ON projects TYPE string DEFAULT 'completed';
        DEFINE FIELD IF NOT EXISTS created_at ON projects TYPE datetime DEFAULT time::now();
        DEFINE FIELD IF NOT EXISTS updated_at ON projects TYPE datetime DEFAULT time::now();
        DEFINE INDEX IF NOT EXISTS projects_slug_idx ON projects FIELDS slug UNIQUE;

        DEFINE TABLE IF NOT EXISTS posts SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS title ON posts TYPE string;
        DEFINE FIELD IF NOT EXISTS slug ON posts TYPE string;
        DEFINE FIELD IF NOT EXISTS excerpt ON posts TYPE string;
        DEFINE FIELD IF NOT EXISTS content ON posts TYPE string;
        DEFINE FIELD IF NOT EXISTS tags ON posts TYPE array<string>;
        DEFINE FIELD IF NOT EXISTS published ON posts TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS reading_time ON posts TYPE int;
        DEFINE FIELD IF NOT EXISTS created_at ON posts TYPE datetime DEFAULT time::now();
        DEFINE FIELD IF NOT EXISTS updated_at ON posts TYPE datetime DEFAULT time::now();
        DEFINE INDEX IF NOT EXISTS posts_slug_idx ON posts FIELDS slug UNIQUE;

        DEFINE TABLE IF NOT EXISTS experiences SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS company ON experiences TYPE string;
        DEFINE FIELD IF NOT EXISTS role ON experiences TYPE string;
        DEFINE FIELD IF NOT EXISTS description ON experiences TYPE string;
        DEFINE FIELD IF NOT EXISTS start_date ON experiences TYPE string;
        DEFINE FIELD IF NOT EXISTS end_date ON experiences TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS current ON experiences TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS order_index ON experiences TYPE int;

        DEFINE TABLE IF NOT EXISTS skills SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS name ON skills TYPE string;
        DEFINE FIELD IF NOT EXISTS category ON skills TYPE string;
        DEFINE FIELD IF NOT EXISTS icon ON skills TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS proficiency ON skills TYPE int;

        DEFINE TABLE IF NOT EXISTS messages SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS name ON messages TYPE string;
        DEFINE FIELD IF NOT EXISTS email ON messages TYPE string;
        DEFINE FIELD IF NOT EXISTS subject ON messages TYPE string;
        DEFINE FIELD IF NOT EXISTS message ON messages TYPE string;
        DEFINE FIELD IF NOT EXISTS read ON messages TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS created_at ON messages TYPE datetime DEFAULT time::now();

        DEFINE TABLE IF NOT EXISTS sessions SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS github_username ON sessions TYPE string;
        DEFINE FIELD IF NOT EXISTS github_id ON sessions TYPE string;
        DEFINE FIELD IF NOT EXISTS avatar_url ON sessions TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS expires_at ON sessions TYPE datetime;
        "#,
    )
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    info!("Database migrations completed");
    Ok(())
}

async fn seed_data() -> Result<(), AppError> {
    let db = get_db();

    let existing: Vec<Project> = db.select("projects").await.unwrap_or_default();
    if !existing.is_empty() {
        info!("Database already seeded, skipping");
        return Ok(());
    }

    info!("Seeding database with empty content - add via admin dashboard");

    // Database starts empty - use admin dashboard to add content
    // Projects, posts, skills, and experiences can be managed via /admin

    info!("Database seeded successfully");
    Ok(())
}
