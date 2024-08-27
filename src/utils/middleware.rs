use {
    actix_web::{
        Error, body::MessageBody, HttpMessage,
        dev::{ServiceRequest, ServiceResponse}, web,
        middleware::Next, http::header::AUTHORIZATION
    },
    crate::{
        App, utils::authorization::extract_header,
        models::UserCredentials
    }
};

pub async fn authorization_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    //TODO: Make this with wrap stuff in config
    if req.path().contains("/auth") {
        return Ok(next.call(req).await?);
    };

    let token = extract_header(&req.request(), AUTHORIZATION)?;
    let credentials = req.app_data::<web::Data<App>>().unwrap().database.fetch_credentials_by_token(token).await?;

    req.extensions_mut().insert(UserCredentials(credentials.0, credentials.1));

    next.call(req).await
}