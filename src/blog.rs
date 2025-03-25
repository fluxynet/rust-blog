use crate::errors::Error;
use crate::web::Listing;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use mockall::predicate::*;
use mockall::*;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[automock]
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
        let page = if page <= 0 { 1 } else { page };

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

#[cfg(test)]
mod default_admin_test {
    use super::*;

    #[tokio::test]
    async fn test_create_article() {
        let mut repo = MockRepo::new();
        let article = Article {
            id: Uuid::new_v4(),
            title: "title".to_string(),
            description: "description".to_string(),
            content: "content".to_string(),
            author: "author".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Draft,
        };

        repo.expect_article_create()
            .returning(move |_| Ok(article.clone()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin
            .create(
                "title".to_string(),
                "description".to_string(),
                "content".to_string(),
                "author".to_string(),
            )
            .await;

        assert!(result.is_ok());
        let created_article = result.unwrap();
        assert_eq!(created_article.title, "title");
        assert_eq!(created_article.description, "description");
        assert_eq!(created_article.content, "content");
        assert_eq!(created_article.author, "author");
        assert_eq!(created_article.status, Status::Draft);
    }

    #[tokio::test]
    async fn test_create_empty_title() {
        let repo = Arc::new(MockRepo::new());
        let admin = DefaultAdmin::new(repo, 10);

        let result = admin
            .create(
                "".to_string(),
                "description".to_string(),
                "content".to_string(),
                "author".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid input: title cannot be empty"
        );
    }

    #[tokio::test]
    async fn test_create_empty_description() {
        let repo = Arc::new(MockRepo::new());
        let admin = DefaultAdmin::new(repo, 10);

        let result = admin
            .create(
                "title".to_string(),
                "".to_string(),
                "content".to_string(),
                "author".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid input: description cannot be empty"
        );
    }

    #[tokio::test]
    async fn test_create_empty_content() {
        let repo = Arc::new(MockRepo::new());
        let admin = DefaultAdmin::new(repo, 10);

        let result = admin
            .create(
                "title".to_string(),
                "description".to_string(),
                "".to_string(),
                "author".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid input: content cannot be empty"
        );
    }

    #[tokio::test]
    async fn test_create_empty_author() {
        let repo = Arc::new(MockRepo::new());
        let admin = DefaultAdmin::new(repo, 10);

        let result = admin
            .create(
                "title".to_string(),
                "description".to_string(),
                "content".to_string(),
                "".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid input: author cannot be empty"
        );
    }

    #[tokio::test]
    async fn test_get_success() {
        let mut repo = MockRepo::new();
        let article = Article {
            id: Uuid::new_v4(),
            title: "title".to_string(),
            description: "description".to_string(),
            content: "content".to_string(),
            author: "author".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Draft,
        };

        let article2 = article.clone();

        repo.expect_articles_get()
            .returning(move |_| Ok(article2.clone()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.get(article.id).await;

        assert!(result.is_ok());
        let fetched_article = result.unwrap();
        assert_eq!(fetched_article.id, article.id);
        assert_eq!(fetched_article.title, article.title);
        assert_eq!(fetched_article.description, article.description);
        assert_eq!(fetched_article.content, article.content);
        assert_eq!(fetched_article.author, article.author);
        assert_eq!(fetched_article.status, article.status);
    }

    #[tokio::test]
    async fn test_get_error() {
        let mut repo = MockRepo::new();
        let id = Uuid::new_v4();

        let id2 = id.clone();
        repo.expect_articles_get()
            .with(eq(id2))
            .returning(move |_| Err(Error::NotFound("article".to_string())));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);
        let result = admin.get(id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "article not found");
    }

    #[tokio::test]
    async fn list_success() {
        let mut repo = MockRepo::new();
        let article1 = Article {
            id: Uuid::new_v4(),
            title: "title1".to_string(),
            description: "description1".to_string(),
            content: "content1".to_string(),
            author: "author1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Draft,
        };

        let article2 = Article {
            id: Uuid::new_v4(),
            title: "title2".to_string(),
            description: "description2".to_string(),
            content: "content2".to_string(),
            author: "author2".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Published,
        };

        let articles = vec![article1.clone(), article2.clone()];
        let count = articles.len() as i64;

        repo.expect_articles_list()
            .returning(move |_, _, _| Ok((articles.clone(), count)));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.list(ArticlesListOptions::All, 1).await;

        assert!(result.is_ok());
        let listing = result.unwrap();
        assert_eq!(listing.items.len(), 2);
        assert_eq!(listing.items[0].title, article1.title);
        assert_eq!(listing.items[1].title, article2.title);
        assert_eq!(listing.pages, 1);
    }

    #[tokio::test]
    async fn list_empty() {
        let mut repo = MockRepo::new();

        repo.expect_articles_list()
            .with(eq(ArticlesListOptions::All), eq(10), eq(0))
            .returning(|_, _, _| Ok((vec![], 0)));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.list(ArticlesListOptions::All, 1).await;

        assert!(result.is_ok());
        let listing = result.unwrap();
        assert_eq!(listing.items.len(), 0);
        assert_eq!(listing.pages, 0);
    }

    #[tokio::test]
    async fn list_page_negative() {
        let mut repo = MockRepo::new();

        repo.expect_articles_list()
            .with(eq(ArticlesListOptions::All), eq(10), eq(0))
            .returning(|_, _, _| Ok((vec![], 0)));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.list(ArticlesListOptions::All, -1).await;

        assert!(result.is_ok());
        let listing = result.unwrap();
        assert_eq!(listing.items.len(), 0);
        assert_eq!(listing.pages, 0);
    }

    #[tokio::test]
    async fn test_update_success() {
        let mut repo = MockRepo::new();
        let article = Article {
            id: Uuid::new_v4(),
            title: "title".to_string(),
            description: "description".to_string(),
            content: "content".to_string(),
            author: "author".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Draft,
        };

        let article2 = article.clone();

        repo.expect_articles_get()
            .returning(move |_| Ok(article2.clone()));

        repo.expect_article_update().returning(|_, _, _, _| Ok(()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin
            .update(
                article.id,
                "new title".to_string(),
                "new description".to_string(),
                "new content".to_string(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_empty_title() {
        let mut repo = MockRepo::new();
        let article = Article {
            id: Uuid::new_v4(),
            title: "title".to_string(),
            description: "description".to_string(),
            content: "content".to_string(),
            author: "author".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Draft,
        };

        let article2 = article.clone();

        repo.expect_articles_get()
            .returning(move |_| Ok(article2.clone()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin
            .update(
                article.id,
                "".to_string(),
                "new description".to_string(),
                "new content".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid input: title cannot be empty"
        );
    }

    #[tokio::test]
    async fn test_update_empty_description() {
        let mut repo = MockRepo::new();
        let article = Article {
            id: Uuid::new_v4(),
            title: "title".to_string(),
            description: "description".to_string(),
            content: "content".to_string(),
            author: "author".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Draft,
        };

        let article2 = article.clone();

        repo.expect_articles_get()
            .returning(move |_| Ok(article2.clone()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin
            .update(
                article.id,
                "new title".to_string(),
                "".to_string(),
                "new content".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid input: description cannot be empty"
        );
    }

    #[tokio::test]
    async fn test_update_empty_content() {
        let mut repo = MockRepo::new();
        let article = Article {
            id: Uuid::new_v4(),
            title: "title".to_string(),
            description: "description".to_string(),
            content: "content".to_string(),
            author: "author".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Draft,
        };

        let article2 = article.clone();

        repo.expect_articles_get()
            .returning(move |_| Ok(article2.clone()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin
            .update(
                article.id,
                "new title".to_string(),
                "new description".to_string(),
                "".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid input: content cannot be empty"
        );
    }

    #[tokio::test]
    async fn test_update_empty_author() {
        let mut repo = MockRepo::new();
        let article = Article {
            id: Uuid::new_v4(),
            title: "title".to_string(),
            description: "description".to_string(),
            content: "content".to_string(),
            author: "author".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Draft,
        };

        let article2 = article.clone();

        repo.expect_articles_get()
            .returning(move |_| Ok(article2.clone()));

        repo.expect_article_update()
            .with(
                eq(article.id),
                eq("new title".to_string()),
                eq("new description".to_string()),
                eq("new content".to_string()),
            )
            .returning(|_, _, _, _| Ok(()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin
            .update(
                article.id,
                "new title".to_string(),
                "new description".to_string(),
                "new content".to_string(),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let mut repo = MockRepo::new();

        repo.expect_articles_get()
            .returning(|_| Err(Error::NotFound("article xxx".to_string())));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin
            .update(
                Uuid::new_v4(),
                "new title".to_string(),
                "new description".to_string(),
                "new content".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "article xxx not found");
    }

    #[tokio::test]
    async fn publish_success() {
        let mut repo = MockRepo::new();
        let article_id = Uuid::new_v4();

        repo.expect_articles_exists().returning(move |_| Ok(()));
        repo.expect_article_set_status()
            .with(eq(article_id), eq(Status::Published))
            .returning(|_, _| Ok(()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.publish(article_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn publish_notfound() {
        let mut repo = MockRepo::new();

        repo.expect_articles_exists()
            .returning(|_| Err(Error::NotFound("article".to_string())));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.publish(Uuid::new_v4()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "article not found");
    }

    #[tokio::test]
    async fn move_to_draft_success() {
        let mut repo = MockRepo::new();
        let article_id = Uuid::new_v4();

        repo.expect_articles_exists().returning(move |_| Ok(()));
        repo.expect_article_set_status()
            .with(eq(article_id), eq(Status::Draft))
            .returning(|_, _| Ok(()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.move_to_draft(article_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn move_to_draft_notfound() {
        let mut repo = MockRepo::new();

        repo.expect_articles_exists()
            .returning(|_| Err(Error::NotFound("article".to_string())));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.move_to_draft(Uuid::new_v4()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "article not found");
    }

    #[tokio::test]
    async fn move_to_trash_success() {
        let mut repo = MockRepo::new();
        let article_id = Uuid::new_v4();

        repo.expect_articles_exists().returning(move |_| Ok(()));
        repo.expect_article_set_status()
            .with(eq(article_id), eq(Status::Trash))
            .returning(|_, _| Ok(()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.move_to_trash(article_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn move_to_trash_notfound() {
        let mut repo = MockRepo::new();

        repo.expect_articles_exists()
            .returning(|_| Err(Error::NotFound("article".to_string())));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.move_to_trash(Uuid::new_v4()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "article not found");
    }

    #[tokio::test]
    async fn delete_success() {
        let mut repo = MockRepo::new();
        let article_id = Uuid::new_v4();

        repo.expect_articles_exists().returning(move |_| Ok(()));
        repo.expect_article_delete()
            .with(eq(article_id))
            .returning(|_| Ok(()));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.delete(article_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_notfound() {
        let mut repo = MockRepo::new();

        repo.expect_articles_exists()
            .returning(|_| Err(Error::NotFound("article".to_string())));

        let admin = DefaultAdmin::new(Arc::new(repo), 10);

        let result = admin.delete(Uuid::new_v4()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "article not found");
    }
}
