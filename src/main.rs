mod auth;
mod blog;
mod errors;
mod web;

use serde::Deserialize;
use std::env;
use std::sync::Arc;
use tokio::fs;
#[derive(Deserialize)]
struct Config {
    base_url: String,
    dsn: String,
    auth: AuthConfig,
    admin: AdminConfig,
}

#[derive(Deserialize)]
struct AuthConfig {
    listen_addr: String,
    redis: String,
    ttl: i64,

    gh_client_id: String,
    gh_client_secret: String,
    gh_org: String,

    cookie: String,
}

#[derive(Deserialize)]
struct AdminConfig {
    listen_addr: String,
}

async fn read_config(path: &str) -> Result<Config, ()> {
    let contents = fs::read_to_string(path).await.unwrap();

    let config: Config = toml::from_str(&contents).unwrap();

    Ok(config)
}

enum Command {
    Help,
    Auth,
    Admin,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = match read_config("config.toml").await {
        Err(_) => panic!("failed to load config"),
        Ok(c) => c,
    };

    let args: Vec<String> = env::args().collect();
    let command = match args.get(1).map(|s| s.as_str()) {
        Some("auth") => Command::Auth,
        Some("admin") => Command::Admin,
        _ => Command::Help,
    };

    match command {
        Command::Help => help(),
        Command::Auth => auth_service(&config).await.unwrap(),
        Command::Admin => admin_service(&config).await.unwrap(),
    }

    Ok(())
}

fn help() {
    println!("📰 blog");
    println!("usage: blog [command]");
    println!("");
    println!("available commands:");
    println!("\t👤 auth  - start the auth service");
    println!("\t💼 admin - start the admin service");
    println!("\t📔 help  - information");
    println!("");
}

async fn auth_service(config: &Config) -> std::io::Result<()> {
    let repo = Arc::new(
        auth::redis::RedisRepo::new(&config.auth.redis, config.auth.ttl)
            .await
            .unwrap(),
    );
    let sessions = Arc::new(auth::DefaultSessionManager::new(repo.clone()));
    let authenticator = Arc::new(
        auth::github::GithubAuthenticator::new(
            repo.clone(),
            config.auth.gh_client_id.clone(),
            config.auth.gh_client_secret.clone(),
            config.auth.gh_org.clone(),
            config.base_url.clone(),
        )
        .unwrap(),
    );

    println!("🏁 starting auth service on {}", config.auth.listen_addr);

    auth::http::server(
        sessions,
        authenticator,
        config.base_url.clone(),
        config.auth.cookie.clone(),
        config.auth.listen_addr.clone(),
    )
    .await
    .unwrap();

    Ok(())
}

async fn admin_service(config: &Config) -> std::io::Result<()> {
    let admin_repo = match blog::postgres::PostgresRepo::new(config.dsn.clone()).await {
        Ok(repo) => Arc::new(repo),
        Err(err) => {
            eprintln!("Failed to connect to Postgres");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to connect to database {}", err.to_string()),
            ));
        }
    };

    let admin = Arc::new(blog::DefaultAdmin::new(admin_repo, 10));

    let auth_repo = Arc::new(
        auth::redis::RedisRepo::new(&config.auth.redis, config.auth.ttl)
            .await
            .unwrap(),
    );
    let sessions = Arc::new(auth::DefaultSessionManager::new(auth_repo.clone()));

    println!("🏁 starting admin service on {}", config.admin.listen_addr);

    blog::http::server(
        admin,
        sessions,
        config.auth.cookie.clone(),
        config.admin.listen_addr.clone(),
    )
    .await
    .unwrap();

    Ok(())
}
