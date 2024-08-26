use {
    actix_web::{
        web, HttpResponse, HttpRequest,
        http::header::AUTHORIZATION
    },
    validator::Validate,
    crate::{
        App,
        routes::{Result, HttpError},
        models::{
            user::Permissions,
            requests::{CreateCategoryPayload, CreateThreadPayload},
            message::{Message, MessageFlags},
            category::Category,
            thread::Thread
        },
        utils::{
            authorization::extract_header,
            snowflake::Snowflake
        }
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("categories")
            .route("{category_id}", web::get().to(get_category))
            // .route("{category_id}", web::patch().to(modify_category))
            .route("", web::post().to(create_category))
            .route("{category_id}", web::delete().to(delete_category))
            .route("{category_id}/threads", web::post().to(create_thread))
    );
}

///  Returns [`Category`] by given ID - `GET /categories/{category.id}`
///
/// ### Errors
///
/// * [`HttpError::UnknownUser`] - If the owner of the category is not found
/// * [`HttpError::UnknownCategory`] - If the category is not found
async fn get_category(
    category_id: web::Path<i64>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    let category = app.database.fetch_category(category_id.into_inner().into()).await
        .ok_or(HttpError::UnknownCategory)?;

    Ok(HttpResponse::Ok().json(category))
}

/// Create a new category and return [`Category`] - `POST /categories`
///
/// ### Errors
///
/// * [`HttpError::MissingAccess`] - If the user does not have [`Permissions::MANAGE_CATEGORIES`]
/// * [`HttpError::Validation`] - If the payload is malformed or doesn't follow requirements
/// * [`HttpError::Database`] - If the database query fails
async fn create_category(
    request: HttpRequest,
    payload: web::Json<CreateCategoryPayload>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;

    if !user.clone().has_permission(Permissions::MANAGE_CATEGORIES) {
        return Err(HttpError::MissingAccess)
    }

    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    let id = app.snowflake.lock().unwrap().build();
    Category::new(id, user, &payload.title, &payload.description, payload.is_locked)
        .save(&app.pool)
        .await
        .map(|row| HttpResponse::Ok().json(row))
}

/// Create a new thread and return [`Thread`] - `POST /categories/{category.id}/threads`
///
/// ### Errors
///
/// * [`HttpError::MissingAccess`] - If the user does not have [`Permissions::CREATE_THREADS`]
/// * [`HttpError::Validation`] - If the payload is malformed or doesn't follow requirements
/// * [`HttpError::Database`] - If the database query fails
async fn create_thread(
    request: HttpRequest,
    payload: web::Json<CreateThreadPayload>,
    path: web::Path<i64>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;

    if !user.clone().has_permission(Permissions::CREATE_THREADS) {
        return Err(HttpError::MissingAccess)
    }

    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    let id = app.snowflake.lock().unwrap().build();
    let mut tx = app.pool.begin().await?;

    let message = Message::new(id, user, id, &payload.content, Some(MessageFlags::UNDELETEABLE))
        .save(&mut *tx).await?;

    let thread = Thread::new(id, path.to_owned().into(), message, &payload.title, None)
        .save(&mut *tx).await?;

    tx.commit()
        .await
        .map(|_| HttpResponse::Ok().json(thread))
        .map_err(HttpError::Database)
}

/// Delete a category - `DELETE /categories/{category_id}`
///
/// ### Path
///
/// * `category_id` - The ID of the category to delete
async fn delete_category(
    request: HttpRequest,
    category_id: web::Path<Snowflake>,
    app: web::Data<App>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;
    let category = app.database.fetch_category(category_id.to_owned()).await
        .ok_or(HttpError::UnknownCategory)?;

    if !user.has_permission(Permissions::MANAGE_CATEGORIES) {
        return Err(HttpError::MissingAccess);
    }

    category.delete(&app.pool).await?;

    Ok(HttpResponse::NoContent().finish())
}
