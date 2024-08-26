use {
    nanoid::nanoid,
    std::collections::HashMap,
    secrecy::{ExposeSecret, SecretString},
    serde::{Deserialize, Serialize, Serializer},
    actix_ws::{CloseCode, CloseReason, ProtocolError},
    crate::{
        models::{
            _SESSION_ID_ALPHABET,
            message::Message,
            thread::Thread,
            user::User
        },
        utils::snowflake::Snowflake,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum GatewayError {
    #[error("Decode error: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("Unknown error")]
    UnknownError,
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] ProtocolError),
    #[error("Unsupported message type")]
    UnsupportedMessageType,
    #[error("Not authenticated")]
    NotAuthenticated,
    #[error("Authentication failed")]
    AuthenticationFail,
    #[error("Already authenticated")]
    AlreadyAuthenticated,
    #[error("Rate limited")]
    RateLimited,
    #[error("Inactive connection")]
    Inactive,
    #[error("Connection closed")]
    Closed,
}

impl GatewayError {
    fn close_code(&self) -> CloseCode {
        match self {
            GatewayError::Inactive => CloseCode::Away,
            GatewayError::Closed => CloseCode::Normal,
            GatewayError::UnsupportedMessageType => CloseCode::Unsupported,

            GatewayError::WebSocketError(..) => CloseCode::Other(4000),
            GatewayError::UnknownError => CloseCode::Other(4001),
            GatewayError::Decode(..) => CloseCode::Other(4002),
            GatewayError::NotAuthenticated => CloseCode::Other(4003),
            GatewayError::AuthenticationFail => CloseCode::Other(4004),
            GatewayError::AlreadyAuthenticated => CloseCode::Other(4005),
            GatewayError::RateLimited => CloseCode::Other(4008)
        }
    }

    pub fn to_close_reason(&self) -> Option<CloseReason> {
        Some(CloseReason {
            code: self.close_code(),
            description: Some(self.to_string())
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct IncomingIdentifyPacket {
    pub token: SecretString,
}

impl Serialize for IncomingIdentifyPacket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct InternalIncomingIdentifyPacket {
            pub token: String,
        }
        InternalIncomingIdentifyPacket {
            token: self.token.expose_secret().to_string(),
        }
            .serialize(serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "op", content = "d")]
pub enum IncomingGatewayPacket {
    #[serde(rename = "ID")]
    Identify(IncomingIdentifyPacket),
    #[serde(rename = "HB")]
    Heartbeat(()),
}

#[derive(Clone, Debug)]
pub enum OutgoingGatewayPacket {
    Hello(GatewayHelloPacket),
    Event(GatewayEvent),
    HeartbeatRequest,
    HeartbeatAck,
}

impl Serialize for OutgoingGatewayPacket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        enum InternalOutgoingGatewayPacket {
            Hello { d: GatewayHelloPacket },
            Event(GatewayEvent),
            HeartbeatRequest,
            HeartbeatAck,
        }
        match self {
            Self::Hello(d) => InternalOutgoingGatewayPacket::Hello { d: d.clone() },
            Self::Event(d) => InternalOutgoingGatewayPacket::Event(d.clone()),
            Self::HeartbeatRequest => InternalOutgoingGatewayPacket::HeartbeatRequest,
            Self::HeartbeatAck => InternalOutgoingGatewayPacket::HeartbeatAck,
        }
            .serialize(serializer)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ready {
    /// The currently authenticated user.
    pub user: User,
    /// All online users. This does not include current user.
    pub users: HashMap<Snowflake, Option<User>>,
}

impl Into<GatewayEvent> for Ready {
    fn into(self) -> GatewayEvent {
        GatewayEvent::Ready(self)
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct GatewayHelloPacket {
    pub heartbeat_interval: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "a",
    content = "d",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum GatewayEvent {
    Ready(Ready),
    ThreadCreate(Thread),
    ThreadUpdate(Thread),
    ThreadDelete {
        thread_id: Snowflake,
    },
    MessageCreate(Message),
    MessageUpdate(Message),
    MessageDelete {
        thread_id: Snowflake,
        message_id: Snowflake,
    },
    ThreadTypingStart {
        thread_id: Snowflake,
        user_id: Snowflake,
    },
    ThreadTypingStop {
        thread_id: Snowflake,
        user_id: Snowflake,
    },
    UserUpdate(User),
}

pub fn new_session_id() -> String {
    nanoid!(32, &_SESSION_ID_ALPHABET)
}