use {
    actix_web::{web, App, HttpServer},
    tokio::sync::broadcast::Sender,
    std::collections::HashMap,
    env_logger::Env,
    log::info,
    sqlx::PgPool,
    forum::{
        routes,
        App as AppData,
        utils::snowflake::{EPOCH, SnowflakeBuilder},
        models::database::Database
    },
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let channel = Sender::new(16384);
    let data = web::Data::new(AppData {
        snowflake: SnowflakeBuilder {
            epoch: EPOCH,
            worker_id: 1,
            increment: 0,
        }.into(),
        channel,
        online: HashMap::new(),
        database: Database::new(pool.clone()),
        pool: pool.clone()
    });

    info!(
        "Listening for HFD Backend on {}",
        dotenvy::var("ADDRESS").unwrap()
    );
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::clone(&data))
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| routes::HttpError::Payload(err).into()),
            )
            .app_data(
                web::PathConfig::default()
                    .error_handler(|err, _| routes::HttpError::Path(err).into()),
            )
            .app_data(
                web::QueryConfig::default()
                    .error_handler(|err, _| routes::HttpError::Query(err).into()),
            )
            .configure(routes::config)
    })
        .bind(dotenvy::var("ADDRESS").unwrap())?
        .run()
        .await
}