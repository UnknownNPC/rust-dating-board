use actix_web::{web, HttpResponse, Responder};

use sailfish::TemplateOnce;

use crate::db::DbProvider;

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home<'a> {
    head_title: &'a str,
}

pub async fn homepage() -> impl Responder {
    HttpResponse::Ok().body(
        Home {
            head_title: "hello",
        }
        .render_once()
        .unwrap(),
    )
}

pub async fn google_verify_token(
    db_provider: web::Data<DbProvider>,
    token: web::Path<(String,)>,
) -> impl Responder {
    HttpResponse::Ok().body(
        Home {
            head_title: &token.0,
        }
        .render_once()
        .unwrap(),
    )
}
