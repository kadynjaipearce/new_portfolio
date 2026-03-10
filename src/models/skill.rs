use crate::db::get_db;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: Option<Thing>,
    pub name: String,
    pub category: String,
    pub icon: Option<String>,
    pub proficiency: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateSkill {
    pub name: String,
    pub category: String,
    pub icon: Option<String>,
    pub proficiency: i32,
}

impl Skill {
    pub async fn all() -> AppResult<Vec<Self>> {
        let db = get_db();
        let skills: Vec<Self> = db
            .query("SELECT * FROM skills ORDER BY category, proficiency DESC")
            .await?
            .take(0)?;
        Ok(skills)
    }

    pub async fn by_category(category: &str) -> AppResult<Vec<Self>> {
        let db = get_db();
        let skills: Vec<Self> = db
            .query("SELECT * FROM skills WHERE category = $category ORDER BY proficiency DESC")
            .bind(("category", category.to_string()))
            .await?
            .take(0)?;
        Ok(skills)
    }

    pub async fn grouped() -> AppResult<std::collections::HashMap<String, Vec<Self>>> {
        let all = Self::all().await?;
        let mut grouped: std::collections::HashMap<String, Vec<Self>> =
            std::collections::HashMap::new();

        for skill in all {
            grouped
                .entry(skill.category.clone())
                .or_default()
                .push(skill);
        }

        Ok(grouped)
    }

    pub async fn by_id(id: &str) -> AppResult<Option<Self>> {
        let db = get_db();
        let skill: Option<Self> = db.select(("skills", id)).await?;
        Ok(skill)
    }

    pub async fn create(data: CreateSkill) -> AppResult<Self> {
        let db = get_db();

        let skill = Skill {
            id: None,
            name: data.name,
            category: data.category,
            icon: data.icon,
            proficiency: data.proficiency,
        };

        let created: Option<Self> = db.create("skills").content(skill).await?;
        created.ok_or_else(|| AppError::DatabaseError("Failed to create skill".to_string()))
    }

    pub async fn delete(id: &str) -> AppResult<()> {
        let db = get_db();
        let _: Option<Self> = db.delete(("skills", id)).await?;
        Ok(())
    }
}
