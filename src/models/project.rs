use crate::db::get_db;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<Thing>,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub content: String,
    pub category: String,
    pub tech_stack: Vec<String>,
    pub github_url: Option<String>,
    pub live_url: Option<String>,
    pub image_url: Option<String>,
    pub featured: bool,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub title: String,
    pub description: String,
    pub content: String,
    pub category: String,
    pub tech_stack: Vec<String>,
    pub github_url: Option<String>,
    pub live_url: Option<String>,
    pub image_url: Option<String>,
    pub featured: bool,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tech_stack: Option<Vec<String>>,
    pub github_url: Option<String>,
    pub live_url: Option<String>,
    pub image_url: Option<String>,
    pub featured: Option<bool>,
    pub status: Option<String>,
}

impl Project {
    pub async fn all() -> AppResult<Vec<Self>> {
        let db = get_db();
        let projects: Vec<Self> = db
            .query("SELECT * FROM projects ORDER BY created_at DESC")
            .await?
            .take(0)?;
        Ok(projects)
    }

    pub async fn featured() -> AppResult<Vec<Self>> {
        let db = get_db();
        let projects: Vec<Self> = db
            .query("SELECT * FROM projects WHERE featured = true ORDER BY created_at DESC LIMIT 3")
            .await?
            .take(0)?;
        Ok(projects)
    }

    pub async fn by_category(category: &str) -> AppResult<Vec<Self>> {
        let db = get_db();
        let projects: Vec<Self> = db
            .query("SELECT * FROM projects WHERE category = $category ORDER BY created_at DESC")
            .bind(("category", category.to_string()))
            .await?
            .take(0)?;
        Ok(projects)
    }

    pub async fn by_slug(slug: &str) -> AppResult<Option<Self>> {
        let db = get_db();
        let projects: Vec<Self> = db
            .query("SELECT * FROM projects WHERE slug = $slug LIMIT 1")
            .bind(("slug", slug.to_string()))
            .await?
            .take(0)?;
        Ok(projects.into_iter().next())
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let db = get_db();
        let project: Option<Self> = db.select(("projects", id)).await?;
        Ok(project)
    }

    pub async fn create(data: CreateProject) -> AppResult<Self> {
        let db = get_db();
        let slug = slug::slugify(&data.title);

        let project = Project {
            id: None,
            title: data.title,
            slug,
            description: data.description,
            content: data.content,
            category: data.category,
            tech_stack: data.tech_stack,
            github_url: data.github_url,
            live_url: data.live_url,
            image_url: data.image_url,
            featured: data.featured,
            status: data.status,
            created_at: None,
            updated_at: None,
        };

        let created: Option<Self> = db.create("projects").content(project).await?;
        created.ok_or_else(|| AppError::DatabaseError("Failed to create project".to_string()))
    }

    pub async fn update(id: &str, data: UpdateProject) -> AppResult<Self> {
        let db = get_db();

        let mut updates = vec!["updated_at = time::now()".to_string()];

        if let Some(title) = &data.title {
            updates.push(format!("title = '{}'", title.replace('\'', "''")));
            updates.push(format!("slug = '{}'", slug::slugify(title)));
        }
        if let Some(desc) = &data.description {
            updates.push(format!("description = '{}'", desc.replace('\'', "''")));
        }
        if let Some(content) = &data.content {
            updates.push(format!("content = '{}'", content.replace('\'', "''")));
        }
        if let Some(cat) = &data.category {
            updates.push(format!("category = '{}'", cat));
        }
        if let Some(featured) = data.featured {
            updates.push(format!("featured = {}", featured));
        }
        if let Some(status) = &data.status {
            updates.push(format!("status = '{}'", status));
        }

        let query = format!(
            "UPDATE projects:{} SET {} RETURN AFTER",
            id,
            updates.join(", ")
        );

        let result: Vec<Self> = db.query(&query).await?.take(0)?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let db = get_db();
        let _: Option<Self> = db.delete(("projects", id)).await?;
        Ok(())
    }

    pub async fn count() -> AppResult<i64> {
        let db = get_db();
        let result: Vec<CountResult> = db
            .query("SELECT count() as count FROM projects GROUP ALL")
            .await?
            .take(0)?;
        Ok(result.first().map(|r| r.count).unwrap_or(0))
    }
}

#[derive(Debug, Deserialize)]
struct CountResult {
    count: i64,
}
