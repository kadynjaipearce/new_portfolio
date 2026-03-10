use crate::db::get_db;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: Option<Thing>,
    pub company: String,
    pub role: String,
    pub description: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub current: bool,
    pub order_index: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateExperience {
    pub company: String,
    pub role: String,
    pub description: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub current: bool,
    pub order_index: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExperience {
    pub company: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub current: Option<bool>,
    pub order_index: Option<i32>,
}

impl Experience {
    pub async fn all() -> AppResult<Vec<Self>> {
        let db = get_db();
        let experiences: Vec<Self> = db
            .query("SELECT * FROM experiences ORDER BY order_index ASC")
            .await?
            .take(0)?;
        Ok(experiences)
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let db = get_db();
        let exp: Option<Self> = db.select(("experiences", id)).await?;
        Ok(exp)
    }

    pub async fn create(data: CreateExperience) -> AppResult<Self> {
        let db = get_db();

        let exp = Experience {
            id: None,
            company: data.company,
            role: data.role,
            description: data.description,
            start_date: data.start_date,
            end_date: data.end_date,
            current: data.current,
            order_index: data.order_index,
        };

        let created: Option<Self> = db.create("experiences").content(exp).await?;
        created.ok_or_else(|| AppError::DatabaseError("Failed to create experience".to_string()))
    }

    pub async fn update(id: &str, data: UpdateExperience) -> AppResult<Self> {
        let db = get_db();

        let mut updates = Vec::new();

        if let Some(company) = &data.company {
            updates.push(format!("company = '{}'", company.replace('\'', "''")));
        }
        if let Some(role) = &data.role {
            updates.push(format!("role = '{}'", role.replace('\'', "''")));
        }
        if let Some(desc) = &data.description {
            updates.push(format!("description = '{}'", desc.replace('\'', "''")));
        }
        if let Some(start) = &data.start_date {
            updates.push(format!("start_date = '{}'", start));
        }
        if let Some(end) = &data.end_date {
            updates.push(format!("end_date = '{}'", end));
        }
        if let Some(current) = data.current {
            updates.push(format!("current = {}", current));
        }
        if let Some(idx) = data.order_index {
            updates.push(format!("order_index = {}", idx));
        }

        if updates.is_empty() {
            return Self::by_id(id)
                .await?
                .ok_or_else(|| AppError::NotFound("Experience not found".to_string()));
        }

        let query = format!(
            "UPDATE experiences:{} SET {} RETURN AFTER",
            id,
            updates.join(", ")
        );

        let result: Vec<Self> = db.query(&query).await?.take(0)?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("Experience not found".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let db = get_db();
        let _: Option<Self> = db.delete(("experiences", id)).await?;
        Ok(())
    }
}
