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

impl Credentials {
    pub fn from_username_and_password(username: String, password: String) -> Self {
        Self {
            username: Some(username),
            auth_type: AuthenticationType::AUTHENTICATION_USER_PASS,
            auth_data: password.into_bytes(),
        }
    }

    pub fn from_access_token(token: String) -> Self {
        Self {
            username: None,
            auth_type: AuthenticationType::AUTHENTICATION_SPOTIFY_TOKEN,
            auth_data: token.into_bytes(),
        }
    }
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
