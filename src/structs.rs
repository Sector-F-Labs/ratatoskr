use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ButtonInfo {
    pub text: String,
    pub callback_data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutgoingKafkaMessage {
    pub chat_id: i64,
    pub text: String,
    pub buttons: Option<Vec<Vec<ButtonInfo>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingCallbackMessage {
    pub chat_id: i64,
    pub user_id: u64,
    pub message_id: i32,
    pub callback_data: String,
    pub callback_query_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageInfo {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    pub file_size: u32,
    pub local_path: String,
}
