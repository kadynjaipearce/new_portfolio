use crate::error::AppResult;
use crate::models::{Post, Project, Skill};
use crate::services::github::GitHubService;
use actix_web::{web, HttpResponse};
use serde::Serialize;

pub fn api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/stats", web::get().to(get_stats))
            .route("/github/stats", web::get().to(get_github_stats))
            .route("/projects", web::get().to(get_projects))
            .route("/posts", web::get().to(get_posts))
            .route("/skills", web::get().to(get_skills)),
    );
}

#[derive(Serialize)]
struct Stats {
    projects: i64,
    posts: i64,
    years_experience: i32,
}

async fn get_stats() -> AppResult<HttpResponse> {
    let stats = Stats {
        projects: Project::count().await.unwrap_or(0),
        posts: Post::count_published().await.unwrap_or(0),
        years_experience: 5,
    };

    Ok(HttpResponse::Ok().json(stats))
}

async fn get_github_stats() -> AppResult<HttpResponse> {
    let stats = GitHubService::get_user_stats("kadynjaipearce").await?;
    Ok(HttpResponse::Ok().json(stats))
}

async fn get_projects() -> AppResult<HttpResponse> {
    let projects = Project::all().await?;
    Ok(HttpResponse::Ok().json(projects))
}

async fn get_posts() -> AppResult<HttpResponse> {
    let posts = Post::published().await?;
    Ok(HttpResponse::Ok().json(posts))
}

async fn get_skills() -> AppResult<HttpResponse> {
    let skills = Skill::grouped().await?;
    Ok(HttpResponse::Ok().json(skills))
}
