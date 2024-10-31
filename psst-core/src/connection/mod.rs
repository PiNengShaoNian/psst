use psst_protocol::authentication::AuthenticationType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "SerializedCredentials")]
#[serde(into = "SerializedCredentials")]
pub struct Credentials {
    pub username: Option<String>,
    pub auth_data: Vec<u8>,
    pub auth_type: AuthenticationType,
}

#[derive(Serialize, Deserialize)]
struct SerializedCredentials {
    username: String,
    auth_data: String,
    auth_type: i32,
}

impl From<SerializedCredentials> for Credentials {
    fn from(value: SerializedCredentials) -> Self {
        Self {
            username: Some(value.username),
            auth_data: value.auth_data.into_bytes(),
            auth_type: value.auth_type.into(),
        }
    }
}

impl From<Credentials> for SerializedCredentials {
    fn from(value: Credentials) -> Self {
        Self {
            username: value.username.unwrap_or_default(),
            auth_data: String::from_utf8(value.auth_data)
                .expect("Invalid UTF-8 in serialized credentials"),
            auth_type: value.auth_type as _,
        }
    }
}
