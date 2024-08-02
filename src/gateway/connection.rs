use {
    actix_ws::Closed,
    futures::StreamExt,
    secrecy::ExposeSecret,
    std::sync::Arc,
    tokio::{select, time::{sleep, Duration}},
    tokio::sync::broadcast::error::RecvError,
    crate::{
        App,
        DispatchTarget,
        models::{
            user::User,
            thread::Thread,
            gateway::{
                GatewayEvent, GatewayHelloPacket, Ready,
                IncomingGatewayPacket, OutgoingGatewayPacket,
                GatewayError
            }
        },
    }
};

pub struct GatewayConnection {
    /// The application.
    pub app: Arc<App>,
    /// The websocket session.
    pub session: actix_ws::Session,
    /// The websocket stream
    pub stream: actix_ws::MessageStream,
    /// The gateway session ID.
    pub session_id: String,
    /// The currently subscribed thread.
    pub thread: Option<Thread>,
    /// The currently authenticated user.
    pub user: Option<User>,
}

pub const HEARTBEAT_INTERVAL: u64 = 27500;
pub const INCOMING_MESSAGE_TIMEOUT: Duration = Duration::from_millis(HEARTBEAT_INTERVAL + 10000);

impl GatewayConnection {
    /// Send a packet.
    pub async fn send(&mut self, packet: OutgoingGatewayPacket) -> Result<(), GatewayError> {
        let message = serde_json::to_string(&packet).map_err(GatewayError::Decode)?;
        self.session
            .text(message)
            .await
            .map_err(|_| GatewayError::Closed)
    }

    /// Dispatch an event.
    pub async fn dispatch<T: Into<GatewayEvent>>(&mut self, event: T) -> Result<(), GatewayError> {
        self.send(OutgoingGatewayPacket::Event(event.into())).await
    }

    /// Get current authenticated user.
    pub fn current_user(&self) -> Result<User, GatewayError> {
        self.user.clone().ok_or(GatewayError::NotAuthenticated)
    }

    /// Handle an incoming packet.
    pub async fn handle(&mut self, packet: IncomingGatewayPacket) -> Result<(), GatewayError> {
        match packet {
            IncomingGatewayPacket::Identify(packet) => {
                if self.user.is_some() {
                    return Err(GatewayError::AlreadyAuthenticated);
                }
                let online = &self.app.online;
                let user = self.app.database.fetch_user_by_token(packet.token.expose_secret()).await
                    .map_err(|_| GatewayError::AuthenticationFail)?;

                if online.get(&user.id.clone()).is_some() {
                    return Err(GatewayError::AlreadyAuthenticated);
                }

                self.user = Some(user.clone());
                //self.app.online.insert(user.id, *self);
                self.dispatch(Ready {
                    user,
                    users: online.iter().map(|x| (x.0.clone(), x.1.user.clone().unwrap())).collect(), // User can be added only at line 77
                })
                    .await
            }
            IncomingGatewayPacket::Heartbeat(_) => {
                self.send(OutgoingGatewayPacket::HeartbeatAck).await
            }
        }
    }

    /// Starts a gateway connection.
    async fn start(&mut self) -> Result<(), GatewayError> {
        self.send(OutgoingGatewayPacket::Hello(GatewayHelloPacket {
            heartbeat_interval: HEARTBEAT_INTERVAL,
        }))
            .await?;

        let mut receiver = self.app.channel.subscribe();

        loop {
            select! {
                _ = sleep(INCOMING_MESSAGE_TIMEOUT) => {
                    return self.session.clone().close(GatewayError::Inactive.to_close_reason()).await.map_err(|_| GatewayError::Closed);
                },
                message = receiver.recv() => {
                    match message {
                        Ok((target, event)) => if let Some(user) = self.user.clone() {
                            match target {
                                DispatchTarget::Global => self.dispatch(event).await?,
                                DispatchTarget::User(target_id) if user.id == target_id => self.dispatch(event).await?,
                                _ => (),
                            }
                        },
                        Err(err) => if matches!(err, RecvError::Closed) {
                            return Err(GatewayError::Closed);
                        },
                    }
                },
                Some(message) = self.stream.next() => {
                    let packet = match message.map_err(GatewayError::WebSocketError)? {
                        actix_ws::Message::Text(text) => serde_json::from_str::<IncomingGatewayPacket>(&text).map_err(GatewayError::Decode),
                        actix_ws::Message::Close(..) => {
                            if let Some(_) = self.user.clone() {
                                //self.app.online.remove(&user.id.clone());
                            }

                            return Err(GatewayError::Closed)
                        },
                        _ => Err(GatewayError::UnsupportedMessageType),
                    }?;
                    self.handle(packet).await?;
                },
                else => break
            }
        }
        Ok(())
    }

    /// Starts a gateway connection, with automatic GatewayError handling.
    pub async fn run(&mut self) -> Result<(), Closed> {
        match self.start().await {
            Ok(ok) => Ok(ok),
            Err(err) => self.session
                .clone()
                .close(err.to_close_reason())
                .await
        }
    }
}

unsafe impl Sync for GatewayConnection {}
unsafe impl Send for GatewayConnection {}