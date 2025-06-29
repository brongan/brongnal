use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, DartSignal)]
pub struct RustStartup {
    pub database_directory: String,
    pub username: Option<String>,
    pub fcm_token: Option<String>,
}

#[derive(Deserialize, DartSignal)]
pub struct RegisterUserRequest {
    pub username: String,
}

#[derive(Serialize, RustSignal)]
pub struct RegisterUserResponse {
    pub username: String,
}

#[derive(Deserialize, DartSignal)]
pub struct SendMessage {
    pub sender: String,
    pub recipient: String,
    pub message: String,
}
