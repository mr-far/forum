use {
    actix_web::{
        web, HttpResponse, HttpRequest,
        http::header::AUTHORIZATION
    },
    crate::{
        App,
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

/// Returns current [`User`] - `GET /users/@me`
async fn get_current_user(
    request: HttpRequest,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;

    app.database.fetch_user_by_token(token).await
        .map(|row| HttpResponse::Ok().json(row))
}

///  Returns [`User`] by given ID - `GET /users/{user_id}`
///
/// ### Errors
///
/// * [`HttpError::UnknownUser`] - If the user is not found
async fn get_user(
    user_id: web::Path<i64>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    app.database.fetch_user(user_id.into_inner().into())
        .await.ok_or(HttpError::UnknownUser)
        .map(|row| HttpResponse::Ok().json(row))
}