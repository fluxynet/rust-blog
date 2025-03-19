use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use bb8_redis::redis::AsyncCommands;
use mockall::predicate::*;
use mockall::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("initialization error: {0}")]
    InitializationError(String),

    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("session not found")]
    NotFound,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    id: u64,
    name: String,
    avatar_url: String,
    login: String,
}

impl User {
    fn from_github(gh: GithubUser) -> Self {
        User {
            id: gh.id,
            name: gh.name,
            avatar_url: gh.avatar_url,
            login: gh.login,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Session {
    user: User,
    token: String,
}

#[automock]
#[async_trait]
pub trait Repo: Send + Sync {
    async fn save(&self, user: User) -> Result<String, AuthError>;
    async fn get(&self, token: String) -> Result<User, AuthError>;
    async fn delete(&self, token: String) -> Result<(), AuthError>;
}

pub struct RedisRepo {
    pool: Pool<RedisConnectionManager>,
}

impl RedisRepo {
    pub async fn new(redis_url: &str) -> Result<Self, AuthError> {
        let manager = match RedisConnectionManager::new(redis_url) {
            Ok(m) => m,
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        };

        match Pool::builder().build(manager).await {
            Ok(pool) => return Ok(RedisRepo { pool }),
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        };
    }
}

#[async_trait]
impl Repo for RedisRepo {
    async fn save(&self, user: User) -> Result<String, AuthError> {
        let mut con = match self.pool.get().await {
            Ok(con) => con,
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        };

        let token = Uuid::new_v4().to_string();

        let data = format!(
            "{}|{}|{}|{}",
            user.id, user.login, user.avatar_url, user.name
        );

        match con.set::<&str, String, ()>(token.as_str(), data).await {
            Ok(_) => return Ok(token),
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        }
    }

    async fn get(&self, token: String) -> Result<User, AuthError> {
        let mut con = match self.pool.get().await {
            Ok(con) => con,
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        };

        let data: String = match con.get(token).await {
            Ok(v) => v,
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        };

        let segments: Vec<&str> = data.splitn(4, '|').collect();
        if segments.len() != 4 {
            return Err(AuthError::SerializationError(
                "Invalid data format".to_string(),
            ));
        }

        let user = User {
            id: segments[0]
                .parse::<u64>()
                .map_err(|_| AuthError::SerializationError("Invalid ID format".to_string()))?,
            login: segments[1].to_string(),
            avatar_url: segments[2].to_string(),
            name: segments[3].to_string(),
        };

        Ok(user)
    }

    async fn delete(&self, token: String) -> Result<(), AuthError> {
        let mut con = match self.pool.get().await {
            Ok(con) => con,
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        };

        match con.del::<&str, ()>(token.as_str()).await {
            Ok(_) => Ok(()),
            Err(err) => Err(AuthError::ConnectionError(err.to_string())),
        }
    }
}

#[automock]
#[async_trait]
pub trait Authenticator {
    async fn start_login(&self) -> Result<String, AuthError>;
    async fn login(&self, code: String) -> Result<Session, AuthError>;
}

pub struct GithubAuthenticator {
    org: String,
    client_id: String,
    client_secret: String,
    url: String,
    api_url: String,
    repo: Arc<dyn Repo>,
}

impl GithubAuthenticator {
    pub fn new(
        repo: Arc<dyn Repo>,
        client_id: String,
        client_secret: String,
        org: String,
    ) -> Result<Self, AuthError> {
        let url = "https://github.com".to_string();
        let api_url = "https://api.github.com".to_string();

        if client_id.is_empty() {
            return Err(AuthError::InitializationError(
                "Client ID is empty".to_string(),
            ));
        }

        if client_secret.is_empty() {
            return Err(AuthError::InitializationError(
                "Client Secret is empty".to_string(),
            ));
        }

        if org.is_empty() {
            return Err(AuthError::InitializationError(
                "Organization is empty".to_string(),
            ));
        }

        Ok(GithubAuthenticator {
            org,
            client_id,
            client_secret,
            url,
            api_url,
            repo,
        })
    }

    #[cfg(test)]
    async fn new_test(
        repo: Arc<dyn Repo>,
        client_id: String,
        client_secret: String,
        org: String,
    ) -> Result<(mockito::ServerGuard, Self), AuthError> {
        let mut auth = GithubAuthenticator::new(repo, client_id, client_secret, org)?;

        let server = mockito::Server::new_async().await;

        let url = server.url();

        auth.url = url.clone();
        auth.api_url = url.clone();

        Ok((server, auth))
    }
}

#[derive(Deserialize)]
struct GithubUser {
    login: String,
    id: u64,
    name: String,
    avatar_url: String,
}

#[derive(Deserialize)]
struct GithubOrgs(Vec<GithubOrg>);
#[derive(Deserialize)]
struct GithubOrg {
    login: String,
}

#[async_trait]
impl Authenticator for GithubAuthenticator {
    async fn start_login(&self) -> Result<String, AuthError> {
        let url = format!(
            "{}/login/oauth/authorize?client_id={}&scope=read:user,read:org",
            self.url, self.client_id
        );

        Ok(url)
    }

    async fn login(&self, code: String) -> Result<Session, AuthError> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("code", &code),
        ];

        let res = match client
            .post(format!("{}/login/oauth/access_token", self.url))
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
        {
            Ok(r) => r,
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        };

        if res.status() != 200 {
            return Err(AuthError::ConnectionError(
                format!("GitHub returned status code: {}", res.status()).to_string(),
            ));
        }

        let res_json: serde_json::Value = res
            .json()
            .await
            .map_err(|err| AuthError::SerializationError(err.to_string()))?;

        let access_token = res_json["access_token"]
            .as_str()
            .ok_or_else(|| AuthError::SerializationError("obtaining access token".to_string()))?;

        let gh_user: GithubUser = client
            .get(format!("{}/user", self.api_url))
            .header("Authorization", format!("token {}", access_token))
            .send()
            .await
            .map_err(|err| AuthError::ConnectionError(err.to_string()))?
            .json()
            .await
            .map_err(|err| AuthError::SerializationError(err.to_string()))?;

        let gh_orgs: GithubOrgs = client
            .get(format!("{}/user/orgs", self.api_url))
            .header("Authorization", format!("token {}", access_token))
            .send()
            .await
            .map_err(|err| AuthError::ConnectionError(err.to_string()))?
            .json()
            .await
            .map_err(|err| AuthError::SerializationError(err.to_string()))?;

        if !gh_orgs.0.iter().any(|org| org.login == self.org) {
            return Err(AuthError::PermissionDenied(
                format!("not a member of {}", self.org).to_string(),
            ));
        }

        let user = User::from_github(gh_user);

        let token = match self.repo.save(user.clone()).await {
            Ok(token) => token,
            Err(err) => return Err(AuthError::ConnectionError(err.to_string())),
        };

        Ok(Session { user, token })
    }
}

#[cfg(test)]
mod github_authenticator_test {
    use super::*;

    #[tokio::test]
    async fn test_start_login() {
        let repo = Arc::new(MockRepo::new());
        let authenticator = GithubAuthenticator::new(
            repo,
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_org".to_string(),
        )
        .unwrap();

        let result = authenticator.start_login().await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "https://github.com/login/oauth/authorize?client_id=test_client_id&scope=read:user,read:org"
        );
    }

    #[tokio::test]
    async fn test_login_success() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_save()
            .returning(|_| Ok("test_token".to_string()));

        let repo = Arc::new(mock_repo);
        let (mut server, authenticator) = GithubAuthenticator::new_test(
            repo,
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_org".to_string(),
        )
        .await
        .unwrap();

        let m_token = server
            .mock("POST", "/login/oauth/access_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"access_token": "test_access_token"}"#)
            .create();

        let m_user = server
            .mock("GET", "/user")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 123456, "login": "test_user", "avatar_url": "https://foo.bar", "name": "John Doe"}"#)
            .create();

        let m_orgs = server
            .mock("GET", "/user/orgs")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"login": "test_org"}]"#)
            .create();

        let code = "test_code".to_string();
        let result = authenticator.login(code).await;
        assert!(result.is_ok());
        let session = result.unwrap();

        assert_eq!(session.token, "test_token");
        assert_eq!(session.user.id, 123456);
        assert_eq!(session.user.login, "test_user");
        assert_eq!(session.user.avatar_url, "https://foo.bar");
        assert_eq!(session.user.name, "John Doe");

        m_token.assert_async().await;
        m_user.assert_async().await;
        m_orgs.assert_async().await;
    }

    #[tokio::test]
    async fn test_login_invalid_code() {
        let mock_repo = Arc::new(MockRepo::new());
        let (mut server, authenticator) = GithubAuthenticator::new_test(
            mock_repo,
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_org".to_string(),
        )
        .await
        .unwrap();

        let m_token = server
            .mock("POST", "/login/oauth/access_token")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "bad_verification_code"}"#)
            .create();

        let code = "invalid_code".to_string();
        let result = authenticator.login(code).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "connection error: GitHub returned status code: 400 Bad Request"
        );

        m_token.assert_async().await;
    }

    #[tokio::test]
    async fn test_login_not_member_of_org() {
        let mut mock_repo = MockRepo::new();
        mock_repo
            .expect_save()
            .returning(|_| Ok("test_token".to_string()));

        let repo = Arc::new(mock_repo);
        let (mut server, authenticator) = GithubAuthenticator::new_test(
            repo,
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_org".to_string(),
        )
        .await
        .unwrap();

        let m_token = server
            .mock("POST", "/login/oauth/access_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"access_token": "test_access_token"}"#)
            .create();

        let m_user = server
            .mock("GET", "/user")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 123456, "login": "test_user", "avatar_url": "https://foo.bar", "name": "John Doe"}"#)
            .create();

        let m_orgs = server
            .mock("GET", "/user/orgs")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"login": "test_org2"}]"#)
            .create();

        let code = "test_code".to_string();
        let result = authenticator.login(code).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "permission denied: not a member of test_org"
        );

        m_token.assert_async().await;
        m_user.assert_async().await;
        m_orgs.assert_async().await;
    }
}

#[automock]
#[async_trait]
pub trait SessionManager {
    async fn session(&self, token: String) -> Result<User, AuthError>;
    async fn logout(&self, token: String) -> Result<(), AuthError>;
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
    async fn session(&self, token: String) -> Result<User, AuthError> {
        self.repo.get(token).await
    }

    async fn logout(&self, token: String) -> Result<(), AuthError> {
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
            .returning(|_| Err(AuthError::NotFound));

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
            .returning(|_| Err(AuthError::ConnectionError("Failed to delete".to_string())));

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
