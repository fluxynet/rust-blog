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
    auth: AuthConfig,
}

#[derive(Deserialize)]
struct AuthConfig {
    listen_addr: String,
    redis: String,

    gh_client_id: String,
    gh_client_secret: String,
    gh_org: String,

    cookie: String,
}

async fn read_config(path: &str) -> Result<Config, ()> {
    let contents = fs::read_to_string(path).await.unwrap();

    let config: Config = toml::from_str(&contents).unwrap();

    Ok(config)
}

enum Command {
    Help,
    Auth,
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
        _ => Command::Help,
    };

    match command {
        Command::Help => help(),
        Command::Auth => auth_service(&config).await.unwrap(),
    }

    Ok(())
}

fn help() {
    println!("📰 fintrellis blog");
    println!("usage: blog [command]");
    println!("");
    println!("available commands:");
    println!("\t👤 auth - start the auth microservice");
    println!("\t📔 help - information");
    println!("");
}

async fn auth_service(config: &Config) -> std::io::Result<()> {
    let repo = Arc::new(
        auth::redis::RedisRepo::new(&config.auth.redis)
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
