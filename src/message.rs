use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub kind: MessageKind,
    pub channel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageKind {
    Relay(String),
    Message(String, String),
    Connect,
    Disconnect,
    SetName(String),
}
