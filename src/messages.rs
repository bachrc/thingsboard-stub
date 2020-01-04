use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct IncomingRPCRequest {
    pub method: String,
    pub params: Option<String>,
}

pub enum IncomingRequest {
    GetValue(u32),
    SetValue(u32, String),
}

pub enum OutgoingMessage {
    Telemetry(HashMap<String, String>),
    AnswerGetValue(u32, String),
}
