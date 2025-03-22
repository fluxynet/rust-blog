use super::{Article, ArticlesListOptions, Repo, Status};
use crate::errors::Error;
use crate::web::{Listing, Pagination};
use chrono::Utc;
use sqlx::{postgres::PgPool, query_builder::QueryBuilder};
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresRepo {
    db: Arc<PgPool>,
}

pub fn new(db: Arc<PgPool>) -> PostgresRepo {
    PostgresRepo { db }
}

impl Repo for PostgresRepo {
    async fn article_create(&self, article: Article) -> Result<Article, Error> {
        let id = Uuid::new_v4();

        let err = sqlx::query!(
            r#"
            INSERT INTO blog.articles (id, title, description, content, updated_at, created_at, status, author)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            id,
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

        Ok(Article {
            id: id.to_string(),
            ..article
        })
    }

    async fn articles_get(&self, id: Uuid) -> Result<Article, Error> {
        let row = match sqlx::query!(
            r#"
            SELECT id, title, description, content, updated_at, created_at, deleted_at, status, author
            FROM blog.articles WHERE id = $1
            "#,
            id
        )
        .fetch_one(&*self.db)
        .await {
            Ok(row) => row,
            Err(err) => return Err(Error::ConnectionError(format!("fetching data: {}", err))),
        };

        let article = Article {
            id: row.id.to_string(),
            title: row.title,
            description: row.description,
            content: row.content,
            updated_at: row.updated_at,
            created_at: row.created_at,
            deleted_at: row.deleted_at,
            status: Status::from_string(&row.status),
            author: row.author,
        };

        Ok(article)
    }

    async fn articles_list(
        &self,
        opts: ArticlesListOptions,
        page: Pagination,
    ) -> Result<Listing<Article>, Error> {
        let mut query = QueryBuilder::new(
            r#"
        SELECT id, title, description, content, updated_at, created_at, deleted_at, status, author FROM blog.articles
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

            query.push(" LIMIT ");
            query.push_bind(page.limit);
            query.push(" OFFSET ");
            query.push_bind(page.offset);

            count.push(" WHERE status = ");
            count.push_bind(status.to_string());

            count.push(" LIMIT ");
            count.push_bind(page.limit);
            count.push(" OFFSET ");
            count.push_bind(page.offset);
        }

        let items = query
            .build_query_as::<Article>()
            .fetch_all(&*self.db)
            .await
            .map_err(|err| Error::ConnectionError(format!("fetching data: {}", err.to_string())))?;

        let count: i64 = count
            .build_query_scalar()
            .fetch_one(&*self.db)
            .await
            .map_err(|err| {
                Error::ConnectionError(format!("fetching count: {}", err.to_string()))
            })?;

        Ok(Listing { count, items })
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

    async fn article_set_draft(&self, id: Uuid) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE blog.articles SET status = $1, updated_at = $2 WHERE id = $3"#,
            Status::Draft.to_string(),
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

    async fn article_move_to_trash(&self, id: Uuid) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE blog.articles SET status = $1, updated_at = $2 WHERE id = $3"#,
            Status::Trash.to_string(),
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

    async fn article_untrash(&self, id: Uuid) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE blog.articles SET status = $1, updated_at = $2 WHERE id = $3"#,
            Status::Draft.to_string(),
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

    async fn article_publish(&self, id: Uuid) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE blog.articles SET status = $1, updated_at = $2 WHERE id = $3"#,
            Status::Published.to_string(),
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
