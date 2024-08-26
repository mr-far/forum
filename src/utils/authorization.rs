use {
    actix_web::{
        HttpRequest,
        http::header::HeaderName
    },
    crate::routes::{HttpError, Result as HttpResult}
};

pub fn debug_ip(request: &HttpRequest) -> HttpResult<String> {
    let socket = request
        .connection_info()
        .realip_remote_addr()
        .ok_or_else(|| HttpError::InvalidCredentials("IP address is not valid".to_string()))?
        .to_string();
    Ok(socket)
}

pub fn extract_ip_from_request(request: &HttpRequest) -> HttpResult<String> {
    let socket = request
        .peer_addr()
        .ok_or_else(|| HttpError::InvalidCredentials("IP address is not valid".to_string()))?
        .ip()
        .to_canonical()
        .to_string();
    Ok(socket)
}

pub fn extract_header(request: &HttpRequest, header: HeaderName) -> HttpResult<&str> {
    let headers = request.headers();
    let header_value = headers.get(header.clone());
    header_value
        .ok_or_else(|| HttpError::Header(format!("Header {} was not found", header.to_string().to_uppercase())))?
        .to_str()
        .map_err(|_| HttpError::Header("".to_string()))
}