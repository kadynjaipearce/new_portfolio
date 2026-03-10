use crate::config::CONFIG;
use crate::error::{AppError, AppResult};
use actix_session::Session;
use actix_web::{http::header, web, HttpResponse};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
};
use serde::Deserialize;

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/github", web::get().to(github_login))
            .route("/github/callback", web::get().to(github_callback))
            .route("/logout", web::get().to(logout)),
    );
}

fn get_oauth_client() -> Result<BasicClient, AppError> {
    let client_id = ClientId::new(CONFIG.github_client_id.clone());
    let client_secret = ClientSecret::new(CONFIG.github_client_secret.clone());
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let redirect_url = RedirectUrl::new(CONFIG.github_redirect_uri.clone())
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(
        BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
            .set_redirect_uri(redirect_url),
    )
}

async fn github_login(session: Session) -> AppResult<HttpResponse> {
    let client = get_oauth_client()?;

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    session
        .insert("oauth_state", csrf_token.secret())
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, auth_url.to_string()))
        .finish())
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GitHubUser {
    login: String,
    id: i64,
    avatar_url: String,
}

async fn github_callback(
    session: Session,
    query: web::Query<CallbackQuery>,
) -> AppResult<HttpResponse> {
    // Verify CSRF state
    let stored_state: Option<String> = session
        .get("oauth_state")
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    match stored_state {
        Some(state) if state == query.state => {}
        _ => return Err(AppError::Unauthorized("Invalid OAuth state".to_string())),
    }

    session.remove("oauth_state");

    // Exchange code for token using simple HTTP request
    let http_client = reqwest::Client::new();

    let token_response = http_client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", CONFIG.github_client_id.as_str()),
            ("client_secret", CONFIG.github_client_secret.as_str()),
            ("code", query.code.as_str()),
            ("redirect_uri", CONFIG.github_redirect_uri.as_str()),
        ])
        .send()
        .await?
        .json::<TokenResponseJson>()
        .await
        .map_err(|e| AppError::InternalError(format!("Token exchange failed: {}", e)))?;

    let access_token = token_response.access_token;

    // Fetch user info from GitHub
    let user: GitHubUser = http_client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "portfolio-app")
        .send()
        .await?
        .json()
        .await?;

    // Check if user is the admin
    if user.login.to_lowercase() != CONFIG.admin_github_username.to_lowercase() {
        return Err(AppError::Unauthorized(
            "You are not authorized to access the admin panel".to_string(),
        ));
    }

    // Store session
    session
        .insert("user_id", user.id.to_string())
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    session
        .insert("github_username", &user.login)
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    session
        .insert("avatar_url", &user.avatar_url)
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    session
        .insert("authenticated", true)
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, "/admin"))
        .finish())
}

#[derive(Deserialize)]
struct TokenResponseJson {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    scope: String,
}

async fn logout(session: Session) -> HttpResponse {
    session.purge();
    HttpResponse::Found()
        .insert_header((header::LOCATION, "/"))
        .finish()
}
