use super::{Authenticator, MockRepo, Repo, Session, User};
use crate::errors::Error;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
pub struct GithubAuthenticator {
    org: String,
    client_id: String,
    client_secret: String,
    url: String,
    api_url: String,
    base_url: String,
    repo: Arc<dyn Repo>,
}

impl GithubAuthenticator {
    pub fn new(
        repo: Arc<dyn Repo>,
        client_id: String,
        client_secret: String,
        org: String,
        base_url: String,
    ) -> Result<Self, Error> {
        let url = "https://github.com".to_string();
        let api_url = "https://api.github.com".to_string();

        if client_id.is_empty() {
            return Err(Error::InitializationError("Client ID is empty".to_string()));
        }

        if client_secret.is_empty() {
            return Err(Error::InitializationError(
                "Client Secret is empty".to_string(),
            ));
        }

        if org.is_empty() {
            return Err(Error::InitializationError(
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
            base_url,
        })
    }

    #[cfg(test)]
    async fn new_test(
        repo: Arc<dyn Repo>,
        client_id: String,
        client_secret: String,
        org: String,
        base_url: String,
    ) -> Result<(mockito::ServerGuard, Self), Error> {
        let mut auth = GithubAuthenticator::new(repo, client_id, client_secret, org, base_url)?;

        let server = mockito::Server::new_async().await;

        let url = server.url();

        auth.url = url.clone();
        auth.api_url = url.clone();

        Ok((server, auth))
    }
}

#[derive(Deserialize)]
struct GithubAccessToken {
    #[serde(default)]
    access_token: String,
    #[serde(default)]
    error: String,
    #[serde(default)]
    error_description: String,
}

#[derive(Deserialize)]
struct GithubUser {
    login: String,
    id: u64,
    name: String,
    avatar_url: String,
}

impl GithubUser {
    fn to_user(&self) -> User {
        User {
            id: self.id,
            name: self.name.clone(),
            avatar_url: self.avatar_url.clone(),
            login: self.login.clone(),
        }
    }
}

#[derive(Deserialize)]
struct GithubOrgs(Vec<GithubOrg>);
#[derive(Deserialize)]
struct GithubOrg {
    login: String,
}

#[async_trait]
impl Authenticator for GithubAuthenticator {
    async fn start_login(&self) -> Result<String, Error> {
        let url = format!(
            "{}/login/oauth/authorize?client_id={}&scope=read:user,read:org&redirect_uri={}/auth/login/callback",
            self.url, self.client_id, self.base_url,
        );

        Ok(url)
    }

    async fn login(&self, code: String) -> Result<Session, Error> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("code", &code),
        ];

        let gh_token: GithubAccessToken = client
            .post(format!("{}/login/oauth/access_token", self.url))
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|err| {
                Error::ConnectionError(format!("getting access_token: {}", err.to_string()))
            })?
            .json()
            .await
            .map_err(|err| {
                Error::SerializationError(format!("reading access_token: {}", err.to_string()))
            })?;

        if !gh_token.error.is_empty() {
            return Err(Error::PermissionDenied(format!(
                "{} ({})",
                gh_token.error_description, gh_token.error
            )));
        }

        if gh_token.access_token.is_empty() {
            return Err(Error::PermissionDenied("access token is empty".to_string()));
        }

        let gh_user: GithubUser = client
            .get(format!("{}/user", self.api_url))
            .header("Authorization", format!("token {}", gh_token.access_token))
            .header("User-Agent", "finblog")
            .send()
            .await
            .map_err(|err| Error::ConnectionError(format!("getting user: {}", err.to_string())))?
            .json()
            .await
            .map_err(|err| {
                Error::SerializationError(format!("reading user: {}", err.to_string()))
            })?;

        let gh_orgs: GithubOrgs = client
            .get(format!("{}/user/orgs", self.api_url))
            .header("Authorization", format!("token {}", gh_token.access_token))
            .header("User-Agent", "finblog")
            .send()
            .await
            .map_err(|err| Error::ConnectionError(format!("getting org: {}", err.to_string())))?
            .json()
            .await
            .map_err(|err| {
                Error::SerializationError(format!("reading org: {}", err.to_string()))
            })?;

        if !gh_orgs.0.iter().any(|org| org.login == self.org) {
            return Err(Error::PermissionDenied(
                format!("not a member of {}", self.org).to_string(),
            ));
        }

        let user = gh_user.to_user();

        let token = match self.repo.save(user.clone()).await {
            Ok(token) => token,
            Err(err) => return Err(Error::ConnectionError(err.to_string())),
        };

        Ok(Session { user, token })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_start_login() {
        let repo = Arc::new(MockRepo::new());
        let authenticator = GithubAuthenticator::new(
            repo,
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "test_org".to_string(),
            "website.local".to_string(),
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
            "website.local".to_string(),
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
            "website.local".to_string(),
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
            "website.local".to_string(),
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
