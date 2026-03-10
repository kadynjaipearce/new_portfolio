use crate::db::get_db;
use crate::error::{AppError, AppResult};
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Option<Thing>,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub reading_time: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub excerpt: String,
    pub content: String,
    pub tags: Vec<String>,
    pub published: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub published: Option<bool>,
}

impl Post {
    pub fn content_html(&self) -> String {
        let parser = Parser::new(&self.content);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }

    fn calculate_reading_time(content: &str) -> i32 {
        let word_count = content.split_whitespace().count();
        ((word_count as f64 / 200.0).ceil() as i32).max(1)
    }

    pub async fn all() -> AppResult<Vec<Self>> {
        let db = get_db();
        let posts: Vec<Self> = db
            .query("SELECT * FROM posts ORDER BY created_at DESC")
            .await?
            .take(0)?;
        Ok(posts)
    }

    pub async fn published() -> AppResult<Vec<Self>> {
        let db = get_db();
        let posts: Vec<Self> = db
            .query("SELECT * FROM posts WHERE published = true ORDER BY created_at DESC")
            .await?
            .take(0)?;
        Ok(posts)
    }

    pub async fn by_tag(tag: &str) -> AppResult<Vec<Self>> {
        let db = get_db();
        let posts: Vec<Self> = db
            .query("SELECT * FROM posts WHERE $tag IN tags AND published = true ORDER BY created_at DESC")
            .bind(("tag", tag.to_string()))
            .await?
            .take(0)?;
        Ok(posts)
    }

    pub async fn by_slug(slug: &str) -> AppResult<Option<Self>> {
        let db = get_db();
        let posts: Vec<Self> = db
            .query("SELECT * FROM posts WHERE slug = $slug LIMIT 1")
            .bind(("slug", slug.to_string()))
            .await?
            .take(0)?;
        Ok(posts.into_iter().next())
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let db = get_db();
        let post: Option<Self> = db.select(("posts", id)).await?;
        Ok(post)
    }

    pub async fn recent(limit: usize) -> AppResult<Vec<Self>> {
        let db = get_db();
        let posts: Vec<Self> = db
            .query(
                "SELECT * FROM posts WHERE published = true ORDER BY created_at DESC LIMIT $limit",
            )
            .bind(("limit", limit as i64))
            .await?
            .take(0)?;
        Ok(posts)
    }

    pub async fn create(data: CreatePost) -> AppResult<Self> {
        let db = get_db();
        let slug = slug::slugify(&data.title);
        let reading_time = Self::calculate_reading_time(&data.content);

        let post = Post {
            id: None,
            title: data.title,
            slug,
            excerpt: data.excerpt,
            content: data.content,
            tags: data.tags,
            published: data.published,
            reading_time,
            created_at: None,
            updated_at: None,
        };

        let created: Option<Self> = db.create("posts").content(post).await?;
        created.ok_or_else(|| AppError::DatabaseError("Failed to create post".to_string()))
    }

    pub async fn update(id: &str, data: UpdatePost) -> AppResult<Self> {
        let db = get_db();

        let mut updates = vec!["updated_at = time::now()".to_string()];

        if let Some(title) = &data.title {
            updates.push(format!("title = '{}'", title.replace('\'', "''")));
            updates.push(format!("slug = '{}'", slug::slugify(title)));
        }
        if let Some(excerpt) = &data.excerpt {
            updates.push(format!("excerpt = '{}'", excerpt.replace('\'', "''")));
        }
        if let Some(content) = &data.content {
            updates.push(format!("content = '{}'", content.replace('\'', "''")));
            updates.push(format!(
                "reading_time = {}",
                Self::calculate_reading_time(content)
            ));
        }
        if let Some(published) = data.published {
            updates.push(format!("published = {}", published));
        }

        let query = format!(
            "UPDATE posts:{} SET {} RETURN AFTER",
            id,
            updates.join(", ")
        );

        let result: Vec<Self> = db.query(&query).await?.take(0)?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let db = get_db();
        let _: Option<Self> = db.delete(("posts", id)).await?;
        Ok(())
    }

    pub async fn count() -> AppResult<i64> {
        let db = get_db();
        let result: Vec<CountResult> = db
            .query("SELECT count() as count FROM posts GROUP ALL")
            .await?
            .take(0)?;
        Ok(result.first().map(|r| r.count).unwrap_or(0))
    }

    pub async fn count_published() -> AppResult<i64> {
        let db = get_db();
        let result: Vec<CountResult> = db
            .query("SELECT count() as count FROM posts WHERE published = true GROUP ALL")
            .await?
            .take(0)?;
        Ok(result.first().map(|r| r.count).unwrap_or(0))
    }
}

#[derive(Debug, Deserialize)]
struct CountResult {
    count: i64,
}
