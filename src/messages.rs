use actix::prelude::{Message, Recipient};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct WsMessage {
    pub data: String,
    pub message_type: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub client_id: String,
    pub user_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: String,
    pub client_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastEvent {
    pub data: BroadcastEventBody,
    pub user_id: String,
    pub client_id: String,
}
#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BroadcastEventBody {
    pub action: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StreamEvent {
    pub user_id: String,
    pub client_id: String,
    pub data: StreamEventBody,
}

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct StreamEventBody {
    pub stream_id: String,
    pub action: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameData {
    pub user_id: String,
    pub client_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct BroadcastState {
    pub viewer_id: String,
}
