use actix_session::SessionExt;
use actix_web::{dev::Payload, http::header, FromRequest, HttpRequest, HttpResponse};
use serde::Serialize;
use std::future::{ready, Ready};

#[derive(Debug, Clone, Serialize)]
pub struct AdminUser {
    pub user_id: String,
    pub github_username: String,
    pub avatar_url: String,
}

impl FromRequest for AdminUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let session = req.get_session();

        let authenticated: bool = session
            .get("authenticated")
            .unwrap_or(None)
            .unwrap_or(false);

        if !authenticated {
            return ready(Err(actix_web::error::InternalError::from_response(
                "Unauthorized",
                HttpResponse::Found()
                    .insert_header((header::LOCATION, "/auth/github"))
                    .finish(),
            )
            .into()));
        }

        let user_id = session
            .get::<String>("user_id")
            .unwrap_or(None)
            .unwrap_or_default();
        let github_username = session
            .get::<String>("github_username")
            .unwrap_or(None)
            .unwrap_or_default();
        let avatar_url = session
            .get::<String>("avatar_url")
            .unwrap_or(None)
            .unwrap_or_default();

        if user_id.is_empty() || github_username.is_empty() {
            return ready(Err(actix_web::error::InternalError::from_response(
                "Unauthorized",
                HttpResponse::Found()
                    .insert_header((header::LOCATION, "/auth/github"))
                    .finish(),
            )
            .into()));
        }

        ready(Ok(AdminUser {
            user_id,
            github_username,
            avatar_url,
        }))
    }
}
