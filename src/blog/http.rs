use super::Admin;
use crate::auth::{SessionManager, http::load_user};
use crate::blog::ArticlesListOptions;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, delete, get, patch, post, put, web,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

struct State {
    admin: Arc<dyn Admin>,
    sessions: Arc<dyn SessionManager>,
    cookie_name: String,
}

#[derive(Deserialize)]
struct ArticleRequest {
    title: String,
    description: String,
    content: String,
}

#[post("/articles")]
async fn create_article(
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

#[derive(Deserialize)]
struct ArticlesListRequest {
    status: Option<String>,
    page: Option<i64>,
}

#[get("/articles")]
async fn list_articles(
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

#[get("/articles/{id}")]
async fn get_article(
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

#[patch("/articles/{id}")]
async fn update_article(
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

#[put("/articles/{id}/status/publish")]
async fn publish_article(
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

#[put("/articles/{id}/status/trash")]
async fn move_article_to_trash(
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

#[put("/articles/{id}/status/draft")]
async fn move_article_to_draft(
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

#[delete("/articles/{id}")]
async fn delete_article(
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
