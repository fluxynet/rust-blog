use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(description = "Blog API"),
    paths(
        crate::auth::http::me,
        crate::blog::http::create_article,
        crate::blog::http::list_articles,
        crate::blog::http::get_article,
        crate::blog::http::update_article,
        crate::blog::http::publish_article,
        crate::blog::http::move_article_to_trash,
        crate::blog::http::move_article_to_draft,
        crate::blog::http::delete_article,
    ),
    components(schemas())
)]

pub struct Doc;
