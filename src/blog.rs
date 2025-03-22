use crate::errors::Error;
use crate::web::{Listing, Pagination};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod postgres;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub status: Status,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
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

    fn from_string(s: &str) -> Status {
        match s {
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

// #[async_trait]
trait Repo {
    // create
    async fn article_create(&self, article: Article) -> Result<Article, Error>;

    // read
    async fn articles_get(&self, id: Uuid) -> Result<Article, Error>;
    async fn articles_list(
        &self,
        opts: ArticlesListOptions,
        page: Pagination,
    ) -> Result<Listing<Article>, Error>;

    // update
    async fn article_update(
        &self,
        id: Uuid,
        title: String,
        description: String,
        content: String,
    ) -> Result<(), Error>;

    async fn article_set_draft(&self, id: Uuid) -> Result<(), Error>;
    async fn article_move_to_trash(&self, id: Uuid) -> Result<(), Error>;
    async fn article_untrash(&self, id: Uuid) -> Result<(), Error>;
    async fn article_publish(&self, id: Uuid) -> Result<(), Error>;

    // delete
    async fn article_delete(&self, id: Uuid) -> Result<(), Error>;
}
