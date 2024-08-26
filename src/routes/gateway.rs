use {
    actix_web::{
        HttpRequest, Responder, web
    },
    crate::{
        App,
        gateway::connection::GatewayConnection,
        models::gateway::new_session_id
    }
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/ws", web::get().to(gateway))
    );
}

async fn gateway(
    request: HttpRequest,
    stream: web::Payload,
    app: web::Data<App>
) -> actix_web::Result<impl Responder> {
    let (response, session, stream) = actix_ws::handle(&request, stream)?;
    let app = app.into_inner();

    actix_web::rt::spawn(async move {
        let mut connection = GatewayConnection {
            app,
            stream,
            session,
            request,
            session_id: new_session_id(),
            user: None,
        };
        connection.run().await
    });

    Ok(response)
}