use {
    actix_web::{
        web, HttpResponse
    },
    crate::{
        AppData,
        routes::{Result, HttpError}
    }
};
use crate::utils::snowflake::Snowflake;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("users")
            .route("{user_id}", web::get().to(get_user))
    );
}

async fn get_user(
    user_id: web::Path<i64>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    app.database.fetch_user(Snowflake(user_id.into_inner()))
        .await
        .ok_or(HttpError::UnknownUser)
        .map(|user| HttpResponse::Ok().json(user))
}