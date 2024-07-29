use serde::Deserialize;
use sqlx::any::AnyRow;
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
            user::Permissions,
            requests::{CreateMessagePayload, ModifyMessagePayload},
            message::{Message, MessageRecord, MessageFlags},
            thread::Thread
        },
        utils::authorization::extract_header
    }
};
use crate::utils::snowflake::Snowflake;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("threads")
            .route("{thread_id}", web::get().to(get_thread))
            .route("{thread_id}", web::delete().to(delete_thread))
            .service(
                web::scope("{thread_id}/messages")
                    .route("", web::get().to(get_messages))
                    .route("", web::post().to(create_message))
                    .route("{message_id}", web::delete().to(delete_message))
                    .route("{message_id}", web::patch().to(modify_message))
            )
    );
}

async fn get_thread(
    category_id: web::Path<i64>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    let thread = app.database.fetch_thread(category_id.into_inner().into())
        .await.ok_or(HttpError::UnknownCategory)?;
    let user = app.database.fetch_user(thread.author_id.into())
        .await.ok_or(HttpError::UnknownUser)?;
    let message = app.database.fetch_message(thread.id.into(), thread.original_message_id.into())
        .await.ok_or(HttpError::UnknownMessage)
        .map(|message| Message::from(message, user.clone()))?;

    Ok(HttpResponse::Ok().json(Thread::from(thread, user, message)))
}

async fn delete_thread(
    request: HttpRequest,
    thread_id: web::Path<i64>,
    app: web::Data<AppData>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;
    let thread = app.database.fetch_thread(thread_id.to_owned().into())
        .await.ok_or(HttpError::UnknownThread)?;

    if user.id.0 != thread.author_id && !user.has_permission(Permissions::MANAGE_THREADS) {
        return Err(HttpError::MissingAccess);
    }

    thread.delete(&app.pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct SearchMessagesQuery {
    pub limit: u16,
    pub before: Option<Snowflake>,
    pub after: Option<Snowflake>
}

async fn get_messages(
    query: web::Query<SearchMessagesQuery>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {

    Ok(HttpResponse::Ok().finish())
}

async fn delete_message(
    request: HttpRequest,
    path: web::Path<(i64, i64)>,
    app: web::Data<AppData>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;
    let message = app.database.fetch_message(path.to_owned().0.into(), path.to_owned().1.into())
        .await.ok_or(HttpError::UnknownMessage)?;

    if user.id.0 != message.author_id && !user.has_permission(Permissions::MANAGE_MESSAGES) {
        return Err(HttpError::MissingAccess);
    }

    if message.is(MessageFlags::UNDELETEABLE) {
        return Err(HttpError::Undeletable)
    }

    message.delete(&app.pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

async fn create_message(
    request: HttpRequest,
    thread_id: web::Path<i64>,
    payload: web::Json<CreateMessagePayload>,
    app: web::Data<AppData>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;

    if !user.clone().has_permission(Permissions::SEND_MESSAGES) {
        return Err(HttpError::MissingAccess)
    }

    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    let id = app.snowflake.lock().unwrap().build();

    let message = MessageRecord {
        id: id.0,
        content: payload.content.clone(),
        thread_id: thread_id.to_owned(),
        flags: 0,
        author_id: user.id.0,
        referenced_message_id: payload.referenced_message_id.map(|x| x.into()),
        updated_at: None
    }.save(&app.pool).await
        .map(|row| Message::from(row, user))?;

    Ok(HttpResponse::Ok().json(message))
}

async fn modify_message(
    request: HttpRequest,
    path: web::Path<(i64, i64)>,
    payload: web::Json<ModifyMessagePayload>,
    app: web::Data<AppData>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;
    let message = app.database.fetch_message(path.to_owned().0.into(), path.to_owned().1.into())
        .await.ok_or(HttpError::UnknownMessage)?;

    if user.id.0 != message.author_id {
        return Err(HttpError::MissingAccess);
    }

    app.database.update_message(path.into_inner().1.into(), payload.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}