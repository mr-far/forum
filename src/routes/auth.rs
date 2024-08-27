use {
    actix_web::{
        HttpResponse, HttpRequest, web,
        http::header::USER_AGENT
    },
    validator::Validate,
    secrecy::ExposeSecret,
    sha256::digest,
    serde::Serialize,
    regex::Regex,
    crate::{
        App,
        routes::{HttpError, Result},
        utils::authorization::{extract_header, extract_ip_from_request},
        models::{
            requests::{RegisterPayload, LoginPayload, LogoutPayload},
            session::Session,
            user::User
        }
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
    );
}

#[derive(Serialize, Validate)]
pub struct RegisterResponse {
    pub user: User,
    pub token: String
}

/// Create a new user and return [`RegisterResponse`] - `POST /auth/register`
///
/// ### Errors
///
/// * [`HttpError::TakenUsername`] - If the username has already been taken
/// * [`HttpError::WeekPassword`] - If the password is too week
async fn register(
    request: HttpRequest,
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

    let user = User::new(id, &payload.username, &payload.display_name, &payload.password)
        .save(&app.pool).await?;
    let secret = Session::new(id, extract_header(&request, USER_AGENT)?.to_string(), extract_ip_from_request(&request)?)
        .save(&app.pool).await?;

    let token = secret.token().expose_secret().to_owned();

    Ok(HttpResponse::Ok().json(RegisterResponse{
        user,
        token
    }))
}

#[derive(Serialize, Validate)]
pub struct LoginResponse {
    pub user: User,
    pub token: String
}

/// Create a new session and return [`LoginResponse`] - `POST /auth/login`
///
/// ### Errors
///
/// * [`HttpError::InvalidCredentials`] - If the username or password is invalid
async fn login(
    request: HttpRequest,
    payload: web::Json<LoginPayload>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    let user = app.database.fetch_user_by_username(&payload.username)
        .await.ok_or(HttpError::InvalidCredentials("Username or password is invalid".to_string()))?;

    if user.password_hash != digest(&payload.password) {
        return Err(HttpError::InvalidCredentials("Username or password is invalid".to_string()))
    }

    let ip = extract_ip_from_request(&request)?;
    let session = if let Some(session) = app.database.fetch_session_by_ip(ip.clone()).await {
        session
    } else {
        Session::new(user.id, extract_header(&request, USER_AGENT)?.to_string(), ip).save(&app.pool).await?
    };

    Ok(HttpResponse::Ok().json(LoginResponse {
        user,
        token: session.token().expose_secret().to_string()
    }))
}

/// Invalidate the given session - `POST /auth/logout`
///
/// ### Errors
///
/// * [`HttpError::Unauthorized`] - If the token is invalid
async fn logout(
    payload: web::Json<LogoutPayload>,
    app: web::Data<App>,
) -> Result<HttpResponse> {
    payload
        .validate()
        .map_err(|err| HttpError::Validation(err))?;

    app.database.fetch_credentials_by_token(&payload.token).await?.1
        .delete(&app.pool).await?;

    Ok(HttpResponse::Ok().finish())
}