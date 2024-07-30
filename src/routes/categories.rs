use {
    actix_web::{
        web, HttpResponse, HttpRequest,
        http::header::AUTHORIZATION
    },
    validator::Validate,
    crate::{
        AppData,
        routes::{Result, HttpError},
        models::{
            category::{Category, CategoryRecord},
            user::Permissions,
            requests::{CreateCategoryPayload, CreateThreadPayload},
            message::{Message, MessageFlags, MessageRecord},
            thread::{Thread, ThreadRecord}
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

///  Returns [`Category`] by given ID - `GET /categories/{category_id}`
///
/// ### Errors
///
/// * [`HttpError::UnknownUser`] - If the owner of the category is not found
/// * [`HttpError::UnknownCategory`] - If the category is not found
async fn get_category(
    category_id: web::Path<i64>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    let category = app.database.fetch_category(category_id.into_inner().into()).await
        .ok_or(HttpError::UnknownCategory)?;

    let user = app.database.fetch_user(category.owner_id.into()).await
        .ok_or(HttpError::UnknownUser)?;

    Ok(HttpResponse::Ok().json(Category::from(category, user)))
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
    app: web::Data<AppData>,
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
    CategoryRecord {
        id: id.0,
        title: payload.title.clone(),
        description: payload.description.clone(),
        owner_id: user.id.0,
        locked: payload.is_locked
    }.save(&app.pool).await
        .map(|row| HttpResponse::Ok().json(Category::from(row, user)))
}

/// Create a new thread and return [`Thread`] - `POST /categories/threads`
///
/// ### Errors
///
/// * [`HttpError::MissingAccess`] - If the user does not have [`Permissions::CREATE_THREADS`]
/// * [`HttpError::Validation`] - If the payload is malformed or doesn't follow requirements
/// * [`HttpError::Database`] - If the database query fails
async fn create_thread(
    request: HttpRequest,
    payload: web::Json<CreateThreadPayload>,
    app: web::Data<AppData>,
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

    let message = MessageRecord {
        id: id.0,
        content: payload.content.clone(),
        thread_id: id.0,
        flags: MessageFlags::UNDELETEABLE.bits(),
        author_id: user.id.0,
        referenced_message_id: None,
        updated_at: None
    }.save(&mut *tx).await
        .map(|row| Message::from(row, user.clone()))?;

    let thread = ThreadRecord {
        id: id.0,
        author_id: user.id.0,
        title: payload.title.clone(),
        category_id: payload.category_id.into(),
        original_message_id: id.0,
        flags: 0
    }.save(&mut *tx).await
        .map(|row| Thread::from(row, user, message))?;

    tx.commit()
        .await
        .map(|_| HttpResponse::Ok().json(thread))
        .map_err(|err| HttpError::Database(err))
}

/// Delete a category - `DELETE /categories/{category_id}`
///
/// ### Path
///
/// * `category_id` - The ID of the category to delete
async fn delete_category(
    request: HttpRequest,
    category_id: web::Path<Snowflake>,
    app: web::Data<AppData>
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
