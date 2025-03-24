use super::{Authenticator, SessionManager, User};
use crate::errors::Error;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, cookie::Cookie, get, web};
use serde::Deserialize;
use std::sync::Arc;

struct State {
    sessions: Arc<dyn SessionManager>,
    auth: Arc<dyn Authenticator>,
    base_url: String,
    cookie_name: String,
}

pub async fn load_user(
    req: actix_web::HttpRequest,
    sessions: &Arc<dyn SessionManager>,
    cookie_name: &str,
) -> Result<User, Error> {
    if let Some(cookie) = req.cookie(cookie_name) {
        match sessions.session(cookie.value().to_string()).await {
            Ok(user) => return Ok(user),
            Err(err) => return Err(err),
        }
    }

    Err(Error::PermissionDenied("no session found".to_string()))
}

#[get("/auth/login")]
async fn login(state: web::Data<State>) -> impl Responder {
    match state.auth.start_login().await {
        Err(err) => err.to_http_response(),
        Ok(url) => HttpResponse::Found()
            .append_header(("Location", url))
            .finish(),
    }
}

#[derive(Deserialize)]
struct LoginCallback {
    code: String,
}

#[get("/auth/login/callback")]
async fn login_callback(
    state: web::Data<State>,
    query: web::Query<LoginCallback>,
) -> impl Responder {
    match state.auth.login(query.code.clone()).await {
        Err(err) => err.to_http_response(),
        Ok(session) => HttpResponse::Ok()
            .cookie(
                Cookie::build(state.cookie_name.clone(), session.token)
                    .domain(state.base_url.clone())
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .finish(),
            )
            .append_header(("Location", state.base_url.clone()))
            .finish(),
    }
}

#[get("/auth/logout")]
async fn logout(state: web::Data<State>, req: HttpRequest) -> impl Responder {
    if let Some(cookie) = req.cookie(&state.cookie_name) {
        match state.sessions.logout(cookie.value().to_string()).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => err.to_http_response(),
        }
    } else {
        HttpResponse::Ok().finish()
    }
}

#[utoipa::path(get, 
    path = "/auth/me", 
    description = "Get current user status",
    tag = "auth",
    responses(
        (status = 200, description = "Current logged in user", body = User)
    ),
)]
#[get("/auth/me")]
pub async fn me(state: web::Data<State>, req: HttpRequest) -> impl Responder {
    match load_user(req, &state.sessions, state.cookie_name.as_str()).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => err.to_http_response(),
    }
}

pub async fn server(
    sessions: Arc<dyn SessionManager>,
    auth: Arc<dyn Authenticator>,
    base_url: String,
    cookie_name: String,
    listen_addr: String,
) -> Result<(), std::io::Error> {
    let data = web::Data::new(State {
        sessions,
        auth,
        base_url,
        cookie_name,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(login)
            .service(login_callback)
            .service(logout)
            .service(me)
    })
    .bind(listen_addr)?
    .run()
    .await
}
