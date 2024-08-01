use {
    tokio::sync::broadcast,
    std::{
        sync::Mutex, collections::HashMap
    },
    sqlx::PgPool,
    crate::{
        models::{
            gateway::GatewayEvent,
            database::DatabaseManager
        },
        utils::snowflake::{SnowflakeBuilder, Snowflake},
        gateway::connection::GatewayConnection
    }
};

pub mod utils;
pub mod routes;
pub mod gateway;
pub mod models;

#[derive(Clone)]
pub enum DispatchTarget {
    Global,
    User(Snowflake),
    Thread(Snowflake)
}

pub struct App {
    pub snowflake: Mutex<SnowflakeBuilder>,
    pub channel: broadcast::Sender<(DispatchTarget, GatewayEvent)>,
    pub online: HashMap<Snowflake, GatewayConnection>,
    pub database: DatabaseManager,
    pub pool: PgPool,
}

impl App {
    /// Dispatch to all users.
    pub fn dispatch<T: Into<GatewayEvent>>(
        &self,
        to: DispatchTarget,
        event: T,
    ) -> Result<(), broadcast::error::SendError<(DispatchTarget, GatewayEvent)>> {
        self.channel.send((to, event.into())).map(|_| ())
    }
}