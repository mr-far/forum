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
            .route("{category_id}", web::get().to(get_category))
            //.route("{category_id}", web::patch().to(modify_category))
            .route("", web::post().to(create_category))
    );
}

async fn get_category(
    category_id: web::Path<i64>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    let category = app.database.fetch_category(category_id.into_inner().into())
        .await
        .ok_or(HttpError::UnknownCategory)?;

    let user = app.database.fetch_user(Snowflake(category.owner_id))
        .await
        .ok_or(HttpError::UnknownUser)?;

    Ok(HttpResponse::Ok().json(Category::from(category, user)))
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

// async fn modify_category(
//     payload: web::Json<CreateCategoryPayload>,
//     app: web::Data<AppData>,
// ) -> Result<HttpResponse> {
//     payload
//         .validate()
//         .map_err(|err| HttpError::Validation(err))?;
//
// }
