use {
    actix_web::{
        web, HttpResponse, HttpRequest,
        http::header::AUTHORIZATION
    },
    validator::Validate,
    serde::Deserialize,
    crate::{
        App,
        DispatchTarget,
        routes::{Result, HttpError},
        models::{
            user::Permissions,
            requests::{CreateMessagePayload, ModifyMessagePayload},
            message::{Message, MessageFlags},
            gateway::GatewayEvent::*
        },
        utils::{
            authorization::extract_header,
            snowflake::Snowflake
        }
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("threads")
            .route("{thread_id}", web::get().to(get_thread))
            .route("{thread_id}", web::delete().to(delete_thread))
            .service(
                web::scope("{thread_id}/messages")
                    .route("", web::post().to(create_message))
                    .route("", web::get().to(get_messages))
                    .route("{message_id}", web::get().to(get_message))
                    .route("{message_id}", web::patch().to(modify_message))
                    .route("{message_id}", web::delete().to(delete_message))
            )
    );
}

///  Returns [`Thread`] by given ID - `GET /threads/{thread_id}`
///
/// ### Errors
///
/// * [`HttpError::UnknownThread`] - If the thread is not found
/// * [`HttpError::UnknownUser`] - If the owner of the thread is not found
/// * [`HttpError::UnknownMessage`] - If the original message of the thread is not found
async fn get_thread(
    thread_id: web::Path<i64>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    let thread = app.database.fetch_thread(thread_id.to_owned().into())
        .await?;

    Ok(HttpResponse::Ok().json(thread))
}

/// Delete a thread - `DELETE /threads/{thread_id}`
///
/// ### Path
///
/// * `thread_id` - The ID of the thread to delete
async fn delete_thread(
    request: HttpRequest,
    thread_id: web::Path<i64>,
    app: web::Data<App>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;
    let thread = app.database.fetch_thread(thread_id.to_owned().into())
        .await?;

    if user.id != thread.author.id && !user.has_permission(Permissions::MANAGE_THREADS) {
        return Err(HttpError::MissingAccess);
    }

    _ = app.dispatch(DispatchTarget::Global, ThreadDelete {category_id: thread.category_id.into(), thread_id: thread_id.to_owned().into()});

    thread.delete(&app.pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct SearchMessagesQuery {
    pub limit: Option<u16>,
    pub before: Option<Snowflake>,
    pub after: Option<Snowflake>
}

///  Returns [`Vec<Message>`] of the thread - `GET /threads/{thread_id}/messages`
///
/// ### Query
///
/// * `limit` - Max number of messages to return (1-100, default 50)
/// * `after` - Get messages after this message ID
/// * `before` - Get messages before this message ID
///
/// ### Errors
///
/// * [`HttpError::UnknownThread`] - If the thread is not found
async fn get_messages(
    request: HttpRequest,
    path: web::Path<i64>,
    query: web::Query<SearchMessagesQuery>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    app.database.fetch_user_by_token(token).await?;

    let messages = app.database.fetch_messages(path.to_owned().into(), query.limit, query.before, query.after).await?;

    Ok(HttpResponse::Ok().json(messages))
}

///  Returns [`Message`] by given ID - `GET /threads/{thread_id}/messages/{message_id}`
///
/// ### Path
///
/// * `thread_id` - The ID of the thread
/// * `message_id` - The ID of the message to fetch
///
/// ### Errors
///
/// * [`HttpError::UnknownMessage`] - If the message is not found
async fn get_message(
    request: HttpRequest,
    path: web::Path<(i64, i64)>,
    app: web::Data<App>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    app.database.fetch_user_by_token(token).await?;

    let message = app.database.fetch_message(path.to_owned().0.into(), path.to_owned().1.into())
        .await.ok_or(HttpError::UnknownMessage)?;

    Ok(HttpResponse::Ok().json(message))
}

/// Create a new message and return [`Message`] - `POST /threads/{thread_id}/messages`
///
/// ### Path
///
/// * `thread_id` - The ID of the thread
///
/// ### Errors
///
/// * [`HttpError::MissingAccess`] - If the user does not have [`Permissions::SEND_MESSAGES`]
/// * [`HttpError::Validation`] - If the payload is malformed or doesn't follow requirements
/// * [`HttpError::UnknownMessage`] - If the reference message is not found
async fn create_message(
    request: HttpRequest,
    thread_id: web::Path<i64>,
    payload: web::Json<CreateMessagePayload>,
    app: web::Data<App>
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

    let message = Message::new(id, user, thread_id.to_owned().into(), &payload.content, None)
        .save(&app.pool).await?;

    _ = app.dispatch(DispatchTarget::Global, MessageCreate(message.clone()));

    Ok(HttpResponse::Ok().json(message))
}

/// Updates a message - `PATCH /threads/{thread_id}/messages/{message_id}`
///
/// ### Path
///
/// * `thread_id` - The ID of the thread
/// * `message_id` - The ID of the message to delete
///
/// ### Errors
///
/// * [`HttpError::MissingAccess`] - If the user is not the message author
/// * [`HttpError::UnknownMessage`] - If the message is not found
async fn modify_message(
    request: HttpRequest,
    path: web::Path<(i64, i64)>,
    payload: web::Json<ModifyMessagePayload>,
    app: web::Data<App>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;
    let message = app.database.fetch_message(path.to_owned().0.into(), path.to_owned().1.into())
        .await.ok_or(HttpError::UnknownMessage)?;

    if user.id != message.author.id {
        return Err(HttpError::MissingAccess);
    }

    message.clone().edit(&app.pool, &payload.content).await?;

    _ = app.dispatch(DispatchTarget::Global, MessageUpdate(message.clone()));

    Ok(HttpResponse::Ok().json(message.clone()))
}

/// Delete a message - `DELETE /threads/{thread_id}/messages/{message_id}`
///
/// ### Path
///
/// * `thread_id` - The ID of the thread
/// * `message_id` - The ID of the message to delete
///
/// ### Errors
///
/// * [`HttpError::MissingAccess`] - If the user does not have [`Permissions::MANAGE_MESSAGES`]
/// * [`HttpError::UnknownMessage`] - If the message is not found
/// * [`HttpError::Undeletable`] - If the message has [`MessageFlags::UNDELETEABLE`]
async fn delete_message(
    request: HttpRequest,
    path: web::Path<(i64, i64)>,
    app: web::Data<App>
) -> Result<HttpResponse> {
    let token = extract_header(&request, AUTHORIZATION)?;
    let user = app.database.fetch_user_by_token(token).await?;
    let message = app.database.fetch_message(path.to_owned().0.into(), path.to_owned().1.into())
        .await.ok_or(HttpError::UnknownMessage)?;

    if user.id != message.author.id && !user.has_permission(Permissions::MANAGE_MESSAGES) {
        return Err(HttpError::MissingAccess);
    }

    if message.clone().is(MessageFlags::UNDELETEABLE) {
        return Err(HttpError::Undeletable)
    }

    _ = app.dispatch(DispatchTarget::Global, MessageDelete {thread_id: message.thread_id, message_id: message.id});

    message.delete(&app.pool).await?;

    Ok(HttpResponse::NoContent().finish())
}