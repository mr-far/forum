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
            category::Category,
            user::Permissions,
            requests::{CreateThreadPayload},
            message::Message,
            thread::Thread
        },
        utils::authorization::extract_header
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("threads")
            .route("{thread_id}", web::get().to(get_thread))
            // .route("", web::post().to(create_thread))
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
    let message = app.database.fetch_message(thread.original_message_id.into())
        .await.ok_or(HttpError::UnknownMessage)
        .map(|message| Message::from(message, user.clone()))?;

    Ok(HttpResponse::Ok().json(Thread::from(thread, user, message)))
}

// async fn create_thread(
//     request: HttpRequest,
//     payload: web::Json<CreateThreadPayload>,
//     app: web::Data<AppData>,
// ) -> Result<HttpResponse> {
//     let token = extract_header(&request, AUTHORIZATION)?;
//     let user = app.database.fetch_user_by_token(token).await?;
//
//     if !user.clone().has_permission(Permissions::CREATE_THREADS) {
//         return Err(HttpError::MissingAccess)
//     }
//
//     payload
//         .validate()
//         .map_err(|err| HttpError::Validation(err))?;
//
//     let id = app.snowflake.lock().unwrap().build();
//
// }

