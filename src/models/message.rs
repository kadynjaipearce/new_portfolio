use crate::db::get_db;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<Thing>,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub read: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

impl Message {
    pub async fn all() -> AppResult<Vec<Self>> {
        let db = get_db();
        let messages: Vec<Self> = db
            .query("SELECT * FROM messages ORDER BY created_at DESC")
            .await?
            .take(0)?;
        Ok(messages)
    }

    pub async fn unread() -> AppResult<Vec<Self>> {
        let db = get_db();
        let messages: Vec<Self> = db
            .query("SELECT * FROM messages WHERE read = false ORDER BY created_at DESC")
            .await?
            .take(0)?;
        Ok(messages)
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let db = get_db();
        let msg: Option<Self> = db.select(("messages", id)).await?;
        Ok(msg)
    }

    pub async fn create(data: CreateMessage) -> AppResult<Self> {
        let db = get_db();

        let msg = Message {
            id: None,
            name: data.name,
            email: data.email,
            subject: data.subject,
            message: data.message,
            read: false,
            created_at: None,
        };

        let created: Option<Self> = db.create("messages").content(msg).await?;
        created.ok_or_else(|| AppError::DatabaseError("Failed to create message".to_string()))
    }

    pub async fn mark_read(id: &str) -> AppResult<Self> {
        let db = get_db();
        let query = format!("UPDATE messages:{} SET read = true RETURN AFTER", id);
        let result: Vec<Self> = db.query(&query).await?.take(0)?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("Message not found".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let db = get_db();
        let _: Option<Self> = db.delete(("messages", id)).await?;
        Ok(())
    }

    pub async fn count_unread() -> AppResult<i64> {
        let db = get_db();
        let result: Vec<CountResult> = db
            .query("SELECT count() as count FROM messages WHERE read = false GROUP ALL")
            .await?
            .take(0)?;
        Ok(result.first().map(|r| r.count).unwrap_or(0))
    }
}

#[derive(Debug, Deserialize)]
struct CountResult {
    count: i64,
}
