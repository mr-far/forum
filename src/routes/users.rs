use {
    actix_web::{
        web, HttpResponse, HttpRequest,
        http::header::AUTHORIZATION
    },
    crate::{
        AppData,
        routes::{Result, HttpError},
        utils::authorization::extract_header
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("users")
            .route("@me", web::get().to(get_current_user))
            .route("{user_id}", web::get().to(get_user))
    );
}

async fn get_current_user(
    request: HttpRequest,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;

    app.database.fetch_user_by_token(token)
        .await.map(|user| HttpResponse::Ok().json(user))
}

async fn get_user(
    user_id: web::Path<i64>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    app.database.fetch_user(user_id.into_inner().into())
        .await.ok_or(HttpError::UnknownUser)
        .map(|user| HttpResponse::Ok().json(user))
}