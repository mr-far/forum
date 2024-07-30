use {
    actix_web::{HttpResponse, web},
    validator::Validate,
    secrecy::ExposeSecret,
    serde::Serialize,
    regex::Regex,
    crate::{
        AppData,
        routes::{HttpError, Result},
        models::{
            requests::RegisterPayload,
            secret::Secret,
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
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    if app.database.fetch_user_by_username(payload.username.as_str()).await.is_some() {
        return Err(HttpError::TakenUsername)
    }

    let strong_password = Regex::new(r"^[a-zA-Z0-9!@#$&()\\-`.+,/]*${12,}").unwrap();
    if !strong_password.is_match(payload.password.as_str()) {
        return Err(HttpError::WeekPassword)
    }

    let id = app.snowflake.lock().unwrap().build();

    let user = app.database.create_user(id, payload.username.as_str(), payload.display_name.as_str()).await
        .map(|row| User::from(row))
        .map_err(|err| HttpError::Database(err))?;
    let secret = app.database.create_secret(id, payload.password.as_str()).await
        .map(|row| Secret::from(row))
        .map_err(|err| HttpError::Database(err))?;

    let token = secret.token().expose_secret().to_owned();

    Ok(HttpResponse::Ok().json(RegisterResponse{
        user,
        token
    }))
}