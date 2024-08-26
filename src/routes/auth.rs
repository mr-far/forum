use {
    actix_web::{HttpResponse, web},
    validator::Validate,
    secrecy::ExposeSecret,
    serde::Serialize,
    regex::Regex,
    crate::{
        App,
        routes::{HttpError, Result},
        models::{
            requests::RegisterPayload,
            user::User
        }
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("auth")
            .route("/register", web::post().to(register))
    );
}

#[derive(Serialize, Validate)]
pub struct RegisterResponse {
    pub user: User,
    pub token: String
}

/// Create a new user and return [`RegisterResponse`] - `POST /register`
///
/// ### Errors
///
/// * [`HttpError::TakenUsername`] - If the username has already been taken
/// * [`HttpError::WeekPassword`] - If the password is too week
async fn register(
    payload: web::Json<RegisterPayload>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    if app.database.fetch_user_by_username(&payload.username).await.is_some() {
        return Err(HttpError::TakenUsername)
    }

    let strong_password = Regex::new(r"^[a-zA-Z0-9!@#$&()\\-`.+,/]*${12,}").unwrap();
    if !strong_password.is_match(&payload.password) {
        return Err(HttpError::WeekPassword)
    }

    let id = app.snowflake.lock().unwrap().build();

    let user = User::new(id, &payload.username, &payload.display_name)
        .save(&app.pool).await?;
    let secret = app.database.create_secret(id, &payload.password).await?;

    let token = secret.token().expose_secret().to_owned();

    Ok(HttpResponse::Ok().json(RegisterResponse{
        user,
        token
    }))
}