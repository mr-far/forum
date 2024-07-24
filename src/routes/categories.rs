use {
    actix_web::{
        web, HttpResponse
    },
    validator::Validate,
    crate::{
        AppData,
        routes::{Result, HttpError},
        models::{
            category::Category,
            user::Permissions,
            requests::CreateCategoryPayload
        },
        utils::snowflake::Snowflake
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("categories")
            .route("", web::post().to(create_category))
            .route("{category_id}", web::get().to(get_category))
    );
}

async fn create_category(
    payload: web::Json<CreateCategoryPayload>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    let user = app.database.fetch_user(Snowflake(123))
        .await
        .ok_or(HttpError::UnknownUser)?;

    if !user.clone().has_permission(Permissions::MANAGE_CATEGORIES) {
        return Err(HttpError::MissingAccess)
    }

    let id = app.snowflake.lock().unwrap().build();
    app.database.create_category(id, user.id, payload.into_inner())
        .await
        .map(|record| HttpResponse::Ok().json(Category::from(record, user)))
        .map_err(|err| HttpError::Database(err))
}

async fn get_category(
    payload: web::Json<CreateCategoryPayload>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    let user = app.database.fetch_category(Snowflake(123))
        .await
        .ok_or(HttpError::UnknownUser)?;

    if !user.clone().has_permission(Permissions::MANAGE_CATEGORIES) {
        return Err(HttpError::MissingAccess)
    }

    let id = app.snowflake.lock().unwrap().build();
    app.database.create_category(id, user.id, payload.into_inner())
        .await
        .map(|record| HttpResponse::Ok().json(Category::from(record, user)))
        .map_err(|err| HttpError::Database(err))
}