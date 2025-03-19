use fintrellis::auth::{AuthError, RedisRepo, User};
use redis::AsyncCommands;
use std::sync::Arc;
use testcontainers::{Container, clients::Cli, images::generic::GenericImage};
use tokio;
use tokio::time::{Duration, sleep};

async fn setup() -> (Arc<RedisRepo>, Container<GenericImage>) {
    let docker = Cli::default();
    let keydb_image = GenericImage::new("eqalpha/keydb", "latest").with_exposed_port(6379);
    let container = docker.run(keydb_image);

    let port = container.get_host_port(6379).unwrap();
    let redis_url = format!("redis://127.0.0.1:{}", port);

    // Wait for Redis to be ready
    let repo = loop {
        match RedisRepo::new(&redis_url).await {
            Ok(repo) => break repo,
            Err(_) => sleep(Duration::from_secs(1)).await,
        }
    };

    (Arc::new(repo), container)
}

#[tokio::test]
async fn test_save_and_get_user() {
    let (repo, _container) = setup().await;

    let user = User {
        id: "1".to_string(),
        name: "Test User".to_string(),
    };

    let token = repo.save(user.clone()).await.unwrap();
    let fetched_user = repo.get(token).await.unwrap();

    assert_eq!(user.id, fetched_user.id);
    assert_eq!(user.name, fetched_user.name);
}

#[tokio::test]
async fn test_delete_user() {
    let (repo, _container) = setup().await;

    let user = User {
        id: "2".to_string(),
        name: "Test User 2".to_string(),
    };

    let token = repo.save(user.clone()).await.unwrap();
    repo.delete(token.clone()).await.unwrap();

    let result = repo.get(token).await;
    assert!(result.is_err());
}
