use crate::errors::Error;
use actix_web::{App, HttpResponse, HttpServer, Responder, cookie::Cookie, get, web};
use async_trait::async_trait;

use mockall::predicate::*;
use mockall::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod github;
pub mod redis;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    id: u64,
    name: String,
    avatar_url: String,
    login: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Session {
    user: User,
    token: String,
}

#[automock]
#[async_trait]
pub trait Repo: Send + Sync {
    async fn save(&self, user: User) -> Result<String, Error>;
    async fn get(&self, token: String) -> Result<User, Error>;
    async fn delete(&self, token: String) -> Result<(), Error>;
}

#[automock]
#[async_trait]
pub trait Authenticator: Sync + Send {
    async fn start_login(&self) -> Result<String, Error>;
    async fn login(&self, code: String) -> Result<Session, Error>;
}

#[automock]
#[async_trait]
pub trait SessionManager: Sync + Send {
    async fn session(&self, token: String) -> Result<User, Error>;
    async fn logout(&self, token: String) -> Result<(), Error>;
}

pub struct DefaultSessionManager {
    repo: Arc<dyn Repo>,
}

impl DefaultSessionManager {
    pub fn new(repo: Arc<dyn Repo>) -> Self {
        DefaultSessionManager { repo }
    }
}

#[async_trait]
impl SessionManager for DefaultSessionManager {
    async fn session(&self, token: String) -> Result<User, Error> {
        self.repo.get(token).await
    }

    async fn logout(&self, token: String) -> Result<(), Error> {
        self.repo.delete(token).await
    }
}

#[cfg(test)]
mod default_session_manager_test {
    use super::*;

    #[tokio::test]
    async fn test_session_success() {
        let mut mock_repo = MockRepo::new();
        mock_repo.expect_get().returning(|_| {
            Ok(User {
                id: 123456,
                name: "John Doe".to_string(),
                avatar_url: "https://foo.bar".to_string(),
                login: "john_doe".to_string(),
            })
        });

        let repo = Arc::new(mock_repo);
        let session_manager = DefaultSessionManager::new(repo);

        let token = "test_token".to_string();
        let result = session_manager.session(token).await;
        assert!(result.is_ok());
        let user = result.unwrap();

        assert_eq!(user.id, 123456);
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.avatar_url, "https://foo.bar");
        assert_eq!(user.login, "john_doe");
    }

    #[tokio::test]
    async fn test_session_not_found() {
        let mut mock_repo = MockRepo::new();
        mock_repo.expect_get().returning(|_| Err(Error::NotFound));

        let repo = Arc::new(mock_repo);
        let session_manager = DefaultSessionManager::new(repo);

        let token = "invalid_token".to_string();
        let result = session_manager.session(token).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "session not found");
    }

    #[tokio::test]
    async fn test_logout_success() {
        let mut mock_repo = MockRepo::new();
        mock_repo.expect_delete().returning(|_| Ok(()));

        let repo = Arc::new(mock_repo);
        let session_manager = DefaultSessionManager::new(repo);

        let token = "test_token".to_string();
        let result = session_manager.logout(token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logout_failure() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_delete()
            .returning(|_| Err(Error::ConnectionError("Failed to delete".to_string())));

        let repo = Arc::new(mock_repo);
        let session_manager = DefaultSessionManager::new(repo);

        let token = "invalid_token".to_string();
        let result = session_manager.logout(token).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "connection error: Failed to delete"
        );
    }
}

struct HttpServerState {
    sessions: Arc<dyn SessionManager>,
    auth: Arc<dyn Authenticator>,
    base_url: String,
    cookie_name: String,
}

#[get("/auth/login")]
async fn login(state: web::Data<HttpServerState>) -> impl Responder {
    match state.auth.start_login().await {
        Err(err) => err.to_http_response(),
        Ok(url) => HttpResponse::Found()
            .append_header(("Location", url))
            .finish(),
    }
}

#[derive(Deserialize)]
struct LoginCallback {
    code: String,
}

#[get("/auth/login/callback")]
async fn login_callback(
    state: web::Data<HttpServerState>,
    query: web::Query<LoginCallback>,
) -> impl Responder {
    match state.auth.login(query.code.clone()).await {
        Err(err) => err.to_http_response(),
        Ok(session) => HttpResponse::Ok()
            .cookie(
                Cookie::build(state.cookie_name.clone(), session.token)
                    .domain(state.base_url.clone())
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .finish(),
            )
            .append_header(("Location", state.base_url.clone()))
            .finish(),
    }
}

#[get("/auth/logout")]
async fn logout(state: web::Data<HttpServerState>, req: actix_web::HttpRequest) -> impl Responder {
    if let Some(cookie) = req.cookie("sid") {
        match state.sessions.logout(cookie.value().to_string()).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => err.to_http_response(),
        }
    } else {
        HttpResponse::Unauthorized().body("No session ID found in cookies")
    }
}

#[get("/api/auth/me")]
async fn me(state: web::Data<HttpServerState>, req: actix_web::HttpRequest) -> impl Responder {
    if let Some(cookie) = req.cookie("sid") {
        match state.sessions.session(cookie.value().to_string()).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(err) => err.to_http_response(),
        }
    } else {
        HttpResponse::Unauthorized().body("No session ID found in cookies")
    }
}

pub async fn http_server(
    sessions: Arc<dyn SessionManager>,
    auth: Arc<dyn Authenticator>,
    base_url: String,
    cookie_name: String,
    listen_addr: String,
) -> Result<(), std::io::Error> {
    let data = web::Data::new(HttpServerState {
        sessions,
        auth,
        base_url,
        cookie_name,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(login)
            .service(login_callback)
            .service(logout)
            .service(me)
    })
    .bind(listen_addr)?
    .run()
    .await
}
