use std::sync::Arc;
use {
    actix_web::{web, App, HttpServer},
    env_logger::Env,
    log::info,
    sqlx::PgPool,
    forum::{
        {app_config, AppData},
        utils::snowflake::{EPOCH, SnowflakeBuilder},
        models::database::DatabaseManager
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

    let app = AppData {
        snowflake: SnowflakeBuilder {
            epoch: EPOCH,
            worker_id: 1,
            increment: 0,
        }.into(),
        database: DatabaseManager::new(pool.clone())
    };

    info!(
        "Listening for HFD Backend on {}",
        dotenvy::var("ADDRESS").unwrap()
    );
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::default())
            .configure(|cfg| app_config(cfg))
            .app_data(web::Data::new(Arc::new(app)))
            .app_data(web::Data::new(pool.clone()))
    })
        .bind(dotenvy::var("ADDRESS").unwrap())?
        .run()
        .await
}