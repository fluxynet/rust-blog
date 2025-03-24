use super::{Article, ArticlesListOptions, Repo, Status};
use crate::errors::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::stream::StreamExt;
use sqlx::{postgres::PgPool, query_builder::QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresRepo {
    db: Arc<PgPool>,
}

impl PostgresRepo {
    pub async fn new(dsn: String) -> Result<PostgresRepo, Error> {
        let db = match PgPool::connect(&dsn).await {
            Ok(pool) => Arc::new(pool),
            Err(err) => {
                return Err(Error::ConnectionError(format!(
                    "connecting to db: {}",
                    err.to_string()
                )));
            }
        };

        Ok(PostgresRepo { db })
    }
}

#[async_trait]
impl Repo for PostgresRepo {
    async fn article_create(&self, article: Article) -> Result<Article, Error> {
        let err = sqlx::query!(
            r#"INSERT INTO blog.articles (id, title, description, content, updated_at, created_at, status, author) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            article.id,
            article.title,
            article.description,
            article.content,
            article.updated_at,
            article.created_at,
            article.status.to_string(),
            article.author,
        )
        .execute(&*self.db)
        .await;

        if let Err(err) = err {
            return Err(Error::ConnectionError(format!("inserting data: {}", err)));
        }

        Ok(article)
    }

    async fn articles_get(&self, id: Uuid) -> Result<Article, Error> {
        let row = match sqlx::query!(
            r#"
            SELECT id, title, description, content, updated_at, created_at, status, author
            FROM blog.articles WHERE id = $1
            "#,
            id
        )
        .fetch_one(&*self.db)
        .await
        {
            Ok(row) => row,
            Err(sqlx::Error::RowNotFound) => {
                return Err(Error::NotFound(format!("article {} ", id)));
            }
            Err(err) => return Err(Error::ConnectionError(format!("fetching data: {}", err))),
        };

        let article = Article {
            id: row.id,
            title: row.title,
            description: row.description,
            content: row.content,
            updated_at: row.updated_at,
            created_at: row.created_at,
            status: Status::from_string(row.status),
            author: row.author,
        };

        Ok(article)
    }

    async fn articles_list(
        &self,
        opts: ArticlesListOptions,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Article>, i64), Error> {
        let mut query = QueryBuilder::new(
            r#"
        SELECT id, title, description, content, updated_at, created_at, status, author FROM blog.articles
        "#,
        );

        let mut count = QueryBuilder::new(
            r#"
        SELECT COUNT(*) FROM blog.articles
        "#,
        );

        if let ArticlesListOptions::Filtered(status) = opts {
            query.push(" WHERE status = ");
            query.push_bind(status.to_string());

            count.push(" WHERE status = ");
            count.push_bind(status.to_string());
        }

        query.push("ORDER BY created_at DESC");
        query.push(" LIMIT ");
        query.push_bind(limit);
        query.push(" OFFSET ");
        query.push_bind(offset);

        count.push(" LIMIT ");
        count.push_bind(limit);
        count.push(" OFFSET ");
        count.push_bind(offset);

        let mut items = Vec::new();
        let mut rows = query
            .build_query_as::<(
                Uuid,
                String,
                String,
                String,
                DateTime<Utc>,
                DateTime<Utc>,
                String,
                String,
            )>()
            .fetch(&*self.db);

        while let Some(row) = rows.next().await {
            let article = match row {
                Ok((id, title, description, content, updated_at, created_at, status, author)) => {
                    Article {
                        id,
                        title,
                        description,
                        content,
                        updated_at,
                        created_at,
                        status: Status::from_string(status),
                        author,
                    }
                }

                Err(err) => {
                    return Err(Error::ConnectionError(format!(
                        "fetching data: {}",
                        err.to_string()
                    )));
                }
            };

            items.push(article);
        }

        let count: i64 = count
            .build_query_scalar()
            .fetch_one(&*self.db)
            .await
            .map_err(|err| {
                Error::ConnectionError(format!("fetching count: {}", err.to_string()))
            })?;

        Ok((items, count))
    }

    async fn articles_exists(&self, id: Uuid) -> Result<(), Error> {
        let exists = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM blog.articles WHERE id = $1)"#,
            id
        )
        .fetch_one(&*self.db)
        .await
        .map_err(|err| Error::ConnectionError(format!("checking existence: {}", err)))?;

        if exists.exists.unwrap_or(false) {
            Ok(())
        } else {
            Err(Error::NotFound(format!("article {} not found", id)))
        }
    }

    async fn article_update(
        &self,
        id: Uuid,
        title: String,
        description: String,
        content: String,
    ) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE blog.articles SET title = $1, description = $2, content = $3, updated_at = $4 WHERE id = $5"#,
            title, description, content, Utc::now(), id,
        )
        .execute(&*self.db)
        .await;

        if let Err(err) = result {
            return Err(Error::ConnectionError(format!("updating data: {}", err)));
        }

        Ok(())
    }

    async fn article_set_status(&self, id: Uuid, status: Status) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE blog.articles SET status = $1, updated_at = $2 WHERE id = $3"#,
            status.to_string(),
            Utc::now(),
            id,
        )
        .execute(&*self.db)
        .await;

        if let Err(err) = result {
            return Err(Error::ConnectionError(format!("updating data: {}", err)));
        }

        Ok(())
    }

    async fn article_delete(&self, id: Uuid) -> Result<(), Error> {
        let result = sqlx::query!(r#"DELETE FROM blog.articles WHERE id = $1"#, id,)
            .execute(&*self.db)
            .await;

        if let Err(err) = result {
            return Err(Error::ConnectionError(format!("deleting data: {}", err)));
        }

        Ok(())
    }
}
