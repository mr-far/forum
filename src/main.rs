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

    let pool = PgPool::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is required"))
        .await?;

    let data = AppData {
        snowflake: SnowflakeBuilder {
            epoch: EPOCH,
            worker_id: 1,
            increment: 0,
        },
        database: DatabaseManager::new(pool.close())
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
            .app_data(web::Data::new(data))
            .app_data(web::Data::new(pool))
    })
        .bind(dotenvy::var("ADDRESS").unwrap())?
        .run()
        .await
}