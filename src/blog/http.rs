use super::Admin;
use crate::auth::{SessionManager, http::load_user};
use crate::blog::ArticlesListOptions;
use crate::web::Listing;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, delete, get, patch, post, put, web,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

struct State {
    admin: Arc<dyn Admin>,
    sessions: Arc<dyn SessionManager>,
    cookie_name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ArticleRequest {
    title: String,
    description: String,
    content: String,
}

// This is to allow openapi schema to be derived using utoipa
// because of incompatible types Uuid and DateTime present in Article
// this matches Article so no conversion needed.
#[allow(dead_code)]
#[derive(ToSchema)]
pub struct ArticleResponse {
    id: String,
    title: String,
    description: String,
    content: String,
    updated_at: String,
    created_at: String,
    status: String,
    author: String,
}

#[utoipa::path(
    post,
    path = "/articles",
    description = "Create a new article",
    tag = "blog",
    responses(
        (status = 202, description = "Article added", body = ArticleResponse),
    ),
    request_body(content=ArticleRequest, content_type = "application/json")
)]
#[post("/articles")]
pub async fn create_article(
    state: web::Data<State>,
    req: HttpRequest,
    body: web::Json<ArticleRequest>,
) -> impl Responder {
    let user = match load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        Ok(user) => user,
        Err(err) => return err.to_http_response(),
    };

    let data = body.into_inner();

    match state
        .admin
        .create(data.title, data.description, data.content, user.login)
        .await
    {
        Ok(article) => HttpResponse::Accepted().json(article),
        Err(err) => err.to_http_response(),
    }
}

#[derive(Deserialize, ToSchema)]
struct ArticlesListRequest {
    status: Option<String>,
    page: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/articles",
    description = "List articles",
    tag = "blog",
    responses(
        (status = 200, description = "Articles listing", body = Listing<ArticleResponse>),
    ),
    params(
        ("id" = u64, Path, description = "List articles"),
    )
)]
#[get("/articles")]
pub async fn list_articles(
    state: web::Data<State>,
    req: HttpRequest,
    query: web::Query<ArticlesListRequest>,
) -> impl Responder {
    if let Err(err) = load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        return err.to_http_response();
    }

    let opts = match &query.status {
        Some(s) => ArticlesListOptions::from_str(s.as_str()),
        None => ArticlesListOptions::All,
    };

    let page = match query.page {
        Some(p) => p,
        None => 0,
    };

    match state.admin.list(opts, page).await {
        Ok(listing) => HttpResponse::Ok().json(listing),
        Err(err) => err.to_http_response(),
    }
}

#[utoipa::path(
    get,
    path = "/articles/{id}",
    description = "Get a specific article",
    tag = "blog",
    responses(
        (status = 200, description = "Article", body = ArticleResponse),
    ),
    params(
        ("id" = u64, Path, description = "Article id"),
    )
)]
#[get("/articles/{id}")]
pub async fn get_article(
    state: web::Data<State>,
    req: HttpRequest,
    path: web::Path<(Uuid,)>,
) -> impl Responder {
    if let Err(err) = load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        return err.to_http_response();
    }

    let id = path.into_inner().0;

    match state.admin.get(id).await {
        Ok(article) => HttpResponse::Ok().json(article),
        Err(err) => err.to_http_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/articles/{id}",
    description = "Update article content",
    tag = "blog",
    responses(
        (status = 202, description = "Article updated"),
    ),
    params(
        ("id" = u64, Path, description = "Article id"),
    ),
    request_body(content=ArticleRequest, content_type = "application/json")
)]
#[patch("/articles/{id}")]
pub async fn update_article(
    state: web::Data<State>,
    req: HttpRequest,
    path: web::Path<(Uuid,)>,
    body: web::Json<ArticleRequest>,
) -> impl Responder {
    if let Err(err) = load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        return err.to_http_response();
    }

    let id = path.into_inner().0;
    let data = body.into_inner();

    match state
        .admin
        .update(id, data.title, data.description, data.content)
        .await
    {
        Ok(_) => HttpResponse::Accepted().finish(),
        Err(err) => err.to_http_response(),
    }
}

#[utoipa::path(
    put,
    path = "/articles/{id}/status/publish",
    description = "Publish article",
    tag = "blog",
    responses(
        (status = 202, description = "Article published"),
    ),
    params(
        ("id" = u64, Path, description = "Article id"),
    ),
)]
#[put("/articles/{id}/status/publish")]
pub async fn publish_article(
    state: web::Data<State>,
    req: HttpRequest,
    path: web::Path<(Uuid,)>,
) -> impl Responder {
    if let Err(err) = load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        return err.to_http_response();
    }

    let id = path.into_inner().0;

    match state.admin.publish(id).await {
        Ok(_) => HttpResponse::Accepted().finish(),
        Err(err) => err.to_http_response(),
    }
}

#[utoipa::path(
    put,
    path = "/articles/{id}/status/trash",
    description = "Move article to trash",
    tag = "blog",
    responses(
        (status = 202, description = "Article sent to trash"),
    ),
    params(
        ("id" = u64, Path, description = "Article id"),
    ),
)]
#[put("/articles/{id}/status/trash")]
pub async fn move_article_to_trash(
    state: web::Data<State>,
    req: HttpRequest,
    path: web::Path<(Uuid,)>,
) -> impl Responder {
    if let Err(err) = load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        return err.to_http_response();
    }

    let id = path.into_inner().0;

    match state.admin.move_to_trash(id).await {
        Ok(_) => HttpResponse::Accepted().finish(),
        Err(err) => err.to_http_response(),
    }
}

#[utoipa::path(
    put,
    path = "/articles/{id}/status/draft",
    description = "Set article to draft",
    tag = "blog",
    responses(
        (status = 202, description = "Article set to draft"),
    ),
    params(
        ("id" = u64, Path, description = "Article id"),
    ),
)]
#[put("/articles/{id}/status/draft")]
pub async fn move_article_to_draft(
    state: web::Data<State>,
    req: HttpRequest,
    path: web::Path<(Uuid,)>,
) -> impl Responder {
    if let Err(err) = load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        return err.to_http_response();
    }

    let id = path.into_inner().0;

    match state.admin.move_to_draft(id).await {
        Ok(_) => HttpResponse::Accepted().finish(),
        Err(err) => err.to_http_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/articles/{id}",
    description = "Permanently delete article",
    tag = "blog",
    responses(
        (status = 202, description = "Article deleted"),
    ),
    params(
        ("id" = u64, Path, description = "Article id"),
    ),
)]
#[delete("/articles/{id}")]
pub async fn delete_article(
    state: web::Data<State>,
    req: HttpRequest,
    path: web::Path<(Uuid,)>,
) -> impl Responder {
    if let Err(err) = load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        return err.to_http_response();
    }

    let id = path.into_inner().0;

    match state.admin.delete(id).await {
        Ok(_) => HttpResponse::Accepted().finish(),
        Err(err) => err.to_http_response(),
    }
}

pub async fn server(
    admin: Arc<dyn Admin>,
    sessions: Arc<dyn SessionManager>,
    cookie_name: String,
    listen_addr: String,
) -> Result<(), std::io::Error> {
    let data = web::Data::new(State {
        admin,
        sessions,
        cookie_name,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(create_article)
            .service(list_articles)
            .service(get_article)
            .service(update_article)
            .service(publish_article)
            .service(move_article_to_trash)
            .service(move_article_to_draft)
            .service(delete_article)
    })
    .bind(listen_addr)?
    .run()
    .await
}
