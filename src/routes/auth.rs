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
            user::User
        }
    }
};
use crate::models::secret::Secret;

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

async fn register(
    payload: web::Json<RegisterPayload>,
    app: web::Data<AppData>,
) -> Result<HttpResponse> {
    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    if app.database.fetch_user_by_username(payload.username.clone()).await.is_some() {
        return Err(HttpError::TakenUsername)
    }

    let strong_password = Regex::new(r"^[a-zA-Z0-9!@#$&()\\-`.+,/]*${12,}").unwrap();
    if !strong_password.is_match(payload.password.as_str()) {
        return Err(HttpError::WeekPassword)
    }

    let id = app.snowflake.lock().unwrap().build();

    let user = app.database.create_user(id, payload.username.clone(), payload.display_name.clone())
        .await.map(|user| User::from(user))
        .map_err(|err| HttpError::Database(err))?;
    let secret = app.database.create_secret(id, payload.password.clone())
        .await.map(|secret| Secret::from(secret))
        .map_err(|err| HttpError::Database(err))?;

    let token = secret.token().expose_secret().to_owned();

    Ok(HttpResponse::Ok().json(RegisterResponse{
        user,
        token
    }))
}