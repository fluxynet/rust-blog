use crate::errors::Error;
use async_trait::async_trait;

use mockall::predicate::*;
use mockall::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod github;
pub mod http;
pub mod redis;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub avatar_url: String,
    pub login: String,
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
        mock_repo
            .expect_get()
            .returning(|_| Err(Error::NotFound("session not found".to_string())));

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
