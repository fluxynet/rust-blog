use super::{Repo, User};
use crate::errors::Error;
use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use bb8_redis::redis::AsyncCommands;
use uuid::Uuid;

pub struct RedisRepo {
    pool: Pool<RedisConnectionManager>,
    ttl: i64,
}

impl RedisRepo {
    pub async fn new(redis_url: &str, ttl: i64) -> Result<Self, Error> {
        let manager = match RedisConnectionManager::new(redis_url) {
            Ok(m) => m,
            Err(err) => return Err(Error::ConnectionError(err.to_string())),
        };

        match Pool::builder().build(manager).await {
            Ok(pool) => return Ok(RedisRepo { pool, ttl }),
            Err(err) => return Err(Error::ConnectionError(err.to_string())),
        };
    }
}

#[async_trait]
impl Repo for RedisRepo {
    async fn save(&self, user: User) -> Result<String, Error> {
        let mut con = match self.pool.get().await {
            Ok(con) => con,
            Err(err) => return Err(Error::ConnectionError(err.to_string())),
        };

        let token = Uuid::new_v4().to_string();

        let data = format!(
            "{}|{}|{}|{}",
            user.id, user.login, user.avatar_url, user.name
        );

        match con.set::<&str, String, ()>(token.as_str(), data).await {
            Ok(_) => {
                match con.expire::<&str, u32>(token.as_str(), self.ttl).await {
                    Ok(_) => (),
                    Err(err) => return Err(Error::ConnectionError(err.to_string())),
                }
                return Ok(token);
            }
            Err(err) => return Err(Error::ConnectionError(err.to_string())),
        }
    }

    async fn get(&self, token: String) -> Result<User, Error> {
        let mut con = match self.pool.get().await {
            Ok(con) => con,
            Err(err) => return Err(Error::ConnectionError(err.to_string())),
        };

        let data: String = con
            .get(token)
            .await
            .map_err(|_| Error::PermissionDenied("no session".to_string()))?;

        let segments: Vec<&str> = data.splitn(4, '|').collect();
        if segments.len() != 4 {
            return Err(Error::SerializationError("Invalid data format".to_string()));
        }

        let user = User {
            id: segments[0]
                .parse::<u64>()
                .map_err(|_| Error::SerializationError("Invalid ID format".to_string()))?,
            login: segments[1].to_string(),
            avatar_url: segments[2].to_string(),
            name: segments[3].to_string(),
        };

        Ok(user)
    }

    async fn delete(&self, token: String) -> Result<(), Error> {
        let mut con = match self.pool.get().await {
            Ok(con) => con,
            Err(err) => return Err(Error::ConnectionError(err.to_string())),
        };

        match con.del::<&str, ()>(token.as_str()).await {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::ConnectionError(err.to_string())),
        }
    }
}
