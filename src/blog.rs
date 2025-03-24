use crate::errors::Error;
use crate::web::Listing;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

pub mod http;
pub mod postgres;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Article {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub content: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub status: Status,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Published,
    Draft,
    Trash,
}

impl Status {
    fn to_string(&self) -> String {
        match self {
            Status::Published => "published".to_string(),
            Status::Draft => "draft".to_string(),
            Status::Trash => "trash".to_string(),
        }
    }

    fn from_string(s: String) -> Status {
        match s.as_str() {
            "published" => Status::Published,
            "draft" => Status::Draft,
            "trash" => Status::Trash,
            _ => panic!("Invalid status string"),
        }
    }
}

pub enum ArticlesListOptions {
    All,
    Filtered(Status),
}

impl ArticlesListOptions {
    pub fn from_str(s: &str) -> Self {
        match s {
            "all" => ArticlesListOptions::All,
            "published" => ArticlesListOptions::Filtered(Status::Published),
            "draft" => ArticlesListOptions::Filtered(Status::Draft),
            "trash" => ArticlesListOptions::Filtered(Status::Trash),
            _ => ArticlesListOptions::All,
        }
    }
}

#[async_trait]
pub trait Repo: Sync + Send {
    // create

    async fn article_create(&self, article: Article) -> Result<Article, Error>;

    // read

    async fn articles_get(&self, id: Uuid) -> Result<Article, Error>;
    async fn articles_list(
        &self,
        opts: ArticlesListOptions,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Article>, i64), Error>;

    async fn articles_exists(&self, id: Uuid) -> Result<(), Error>;

    // update

    async fn article_update(
        &self,
        id: Uuid,
        title: String,
        description: String,
        content: String,
    ) -> Result<(), Error>;

    async fn article_set_status(&self, id: Uuid, status: Status) -> Result<(), Error>;

    // delete

    async fn article_delete(&self, id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait Admin: Send + Sync {
    // create

    async fn create(
        &self,
        title: String,
        description: String,
        content: String,
        author: String,
    ) -> Result<Article, Error>;

    //read

    async fn get(&self, id: Uuid) -> Result<Article, Error>;

    async fn list(&self, opts: ArticlesListOptions, page: i64) -> Result<Listing<Article>, Error>;

    // update

    async fn update(
        &self,
        id: Uuid,
        title: String,
        description: String,
        content: String,
    ) -> Result<(), Error>;

    async fn publish(&self, id: Uuid) -> Result<(), Error>;

    async fn move_to_draft(&self, id: Uuid) -> Result<(), Error>;

    async fn move_to_trash(&self, id: Uuid) -> Result<(), Error>;

    // delete

    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}

pub struct DefaultAdmin {
    repo: Arc<dyn Repo>,
    list_page_size: i64,
}

impl DefaultAdmin {
    pub fn new(repo: Arc<dyn Repo>, list_page_size: i64) -> Self {
        DefaultAdmin {
            repo,
            list_page_size,
        }
    }
}

#[async_trait]
impl Admin for DefaultAdmin {
    async fn create(
        &self,
        title: String,
        description: String,
        content: String,
        author: String,
    ) -> Result<Article, Error> {
        if title.is_empty() {
            return Err(Error::InvalidInput("title cannot be empty".to_string()));
        }

        if description.is_empty() {
            return Err(Error::InvalidInput(
                "description cannot be empty".to_string(),
            ));
        }

        if content.is_empty() {
            return Err(Error::InvalidInput("content cannot be empty".to_string()));
        }

        if author.is_empty() {
            return Err(Error::InvalidInput("author cannot be empty".to_string()));
        }

        let id = Uuid::new_v4();
        let created_at: DateTime<Utc> = Utc::now();
        let updated_at = Utc::now();
        let status = Status::Draft;

        let article = Article {
            id,
            title,
            description,
            content,
            author,
            created_at,
            updated_at,
            status,
        };

        let article = self.repo.article_create(article).await?;

        Ok(article)
    }

    async fn get(&self, id: Uuid) -> Result<Article, Error> {
        let article = self.repo.articles_get(id).await?;

        Ok(article)
    }

    async fn list(&self, opts: ArticlesListOptions, page: i64) -> Result<Listing<Article>, Error> {
        let offset = (page - 1) * self.list_page_size;
        let (articles, count) = self
            .repo
            .articles_list(opts, self.list_page_size, offset)
            .await?;

        let pages = (count as f64 / self.list_page_size as f64).ceil() as i64;
        let listing = Listing {
            items: articles,
            pages,
        };

        Ok(listing)
    }

    async fn update(
        &self,
        id: Uuid,
        title: String,
        description: String,
        content: String,
    ) -> Result<(), Error> {
        self.repo.articles_get(id).await?;

        if title.is_empty() {
            return Err(Error::InvalidInput("title cannot be empty".to_string()));
        }

        if description.is_empty() {
            return Err(Error::InvalidInput(
                "description cannot be empty".to_string(),
            ));
        }

        if content.is_empty() {
            return Err(Error::InvalidInput("content cannot be empty".to_string()));
        }

        self.repo
            .article_update(id, title, description, content)
            .await
    }

    async fn publish(&self, id: Uuid) -> Result<(), Error> {
        self.repo.articles_exists(id).await?;
        self.repo.article_set_status(id, Status::Published).await
    }

    async fn move_to_draft(&self, id: Uuid) -> Result<(), Error> {
        self.repo.articles_exists(id).await?;
        self.repo.article_set_status(id, Status::Draft).await
    }

    async fn move_to_trash(&self, id: Uuid) -> Result<(), Error> {
        self.repo.articles_exists(id).await?;
        self.repo.article_set_status(id, Status::Trash).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        self.repo.articles_exists(id).await?;
        self.repo.article_delete(id).await
    }
}
