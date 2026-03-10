use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubStats {
    pub public_repos: i32,
    pub followers: i32,
    pub following: i32,
    pub total_stars: i32,
    pub avatar_url: String,
    pub bio: Option<String>,
    pub recent_repos: Vec<RepoInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub description: Option<String>,
    pub stars: i32,
    pub forks: i32,
    pub language: Option<String>,
    pub url: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    pub public_repos: i32,
    pub followers: i32,
    pub following: i32,
    pub avatar_url: String,
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    pub name: String,
    pub description: Option<String>,
    pub stargazers_count: i32,
    pub forks_count: i32,
    pub language: Option<String>,
    pub html_url: String,
    pub updated_at: String,
    pub fork: bool,
}

pub struct GitHubService;

impl GitHubService {
    pub async fn get_user_stats(username: &str) -> AppResult<GitHubStats> {
        let client = reqwest::Client::new();

        // Fetch user info
        let user: GitHubUser = client
            .get(format!("https://api.github.com/users/{}", username))
            .header("User-Agent", "portfolio-app")
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to fetch GitHub user: {}", e)))?;

        // Fetch repos
        let repos: Vec<GitHubRepo> = client
            .get(format!(
                "https://api.github.com/users/{}/repos?sort=updated&per_page=10",
                username
            ))
            .header("User-Agent", "portfolio-app")
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to fetch GitHub repos: {}", e)))?;

        // Calculate total stars
        let total_stars: i32 = repos.iter().map(|r| r.stargazers_count).sum();

        // Filter and map repos (exclude forks)
        let recent_repos: Vec<RepoInfo> = repos
            .into_iter()
            .filter(|r| !r.fork)
            .take(5)
            .map(|r| RepoInfo {
                name: r.name,
                description: r.description,
                stars: r.stargazers_count,
                forks: r.forks_count,
                language: r.language,
                url: r.html_url,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(GitHubStats {
            public_repos: user.public_repos,
            followers: user.followers,
            following: user.following,
            total_stars,
            avatar_url: user.avatar_url,
            bio: user.bio,
            recent_repos,
        })
    }
}
